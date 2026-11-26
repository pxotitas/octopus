//! Widgets on the desktop, in one screen-sized layer under the windows.
//! They stay put until "arrange widgets" (right-click on the desktop): then
//! the layer rises over the windows, a widget moves when dragged and resizes
//! from its corner, and the case at the bottom holds the ones not in use.
//! Where they sit is kept in ~/.local/state/octopus/desktop.json.

mod battery;
mod catalog;
mod contributions;
mod digital;
mod layout;
mod menu;
mod note;
pub mod photo;
mod system;
mod tray;
mod weather;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use gtk::prelude::*;
use gtk::{gdk, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::config;
use crate::icons;
use crate::services::Services;
use crate::widgets::shape::{Kind, Shape};
use layout::Rect;

const MARGIN: i32 = 28;
const THRESHOLD: f64 = 5.0;

struct Item {
    id: &'static str,
    frame: gtk::Overlay,
    content: gtk::Widget,
    handle: gtk::Widget,
    remove: gtk::Widget,
    min: (i32, i32),
    resize: Box<dyn Fn(i32, i32)>,
    click: Option<Box<dyn Fn()>>,
}

#[derive(Clone, Copy)]
enum Hold {
    Click,
    Move { x: f64, y: f64 },
    Resize { w: i32, h: i32 },
}

struct Grab {
    item: Rc<Item>,
    kind: Hold,
    moved: bool,
}

pub struct Desktop {
    window: gtk::Window,
    fixed: gtk::Fixed,
    tray: tray::Tray,
    services: Rc<Services>,
    config: config::Desktop,
    items: RefCell<Vec<Rc<Item>>>,
    layout: RefCell<HashMap<String, Rect>>,
    grab: RefCell<Option<Grab>>,
    editing: Cell<bool>,
    screen: (i32, i32),
    left_edge: i32,
    on_change: Rc<dyn Fn(Vec<String>)>,
}

impl Desktop {
    pub fn new(
        app: &gtk::Application,
        s: &Rc<Services>,
        config: &config::Config,
        on_change: Rc<dyn Fn(Vec<String>)>,
    ) -> Option<Rc<Self>> {
        if !config.desktop.enabled {
            return None;
        }
        let screen = gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_downcast::<gdk::Monitor>()
            .map(|m| m.geometry())
            .map(|g| (g.width(), g.height()))
            .unwrap_or((1440, 900));
        let left_edge = if config.bar.right() { MARGIN } else { config.bar.width as i32 + 40 };

        let fixed = gtk::Fixed::new();
        let stage = gtk::Overlay::new();
        stage.set_child(Some(&fixed));
        let window = gtk::Window::builder().application(app).child(&stage).build();
        window.add_css_class("octopus-desktop");
        window.add_css_class(&format!("cards-{}", config.desktop.card));
        style(&config.desktop);
        window.init_layer_shell();
        window.set_namespace(Some("octopus-desktop"));
        window.set_layer(Layer::Bottom);
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_exclusive_zone(-1);
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            window.set_anchor(edge, true);
        }

        let desktop = Rc::new_cyclic(|weak: &Weak<Self>| {
            let add = weak.clone();
            let done = weak.clone();
            let tray = tray::Tray::new(
                &config.desktop,
                Rc::new(move |id: &str| {
                    if let Some(d) = add.upgrade() {
                        d.add(id, None, true);
                    }
                }),
                Rc::new(move || {
                    if let Some(d) = done.upgrade() {
                        d.set_editing(false);
                    }
                }),
            );
            Self {
                window: window.clone(),
                fixed: fixed.clone(),
                tray,
                services: s.clone(),
                config: config.desktop.clone(),
                items: RefCell::default(),
                layout: RefCell::new(layout::load()),
                grab: RefCell::default(),
                editing: Cell::new(false),
                screen,
                left_edge,
                on_change,
            }
        });
        desktop.tray.root.set_visible(false);
        stage.add_overlay(&desktop.tray.root);

        for id in &config.desktop.widgets {
            desktop.add(id, None, false);
        }
        desktop.gestures();
        window.present();
        Some(desktop)
    }

    /// To the grid from the settings, while arranging.
    fn snap(&self, v: f64) -> f64 {
        match self.config.snap {
            0 => v,
            g => (v / g as f64).round() * g as f64,
        }
    }

    fn names(&self) -> Vec<String> {
        self.items.borrow().iter().map(|i| i.id.to_owned()).collect()
    }

    fn changed(&self) {
        let names = self.names();
        self.tray.refresh(&self.config, &names);
        (self.on_change)(names);
    }

    fn clamp(&self, rect: Rect) -> Rect {
        let (sw, sh) = self.screen;
        Rect {
            x: rect.x.clamp(self.left_edge, (sw - rect.w).max(self.left_edge)),
            y: rect.y.clamp(0, (sh - rect.h).max(0)),
            ..rect
        }
    }

    /// Top right first, then down the column, then the next column left.
    fn free_spot(&self, w: i32, h: i32) -> Rect {
        let taken: Vec<Rect> = self.items.borrow().iter().filter_map(|i| self.layout.borrow().get(i.id).copied()).collect();
        let overlaps = |r: &Rect| taken.iter().any(|t| r.x < t.x + t.w && t.x < r.x + r.w && r.y < t.y + t.h && t.y < r.y + r.h);
        let (sw, sh) = self.screen;
        let mut rect = Rect { x: sw - w - MARGIN, y: MARGIN, w, h };
        while overlaps(&rect) && rect.x > self.left_edge {
            rect.y += 24;
            if rect.y + h > sh - MARGIN {
                rect.y = MARGIN;
                rect.x -= 48;
            }
        }
        self.clamp(rect)
    }

    /// Places a widget; `at` is where it was dropped, centered under the pointer.
    fn add(self: &Rc<Self>, id: &str, at: Option<(f64, f64)>, by_hand: bool) -> bool {
        let Some(entry) = catalog::entry(id) else { return false };
        if self.items.borrow().iter().any(|i| i.id == entry.id) {
            return false;
        }
        let Some(built) = catalog::build(entry.id, &self.services, &self.config) else { return false };
        let saved = self.layout.borrow().get(entry.id).copied();
        let (w, h) = saved.map(|r| (r.w, r.h)).unwrap_or(entry.default);
        let rect = match (at, saved) {
            (Some((x, y)), _) => self.clamp(Rect { x: x as i32 - w / 2, y: y as i32 - h / 2, w, h }),
            (None, Some(rect)) => self.clamp(rect),
            (None, None) => self.free_spot(w, h),
        };

        let frame = gtk::Overlay::new();
        frame.add_css_class("desktop-frame");
        built.content.set_size_request(rect.w, rect.h);
        frame.set_child(Some(&built.content));
        let handle = Shape::new(Kind::Cookie4, 30, true, Some(&icons::icon("open_in_full")));
        handle.add_css_class("resize-handle");
        handle.set_halign(gtk::Align::End);
        handle.set_valign(gtk::Align::End);
        handle.set_margin_end(20);
        handle.set_margin_bottom(6);
        // room at the top right for the remove button, on the card's corner
        built.content.set_margin_top(14);
        built.content.set_margin_end(14);
        handle.set_tooltip_text(Some(crate::i18n::t("arrasta pra mudar o tamanho")));
        frame.add_overlay(&handle);
        let remove = icons::button("close", crate::i18n::t("guardar na maleta"));
        remove.add_css_class("remove-widget");
        remove.set_halign(gtk::Align::End);
        remove.set_valign(gtk::Align::Start);

        let weak = Rc::downgrade(self);
        let gone = entry.id;
        remove.connect_clicked(move |_| {
            if let Some(d) = weak.upgrade() {
                d.remove(gone);
            }
        });
        frame.add_overlay(&remove);
        // hidden outside arranging: they shouldn't catch the pointer or show tooltips
        handle.set_can_target(self.editing.get());
        remove.set_can_target(self.editing.get());
        (built.resize)(rect.w, rect.h);
        self.fixed.put(&frame, rect.x as f64, rect.y as f64);
        if by_hand {
            frame.add_css_class("arriving");
            let f = frame.clone();
            glib::timeout_add_local_once(std::time::Duration::from_millis(30), move || f.remove_css_class("arriving"));
        }

        self.items.borrow_mut().push(Rc::new(Item {
            id: entry.id,
            frame,
            content: built.content,
            handle: handle.upcast(),
            remove: remove.upcast(),
            min: entry.min,
            resize: built.resize,
            click: built.click,
        }));
        self.layout.borrow_mut().insert(entry.id.to_owned(), rect);
        if by_hand {
            layout::save(&self.layout.borrow());
            self.changed();
        }
        true
    }

    fn remove(&self, id: &str) {
        let item = {
            let mut items = self.items.borrow_mut();
            let Some(index) = items.iter().position(|i| i.id == id) else { return };
            items.remove(index)
        };
        self.fixed.remove(&item.frame);
        self.changed();
        crate::memory::release_soon();
    }

    pub fn set_editing(&self, on: bool) {
        if self.editing.replace(on) == on {
            return;
        }
        self.window.set_layer(if on { Layer::Top } else { Layer::Bottom });
        self.window.set_keyboard_mode(if on { KeyboardMode::Exclusive } else { KeyboardMode::OnDemand });
        for item in self.items.borrow().iter() {
            item.handle.set_can_target(on);
            item.remove.set_can_target(on);
        }
        if on {
            self.window.add_css_class("editing");
            self.tray.refresh(&self.config, &self.names());
        } else {
            self.window.remove_css_class("editing");
            self.tray.set_receiving(false);
        }
        self.tray.root.set_visible(on);
    }

    fn item_at(&self, x: f64, y: f64) -> Option<(Rc<Item>, gtk::Widget)> {
        let picked = self.fixed.pick(x, y, gtk::PickFlags::DEFAULT)?;
        let item = self
            .items
            .borrow()
            .iter()
            .find(|i| picked == i.frame || picked.is_ancestor(&i.frame))
            .cloned()?;
        Some((item, picked))
    }

    fn gestures(self: &Rc<Self>) {
        let drag = gtk::GestureDrag::new();
        drag.set_button(gdk::BUTTON_PRIMARY);

        let weak = Rc::downgrade(self);
        drag.connect_drag_begin(move |gesture, x, y| {
            let Some(d) = weak.upgrade() else { return };
            let Some((item, picked)) = d.item_at(x, y) else {
                gesture.set_state(gtk::EventSequenceState::Denied);
                return;
            };
            let within = |w: &gtk::Widget| picked == *w || picked.is_ancestor(w);
            let kind = if !d.editing.get() {
                if item.click.is_none() {
                    gesture.set_state(gtk::EventSequenceState::Denied);
                    return;
                }
                Hold::Click
            } else if within(&item.remove) {
                gesture.set_state(gtk::EventSequenceState::Denied);
                return;
            } else if within(&item.handle) {
                Hold::Resize { w: item.content.width(), h: item.content.height() }
            } else {
                let (x, y) = d.fixed.child_position(&item.frame);
                Hold::Move { x, y }
            };
            if !matches!(kind, Hold::Click) {
                item.frame.add_css_class("held");
            }
            *d.grab.borrow_mut() = Some(Grab { item, kind, moved: false });
        });

        let weak = Rc::downgrade(self);
        drag.connect_drag_update(move |gesture, dx, dy| {
            let Some(d) = weak.upgrade() else { return };
            let mut grab = d.grab.borrow_mut();
            let Some(grab) = grab.as_mut() else { return };
            if !grab.moved && dx.hypot(dy) < THRESHOLD {
                return;
            }
            grab.moved = true;
            let item = &grab.item;
            let (sw, sh) = (d.screen.0 as f64, d.screen.1 as f64);
            match grab.kind {
                Hold::Click => {}
                Hold::Move { x, y } => {
                    let (w, h) = (item.frame.width() as f64, item.frame.height() as f64);
                    let x = (d.snap(x + dx)).clamp(d.left_edge as f64, (sw - w).max(d.left_edge as f64));
                    let y = (d.snap(y + dy)).clamp(0.0, (sh - h).max(0.0));
                    d.fixed.move_(&item.frame, x.round(), y.round());
                    if let Some((px, py)) = gesture.start_point() {
                        d.tray.set_receiving(d.tray.contains(&d.fixed, px + dx, py + dy));
                    }
                }
                Hold::Resize { w, h } => {
                    let (x, y) = d.fixed.child_position(&item.frame);
                    let w = d.snap(w as f64 + dx).clamp(item.min.0 as f64, (sw - x).max(item.min.0 as f64)) as i32;
                    let h = d.snap(h as f64 + dy).clamp(item.min.1 as f64, (sh - y).max(item.min.1 as f64)) as i32;
                    item.content.set_size_request(w, h);
                    (item.resize)(w, h);
                }
            }
        });

        let weak = Rc::downgrade(self);
        drag.connect_drag_end(move |gesture, dx, dy| {
            let Some(d) = weak.upgrade() else { return };
            let Some(grab) = d.grab.borrow_mut().take() else { return };
            grab.item.frame.remove_css_class("held");
            if !grab.moved {
                if let (Hold::Click, Some(click)) = (grab.kind, &grab.item.click) {
                    click();
                }
                return;
            }
            if matches!(grab.kind, Hold::Move { .. }) {
                let over = gesture
                    .start_point()
                    .is_some_and(|(px, py)| d.tray.contains(&d.fixed, px + dx, py + dy));
                d.tray.set_receiving(false);
                if over {
                    d.remove(grab.item.id);
                    return;
                }
            }
            let (x, y) = d.fixed.child_position(&grab.item.frame);
            let (w, h) = grab.item.content.size_request();
            let mut layout = d.layout.borrow_mut();
            layout.insert(grab.item.id.to_owned(), Rect { x: x as i32, y: y as i32, w, h });
            layout::save(&layout);
        });
        self.fixed.add_controller(drag);

        let context = gtk::GestureClick::new();
        context.set_button(gdk::BUTTON_SECONDARY);
        let weak = Rc::downgrade(self);
        context.connect_pressed(move |_, _, x, y| {
            if let Some(d) = weak.upgrade() {
                d.open_menu(x, y);
            }
        });
        self.fixed.add_controller(context);

        let drop = gtk::DropTarget::new(String::static_type(), gdk::DragAction::COPY);
        let weak = Rc::downgrade(self);
        drop.connect_drop(move |_, value, x, y| {
            let (Some(d), Ok(id)) = (weak.upgrade(), value.get::<String>()) else { return false };
            if d.tray.contains(&d.fixed, x, y) {
                return false;
            }
            d.add(&id, Some((x, y)), true)
        });
        self.fixed.add_controller(drop);

        let keys = gtk::EventControllerKey::new();
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(d) = weak.upgrade() {
                    d.set_editing(false);
                }
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        self.window.add_controller(keys);
    }

    fn open_menu(self: &Rc<Self>, x: f64, y: f64) {
        let row = |icon, label, danger, action: Rc<dyn Fn()>| menu::Row { icon, label, danger, action };
        let weak = Rc::downgrade(self);
        let mut rows = vec![if self.editing.get() {
            row("check", "terminar de organizar", false, Rc::new(move || {
                if let Some(d) = weak.upgrade() {
                    d.set_editing(false);
                }
            }))
        } else {
            row("dashboard_customize", "organizar widgets", false, Rc::new(move || {
                if let Some(d) = weak.upgrade() {
                    d.set_editing(true);
                }
            }))
        }];
        rows.push(row("wallpaper", "trocar o wallpaper", false, Rc::new(|| {
            crate::exec::spawn("octopus", &["open", "wallpapers"]);
        })));
        rows.push(row("settings", "configurar a área de trabalho", false, Rc::new(|| {
            crate::exec::spawn("octopus", &["settings", "desktop"]);
        })));
        if let Some((item, _)) = self.item_at(x, y) {
            let weak = Rc::downgrade(self);
            rows.push(row("inventory_2", "guardar esse widget", true, Rc::new(move || {
                if let Some(d) = weak.upgrade() {
                    d.remove(item.id);
                }
            })));
        }
        menu::open(&self.fixed, x, y, rows, || {});
    }
}

/// The corners and the note's text, from the settings; the cards' color is a
/// class on the window.
fn style(desktop: &config::Desktop) {
    let Some(display) = gdk::Display::default() else { return };
    let r = desktop.corners.min(48);
    let css = format!(
        "window.octopus-desktop .desktop-widget, window.octopus-desktop .desktop-photo picture {{ border-radius: {r}px; }}\n\
         window.octopus-desktop .desktop-clock:not(.digital) {{ border-radius: {}px; }}\n\
         window.octopus-desktop textview.note-text, window.octopus-desktop textview.note-text text {{ font-size: {}px; }}",
        if r >= 40 { 999 } else { r },
        desktop.note.size.clamp(10, 32),
    );
    let provider = gtk::CssProvider::new();
    provider.load_from_string(&css);
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_USER + 1);
}
