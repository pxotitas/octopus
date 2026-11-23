use std::cell::RefCell;
use std::rc::Rc;

use crate::store::Listeners;
use gtk::prelude::*;
use gtk::{gdk, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

/// The bar sits on the left, so its menus open beside it; anchoring the left
/// edge only (no margin) puts them right after the bar's exclusive zone.
#[derive(Clone, Copy)]
pub enum Place {
    /// Beside the bottom of the bar, where the control group lives.
    BarBottom,
    /// Beside the middle of the bar, where the clock lives.
    BarMiddle,
    BottomCenter,
}

const GAP: i32 = 8;

pub struct Popup {
    pub window: gtk::Window,
    revealer: gtk::Revealer,
    listeners: Listeners<bool>,
}

impl Popup {
    pub fn new(
        app: &gtk::Application,
        name: &str,
        place: Place,
        interactive: bool,
        child: &impl IsA<gtk::Widget>,
    ) -> Rc<Self> {
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .build();
        window.add_css_class("octopus-popup");
        window.init_layer_shell();
        window.set_namespace(Some(&format!("octopus-{name}")));
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(if interactive {
            KeyboardMode::Exclusive
        } else {
            KeyboardMode::None
        });

        let transition = match place {
            Place::BarBottom | Place::BarMiddle => {
                window.set_anchor(Edge::Left, true);
                window.set_margin(Edge::Left, GAP);
                if matches!(place, Place::BarBottom) {
                    window.set_anchor(Edge::Bottom, true);
                    window.set_margin(Edge::Bottom, GAP);
                }
                gtk::RevealerTransitionType::SlideRight
            }
            Place::BottomCenter => {
                window.set_anchor(Edge::Bottom, true);
                window.set_margin(Edge::Bottom, GAP * 6);
                gtk::RevealerTransitionType::SlideUp
            }
        };

        let revealer = gtk::Revealer::builder()
            .transition_type(transition)
            .transition_duration(240)
            .child(child)
            .build();
        window.set_child(Some(&revealer));

        let popup = Rc::new(Self {
            window,
            revealer,
            listeners: RefCell::new(Vec::new()),
        });

        let weak = Rc::downgrade(&popup);
        popup
            .revealer
            .connect_child_revealed_notify(move |revealer| {
                if let Some(popup) = weak.upgrade() {
                    if !revealer.is_child_revealed() && !revealer.reveals_child() {
                        popup.window.set_visible(false);
                    }
                }
            });

        if interactive {
            let keys = gtk::EventControllerKey::new();
            let weak = Rc::downgrade(&popup);
            keys.connect_key_pressed(move |_, key, _, _| {
                if key == gdk::Key::Escape {
                    if let Some(popup) = weak.upgrade() {
                        popup.hide();
                    }
                    return glib::Propagation::Stop;
                }
                glib::Propagation::Proceed
            });
            popup.window.add_controller(keys);
        }

        popup
    }

    pub fn is_open(&self) -> bool {
        self.revealer.reveals_child()
    }

    pub fn show(&self) {
        if self.is_open() {
            return;
        }
        self.window.present();
        let revealer = self.revealer.clone();
        glib::idle_add_local_once(move || revealer.set_reveal_child(true));
        self.emit(true);
    }

    pub fn hide(&self) {
        if !self.is_open() {
            return;
        }
        self.revealer.set_reveal_child(false);
        self.emit(false);
    }

    pub fn on_visibility(&self, listener: impl Fn(bool) + 'static) {
        self.listeners.borrow_mut().push(Box::new(listener));
    }

    fn emit(&self, open: bool) {
        for listener in self.listeners.borrow().iter() {
            listener(open);
        }
    }
}

/// An invisible full-screen layer under the menus that closes them on a
/// click outside. It keeps the bar's exclusive zone, so the bar stays
/// clickable and can switch straight to another menu.
pub fn backdrop(app: &gtk::Application, on_click: impl Fn() + 'static) -> gtk::Window {
    let window = gtk::Window::builder()
        .application(app)
        .decorated(false)
        .build();
    window.add_css_class("octopus-backdrop");
    window.init_layer_shell();
    window.set_namespace(Some("octopus-backdrop"));
    window.set_layer(Layer::Top);
    window.set_keyboard_mode(KeyboardMode::None);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
    let click = gtk::GestureClick::new();
    click.set_button(0);
    click.connect_pressed(move |_, _, _, _| on_click());
    window.add_controller(click);
    window
}
