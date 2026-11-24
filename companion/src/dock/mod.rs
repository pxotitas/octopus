//! Ported from Velatura's dock (github.com/pxdritz1/velatura), itself from impasto
//! (github.com/andreumassanet/impasto), GPL-3.0.

mod apps;
mod shelf;

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
use shelf::Shelf;

/// How large the icon under the pointer grows.
const MAGNIFY: f64 = 1.55;
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
    shelf: Shelf,
    swell: Rc<Tween>,
    icon: i32,
    headroom: i32,
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
        config: &crate::config::Dock,
        save: Rc<dyn Fn(Vec<String>)>,
    ) -> Rc<Self> {
        let mode = config.mode.as_str();
        let icon = config.size.clamp(24, 72) as i32;
        let magnify = if config.magnify { MAGNIFY } else { 1.0 };
        // Room around the capsule for the icons to swell into: they grow from
        // their bottom centre, by their whole height (icon, padding, dots).
        let item = icon as f64 + 16.0;
        let headroom = (item * (magnify - 1.0)).ceil() as i32 + 2;
        let sides = (item * (magnify - 1.0) / 2.0).ceil() as i32 + 2;
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
        let capsule = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        capsule.add_css_class("dock-capsule");
        capsule.set_margin_top(headroom);
        capsule.set_margin_start(sides);
        capsule.set_margin_end(sides);
        capsule.set_halign(gtk::Align::Center);
        let shelf = Shelf::new(4, magnify);
        capsule.append(&shelf);
        if magnify > 1.0 {
            capsule.add_css_class("magnify");
        }
        let motion = Motion::new(&capsule, Origin::BottomEdge);
        window.set_child(Some(&motion));

        let dock = Rc::new(Self {
            window,
            motion,
            tween: Tween::new(0.0),
            capsule,
            shelf: shelf.clone(),
            swell: Tween::new(0.0),
            icon,
            headroom,
            niri: Niri::new(),
            apps: Apps::load(),
            pinned: RefCell::new(config.pinned.clone()),
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
                d.refresh_input();
            }
        });
        let weak = Rc::downgrade(&dock);
        pointer.connect_leave(move |_| {
            if let Some(d) = weak.upgrade() {
                d.hovered.set(false);
                d.settle(HIDE_DELAY);
                d.refresh_input();
            }
        });
        dock.window.add_controller(pointer);

        if dock.shelf.magnifies() {
            let swelling = gtk::EventControllerMotion::new();
            let weak = Rc::downgrade(&dock);
            swelling.connect_enter(move |_, x, _| {
                if let Some(d) = weak.upgrade() {
                    d.shelf.set_pointer(x);
                    d.set_swell(1.0);
                }
            });
            let weak = Rc::downgrade(&dock);
            swelling.connect_motion(move |_, x, _| {
                if let Some(d) = weak.upgrade() {
                    d.shelf.set_pointer(x);
                }
            });
            let weak = Rc::downgrade(&dock);
            swelling.connect_leave(move |_| {
                if let Some(d) = weak.upgrade() {
                    d.set_swell(0.0);
                }
            });
            dock.shelf.add_controller(swelling);
        }

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
        self.shelf.clear();
        let pinned_count = items.iter().filter(|i| i.pinned).count();
        for (index, item) in items.into_iter().enumerate() {
            if index == pinned_count && pinned_count > 0 {
                let divider = gtk::Separator::new(gtk::Orientation::Vertical);
                divider.add_css_class("dock-divider");
                self.shelf.append(&divider);
            }
            self.shelf.append(&self.button(item));
        }
        self.window.set_visible(self.shelf.first_child().is_some());
        crate::popup::fit(&self.capsule);
        if self.mode == "fixed" {
            let (_, height, _, _) = self.capsule.measure(gtk::Orientation::Vertical, -1);
            self.window
                .set_exclusive_zone(height - self.headroom + self.capsule.margin_bottom());
        }
    }

    fn button(self: &Rc<Self>, item: Item) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("dock-item");
        if item.active() {
            button.add_css_class("active");
        }
        button.set_tooltip_text(Some(&item.name));
        let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
        column.append(&self.icon_of(&item));
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

    /// The icon loaded at its magnified size and drawn smaller at rest, so
    /// it stays sharp when it swells.
    fn icon_of(&self, item: &Item) -> gtk::Widget {
        let picture = gtk::Picture::new();
        picture.set_can_shrink(true);
        picture.set_content_fit(gtk::ContentFit::Contain);
        picture.set_size_request(self.icon, self.icon);
        let large = (self.icon as f64 * MAGNIFY).ceil() as i32;
        let theme = gtk::IconTheme::for_display(&WidgetExt::display(&self.window));
        let scale = self.window.scale_factor();
        let paintable = match &item.icon {
            Some(icon) => theme.lookup_by_gicon(
                icon,
                large,
                scale,
                gtk::TextDirection::None,
                gtk::IconLookupFlags::empty(),
            ),
            None => theme.lookup_icon(
                item.key.trim_start_matches("class:"),
                &["application-x-executable"],
                large,
                scale,
                gtk::TextDirection::None,
                gtk::IconLookupFlags::empty(),
            ),
        };
        picture.set_paintable(Some(&paintable));
        picture.upcast()
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
        let Some(bounds) = self.capsule.compute_bounds(&self.window) else {
            return false;
        };
        let (w, h) = (bounds.width() as f64, bounds.height() as f64);
        let below = self.window.height() as f64 - (bounds.y() as f64 + h);
        let dock = [(sw - w) / 2.0, sh - h - below, w, h];
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
        self.refresh_input();
    }

    fn set_swell(&self, to: f64) {
        let shelf = self.shelf.clone();
        self.swell.animate(
            &self.shelf,
            to,
            if to > 0.5 { 180 } else { 240 },
            anim::STANDARD,
            move |v| shelf.set_swell(v),
            || {},
        );
    }

    /// Hidden, only a sliver along the screen's edge takes the pointer, so
    /// what is under the dock stays clickable. Shown, the capsule does; under
    /// the pointer, the room above it too, where the magnified icons grow.
    fn refresh_input(&self) {
        let Some(surface) = self.window.surface() else {
            return;
        };
        let (w, h) = (self.window.width().max(1), self.window.height().max(1));
        let rect = if !self.shown.get() {
            gtk::cairo::RectangleInt::new(0, (h - 3).max(0), w, 3)
        } else if self.hovered.get() || self.menu_open.get() {
            gtk::cairo::RectangleInt::new(0, 0, w, h)
        } else {
            match self.capsule.compute_bounds(&self.window) {
                Some(b) => gtk::cairo::RectangleInt::new(
                    b.x() as i32,
                    b.y() as i32,
                    b.width().ceil() as i32,
                    h - b.y() as i32,
                ),
                None => gtk::cairo::RectangleInt::new(0, 0, w, h),
            }
        };
        surface.set_input_region(Some(&gtk::cairo::Region::create_rectangle(&rect)));
    }
}
