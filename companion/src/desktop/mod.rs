//! Widgets on the desktop, in one screen-sized layer under the windows.
//! Dragging a widget moves it; dragging the handle in its corner resizes it.
//! Both happen in the layer's fixed coordinates, so nothing jumps, and the
//! result is kept in ~/.local/state/octopus/desktop.json.

mod contributions;
mod layout;
mod weather;

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::config;
use crate::icons;
use crate::services::Services;
use crate::widgets::shape::{Kind, Shape};
use layout::Rect;

const MARGIN: i32 = 28;
const THRESHOLD: f64 = 5.0;

struct Item {
    name: String,
    frame: gtk::Overlay,
    content: gtk::Widget,
    handle: gtk::Widget,
    min: (i32, i32),
    resize: Box<dyn Fn(i32, i32)>,
    click: Option<Box<dyn Fn()>>,
}

#[derive(Clone, Copy)]
enum Grab {
    Move { x: f64, y: f64 },
    Resize { w: i32, h: i32 },
}

pub struct Desktop {
    _window: gtk::Window,
    fixed: gtk::Fixed,
    items: Vec<Item>,
    layout: RefCell<HashMap<String, Rect>>,
    grab: Cell<Option<(usize, Grab, bool)>>,
}

impl Desktop {
    pub fn new(app: &gtk::Application, s: &Rc<Services>, config: &config::Config) -> Option<Rc<Self>> {
        let desktop = &config.desktop;
        if !desktop.enabled || desktop.widgets.is_empty() {
            return None;
        }

        let screen = gtk::gdk::Display::default()
            .and_then(|d| d.monitors().item(0))
            .and_downcast::<gtk::gdk::Monitor>()
            .map(|m| m.geometry())
            .map(|g| (g.width(), g.height()))
            .unwrap_or((1440, 900));
        let left_edge = if config.bar.right() { MARGIN } else { config.bar.width as i32 + 40 };

        let mut items = Vec::new();
        let mut next_y = MARGIN;
        let saved = layout::load();
        let mut layout = HashMap::new();
        for name in &desktop.widgets {
            let (content, min, default, resize, click): (gtk::Widget, (i32, i32), (i32, i32), Box<dyn Fn(i32, i32)>, Option<Box<dyn Fn()>>) =
                match name.as_str() {
                    "weather" => {
                        let w = weather::Weather::new(s);
                        let root = w.root.clone().upcast();
                        (
                            root,
                            (300, 220),
                            (380, 270),
                            Box::new(move |width, height| w.resize(width, height)),
                            Some(Box::new(|| crate::exec::spawn("octopus", &["open", "weather"]))),
                        )
                    }
                    "contributions" | "github" if !desktop.github.is_empty() => (
                        contributions::build(&desktop.github, desktop.contributions_colors != "theme").upcast(),
                        (280, 170),
                        (560, 230),
                        Box::new(|_, _| {}),
                        None,
                    ),
                    _ => continue,
                };
            let rect = saved.get(name).copied().unwrap_or_else(|| {
                let rect = Rect { x: screen.0 - default.0 - MARGIN, y: next_y, w: default.0, h: default.1 };
                next_y += default.1 + 16;
                rect
            });
            layout.insert(name.clone(), rect);

            let frame = gtk::Overlay::new();
            frame.add_css_class("desktop-frame");
            content.set_size_request(rect.w, rect.h);
            frame.set_child(Some(&content));
            let handle = Shape::new(Kind::Cookie4, 30, true, Some(&icons::icon("open_in_full")));
            handle.add_css_class("resize-handle");
            handle.set_halign(gtk::Align::End);
            handle.set_valign(gtk::Align::End);
            handle.set_margin_end(6);
            handle.set_margin_bottom(6);
            handle.set_tooltip_text(Some(crate::i18n::t("arrasta pra mudar o tamanho")));
            frame.add_overlay(&handle);
            resize(rect.w, rect.h);
            items.push(Item {
                name: name.clone(),
                frame,
                content,
                handle: handle.upcast(),
                min,
                resize,
                click,
            });
        }
        if items.is_empty() {
            return None;
        }

        let fixed = gtk::Fixed::new();
        for item in &items {
            let rect = layout[&item.name];
            fixed.put(&item.frame, rect.x.max(left_edge) as f64, rect.y as f64);
        }

        let window = gtk::Window::builder().application(app).child(&fixed).build();
        window.add_css_class("octopus-desktop");
        window.init_layer_shell();
        window.set_namespace(Some("octopus-desktop"));
        window.set_layer(Layer::Bottom);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_exclusive_zone(-1);
        for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
            window.set_anchor(edge, true);
        }

        let desktop = Rc::new(Self {
            _window: window.clone(),
            fixed: fixed.clone(),
            items,
            layout: RefCell::new(layout),
            grab: Cell::new(None),
        });
        desktop.gestures(left_edge);
        window.present();
        Some(desktop)
    }

    fn gestures(self: &Rc<Self>, left_edge: i32) {
        let drag = gtk::GestureDrag::new();
        drag.set_button(gtk::gdk::BUTTON_PRIMARY);

        let weak = Rc::downgrade(self);
        drag.connect_drag_begin(move |gesture, x, y| {
            let Some(d) = weak.upgrade() else { return };
            let Some(picked) = d.fixed.pick(x, y, gtk::PickFlags::DEFAULT) else {
                gesture.set_state(gtk::EventSequenceState::Denied);
                return;
            };
            let found = d.items.iter().enumerate().find(|(_, item)| picked.is_ancestor(&item.frame) || picked == item.frame);
            let Some((index, item)) = found else {
                gesture.set_state(gtk::EventSequenceState::Denied);
                return;
            };
            let on_handle = picked == item.handle || picked.is_ancestor(&item.handle);
            let grab = if on_handle {
                Grab::Resize { w: item.content.width(), h: item.content.height() }
            } else {
                let (x, y) = d.fixed.child_position(&item.frame);
                Grab::Move { x, y }
            };
            d.grab.set(Some((index, grab, false)));
        });

        let weak = Rc::downgrade(self);
        drag.connect_drag_update(move |_, dx, dy| {
            let Some(d) = weak.upgrade() else { return };
            let Some((index, grab, mut moved)) = d.grab.get() else { return };
            if !moved && dx.hypot(dy) < THRESHOLD {
                return;
            }
            moved = true;
            d.grab.set(Some((index, grab, moved)));
            let item = &d.items[index];
            let (screen_w, screen_h) = (d.fixed.width() as f64, d.fixed.height() as f64);
            match grab {
                Grab::Move { x, y } => {
                    let (w, h) = (item.frame.width() as f64, item.frame.height() as f64);
                    let x = (x + dx).clamp(left_edge as f64, (screen_w - w).max(left_edge as f64));
                    let y = (y + dy).clamp(0.0, (screen_h - h).max(0.0));
                    d.fixed.move_(&item.frame, x.round(), y.round());
                }
                Grab::Resize { w, h } => {
                    let (x, y) = d.fixed.child_position(&item.frame);
                    let w = (w as f64 + dx).clamp(item.min.0 as f64, screen_w - x) as i32;
                    let h = (h as f64 + dy).clamp(item.min.1 as f64, screen_h - y) as i32;
                    item.content.set_size_request(w, h);
                    (item.resize)(w, h);
                }
            }
        });

        let weak = Rc::downgrade(self);
        drag.connect_drag_end(move |_, _, _| {
            let Some(d) = weak.upgrade() else { return };
            let Some((index, grab, moved)) = d.grab.take() else { return };
            let item = &d.items[index];
            if !moved {
                if matches!(grab, Grab::Move { .. }) {
                    if let Some(click) = &item.click {
                        click();
                    }
                }
                return;
            }
            let (x, y) = d.fixed.child_position(&item.frame);
            let (w, h) = item.content.size_request();
            let mut layout = d.layout.borrow_mut();
            layout.insert(item.name.clone(), Rect { x: x as i32, y: y as i32, w, h });
            layout::save(&layout);
        });
        self.fixed.add_controller(drag);
    }
}
