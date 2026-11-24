mod apps;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::i18n::t;
use crate::services::niri::{Niri, Window};
use crate::widgets::anim::{self, Tween};
use crate::widgets::motion::{Motion, Origin};
use apps::Apps;

const ICON: i32 = 40;
const HIDE_DELAY: Duration = Duration::from_millis(500);

/// One app on the dock: pinned, running, or both.
struct Item {
    key: String,
    id: Option<String>,
    name: String,
    icon: Option<gio::Icon>,
    pinned: bool,
    windows: Vec<Window>,
}

impl Item {
    fn active(&self) -> bool {
        self.windows.iter().any(|w| w.focused)
    }
}

/// Velatura's dock: pinned apps first, then whatever else runs, one icon per
/// app with a dot per window. "fixed" keeps its space; "autohide" waits on
/// the screen's bottom edge; "smart" steps aside only while a window on the
/// workspace would sit under it.
pub struct Dock {
    window: gtk::Window,
    motion: Motion,
    tween: Rc<Tween>,
    capsule: gtk::Box,
    niri: Rc<Niri>,
    apps: Apps,
    pinned: RefCell<Vec<String>>,
    mode: String,
    shown: Cell<bool>,
    hovered: Cell<bool>,
    menu_open: Cell<bool>,
    timer: Cell<Option<glib::SourceId>>,
    signature: RefCell<String>,
    save: Rc<dyn Fn(Vec<String>)>,
}

impl Dock {
    pub fn new(
        app: &gtk::Application,
        mode: &str,
        pinned: Vec<String>,
        save: Rc<dyn Fn(Vec<String>)>,
    ) -> Rc<Self> {
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .build();
        window.add_css_class("octopus-popup");
        window.init_layer_shell();
        window.set_namespace(Some("octopus-dock"));
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_anchor(Edge::Bottom, true);
        if mode == "fixed" {
            window.auto_exclusive_zone_enable();
        }

        let capsule = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        capsule.add_css_class("dock-capsule");
        let motion = Motion::new(&capsule, Origin::BottomEdge);
        window.set_child(Some(&motion));

        let dock = Rc::new(Self {
            window,
            motion,
            tween: Tween::new(0.0),
            capsule,
            niri: Niri::new(),
            apps: Apps::load(),
            pinned: RefCell::new(pinned),
            mode: mode.to_owned(),
            shown: Cell::new(false),
            hovered: Cell::new(false),
            menu_open: Cell::new(false),
            timer: Cell::new(None),
            signature: RefCell::new(String::new()),
            save,
        });

        let pointer = gtk::EventControllerMotion::new();
        let weak = Rc::downgrade(&dock);
        pointer.connect_enter(move |_, _, _| {
            if let Some(d) = weak.upgrade() {
                d.hovered.set(true);
                d.settle(Duration::ZERO);
            }
        });
        let weak = Rc::downgrade(&dock);
        pointer.connect_leave(move |_| {
            if let Some(d) = weak.upgrade() {
                d.hovered.set(false);
                d.settle(HIDE_DELAY);
            }
        });
        dock.window.add_controller(pointer);

        let weak = Rc::downgrade(&dock);
        dock.niri.windows.subscribe(move |_| {
            if let Some(d) = weak.upgrade() {
                d.render();
                d.settle(HIDE_DELAY);
            }
        });
        let weak = Rc::downgrade(&dock);
        dock.niri.workspaces.subscribe(move |_| {
            if let Some(d) = weak.upgrade() {
                d.settle(Duration::ZERO);
            }
        });

        dock.motion.set_progress(0.0);
        dock.window.present();
        dock.settle(Duration::from_millis(300));
        dock
    }

    fn items(&self) -> Vec<Item> {
        let windows = self.niri.windows.get().clone();
        let mut items: Vec<Item> = self
            .pinned
            .borrow()
            .iter()
            .map(|id| self.describe(Some(id.clone()), id.clone(), true))
            .collect();
        let mut sorted = windows;
        sorted.sort_by_key(|w| w.id);
        for w in sorted {
            let id = self.apps.id_for(&w.app_id).map(str::to_owned);
            let key = id
                .clone()
                .unwrap_or_else(|| format!("class:{}", w.app_id.to_lowercase()));
            match items.iter_mut().find(|i| i.key == key) {
                Some(item) => item.windows.push(w),
                None => {
                    let mut item = self.describe(id, key, false);
                    if item.name.is_empty() {
                        item.name = w.app_id.clone();
                    }
                    item.windows.push(w);
                    items.push(item);
                }
            }
        }
        items
    }

    fn describe(&self, id: Option<String>, key: String, pinned: bool) -> Item {
        let app = id.as_deref().and_then(|id| self.apps.get(id));
        Item {
            name: app.map(|a| a.name().to_string()).unwrap_or_default(),
            icon: app.and_then(|a| a.icon()),
            key,
            id,
            pinned,
            windows: Vec::new(),
        }
    }

    fn render(self: &Rc<Self>) {
        let items = self.items();
        let signature: String = items
            .iter()
            .map(|i| format!("{}:{}:{}:{};", i.key, i.windows.len(), i.active(), i.pinned))
            .collect();
        if *self.signature.borrow() == signature {
            return;
        }
        *self.signature.borrow_mut() = signature;
        while let Some(child) = self.capsule.first_child() {
            self.capsule.remove(&child);
        }
        let pinned_count = items.iter().filter(|i| i.pinned).count();
        for (index, item) in items.into_iter().enumerate() {
            if index == pinned_count && pinned_count > 0 {
                let divider = gtk::Separator::new(gtk::Orientation::Vertical);
                divider.add_css_class("dock-divider");
                self.capsule.append(&divider);
            }
            self.capsule.append(&self.button(item));
        }
        self.window
            .set_visible(self.capsule.first_child().is_some());
        crate::popup::fit(&self.capsule);
    }

    fn button(self: &Rc<Self>, item: Item) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("dock-item");
        if item.active() {
            button.add_css_class("active");
        }
        button.set_tooltip_text(Some(&item.name));
        let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let image = match &item.icon {
            Some(icon) => gtk::Image::from_gicon(icon),
            None => gtk::Image::from_icon_name(item.key.trim_start_matches("class:")),
        };
        image.set_pixel_size(ICON);
        column.append(&image);
        let dots = gtk::Box::new(gtk::Orientation::Horizontal, 3);
        dots.set_halign(gtk::Align::Center);
        dots.add_css_class("dock-dots");
        for _ in 0..item.windows.len().min(3) {
            let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            dot.add_css_class("dock-dot");
            dots.append(&dot);
        }
        column.append(&dots);
        button.set_child(Some(&column));

        let item = Rc::new(item);
        let click = gtk::GestureClick::new();
        click.set_button(0);
        let weak = Rc::downgrade(self);
        let target = item.clone();
        click.connect_released(move |gesture, _, _, _| {
            let Some(dock) = weak.upgrade() else { return };
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let Some(button) = gesture.widget().and_downcast::<gtk::Button>() else {
                return;
            };
            match gesture.current_button() {
                gdk::BUTTON_SECONDARY => dock.menu(&button, &target),
                gdk::BUTTON_MIDDLE => dock.launch(&target),
                _ => dock.activate(&target),
            }
        });
        button.add_controller(click);
        button
    }

    /// Opens it if nothing runs, focuses it if it is not focused, otherwise
    /// cycles through its windows.
    fn activate(&self, item: &Item) {
        if item.windows.is_empty() {
            self.launch(item);
            return;
        }
        let at = item.windows.iter().position(|w| w.focused);
        let next = match at {
            None => 0,
            Some(i) => (i + 1) % item.windows.len(),
        };
        if at != Some(next) {
            self.niri.focus(item.windows[next].id);
        }
    }

    fn launch(&self, item: &Item) {
        if let Some(id) = &item.id {
            self.apps.launch(id);
        }
    }

    fn set_pinned(self: &Rc<Self>, id: &str, pin: bool) {
        {
            let mut pinned = self.pinned.borrow_mut();
            pinned.retain(|p| p != id);
            if pin {
                pinned.push(id.to_owned());
            }
        }
        (self.save)(self.pinned.borrow().clone());
        self.signature.borrow_mut().clear();
        self.render();
    }

    fn menu(self: &Rc<Self>, button: &gtk::Button, item: &Rc<Item>) {
        let popover = gtk::Popover::new();
        popover.add_css_class("dock-menu");
        popover.set_has_arrow(false);
        popover.set_position(gtk::PositionType::Top);
        let column = gtk::Box::new(gtk::Orientation::Vertical, 2);

        let row = |icon: &str, text: &str| {
            let button = gtk::Button::new();
            button.add_css_class("menu-row");
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            content.append(&crate::icons::icon(icon));
            let label = gtk::Label::new(Some(text));
            label.set_xalign(0.0);
            label.set_ellipsize(gtk::pango::EllipsizeMode::End);
            label.set_max_width_chars(28);
            content.append(&label);
            button.set_child(Some(&content));
            button
        };

        for w in &item.windows {
            let entry = row(
                if w.focused {
                    "radio_button_checked"
                } else {
                    "web_asset"
                },
                &w.title,
            );
            let (niri, id, pop) = (self.niri.clone(), w.id, popover.clone());
            entry.connect_clicked(move |_| {
                niri.focus(id);
                pop.popdown();
            });
            column.append(&entry);
        }
        if !item.windows.is_empty() {
            let line = gtk::Separator::new(gtk::Orientation::Horizontal);
            line.add_css_class("tray-separator");
            column.append(&line);
        }
        if item.id.is_some() {
            let open = row(
                "add",
                if item.windows.is_empty() {
                    t("abrir")
                } else {
                    t("nova janela")
                },
            );
            let (weak, target, pop) = (Rc::downgrade(self), item.clone(), popover.clone());
            open.connect_clicked(move |_| {
                if let Some(d) = weak.upgrade() {
                    d.launch(&target);
                }
                pop.popdown();
            });
            column.append(&open);
            let pin = row(
                if item.pinned { "keep_off" } else { "keep" },
                if item.pinned {
                    t("desafixar")
                } else {
                    t("fixar")
                },
            );
            let (weak, id, pinned, pop) = (
                Rc::downgrade(self),
                item.id.clone().unwrap_or_default(),
                item.pinned,
                popover.clone(),
            );
            pin.connect_clicked(move |_| {
                pop.popdown();
                if let Some(d) = weak.upgrade() {
                    d.set_pinned(&id, !pinned);
                }
            });
            column.append(&pin);
        }
        if !item.windows.is_empty() {
            let close = row(
                "close",
                if item.windows.len() > 1 {
                    t("fechar todas")
                } else {
                    t("fechar")
                },
            );
            let (niri, ids, pop) = (
                self.niri.clone(),
                item.windows.iter().map(|w| w.id).collect::<Vec<_>>(),
                popover.clone(),
            );
            close.connect_clicked(move |_| {
                for id in &ids {
                    niri.close(*id);
                }
                pop.popdown();
            });
            column.append(&close);
        }

        popover.set_child(Some(&column));
        popover.set_parent(button);
        self.menu_open.set(true);
        let weak = Rc::downgrade(self);
        popover.connect_closed(move |pop| {
            let pop = pop.clone();
            glib::idle_add_local_once(move || pop.unparent());
            if let Some(d) = weak.upgrade() {
                d.menu_open.set(false);
                d.settle(HIDE_DELAY);
            }
        });
        popover.popup();
    }

    /// Whether a window on this screen's workspace sits where the dock is.
    fn covered(&self) -> bool {
        let Some(surface) = self.window.surface() else {
            return false;
        };
        let Some(monitor) = WidgetExt::display(&self.window).monitor_at_surface(&surface) else {
            return false;
        };
        let geometry = monitor.geometry();
        let (sw, sh) = (geometry.width() as f64, geometry.height() as f64);
        let (w, h) = (self.window.width() as f64, self.window.height() as f64);
        let dock = [(sw - w) / 2.0, sh - h, w, h];
        let connector = monitor
            .connector()
            .map(|c| c.to_string())
            .unwrap_or_default();
        let workspaces = self.niri.workspaces.get();
        let active: Vec<u64> = workspaces
            .iter()
            .filter(|ws| ws.active && (connector.is_empty() || ws.output == connector))
            .map(|ws| ws.id)
            .collect();
        self.niri.windows.get().iter().any(|win| {
            win.workspace.is_some_and(|id| active.contains(&id))
                && win.tile.is_some_and(|t| {
                    t[0] < dock[0] + dock[2]
                        && t[0] + t[2] > dock[0]
                        && t[1] < dock[1] + dock[3]
                        && t[1] + t[3] > dock[1]
                })
        })
    }

    fn wanted(&self) -> bool {
        if self.hovered.get() || self.menu_open.get() {
            return true;
        }
        match self.mode.as_str() {
            "autohide" => false,
            "smart" => !self.covered(),
            _ => true,
        }
    }

    /// Shows or hides it after `delay`, judged again when the time comes.
    fn settle(self: &Rc<Self>, delay: Duration) {
        if let Some(id) = self.timer.take() {
            id.remove();
        }
        if delay.is_zero() {
            self.show(self.wanted());
            return;
        }
        let weak = Rc::downgrade(self);
        self.timer
            .set(Some(glib::timeout_add_local_once(delay, move || {
                if let Some(d) = weak.upgrade() {
                    d.timer.set(None);
                    d.show(d.wanted());
                }
            })));
    }

    fn show(&self, on: bool) {
        if self.shown.replace(on) == on {
            return;
        }
        let motion = self.motion.clone();
        self.tween.animate(
            &self.motion,
            if on { 1.0 } else { 0.0 },
            if on { 320 } else { 220 },
            if on {
                anim::EMPHASIZED_DECELERATE
            } else {
                anim::EMPHASIZED_ACCELERATE
            },
            move |p| motion.set_progress(p),
            || {},
        );
        self.set_input(on);
    }

    /// Hidden, only a sliver along the screen's edge takes the pointer, so
    /// what is under the dock stays clickable.
    fn set_input(&self, full: bool) {
        let Some(surface) = self.window.surface() else {
            return;
        };
        let (w, h) = (self.window.width(), self.window.height());
        let rect = if full {
            gtk::cairo::RectangleInt::new(0, 0, w.max(1), h.max(1))
        } else {
            gtk::cairo::RectangleInt::new(0, (h - 3).max(0), w.max(1), 3)
        };
        surface.set_input_region(Some(&gtk::cairo::Region::create_rectangle(&rect)));
    }
}
