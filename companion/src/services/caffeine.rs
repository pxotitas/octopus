//! Keeps the screen awake: a 1×1 layer surface that no one sees or touches,
//! and an idle inhibitor on it. Compositors only honour an inhibitor while
//! its surface is mapped, so the surface lives exactly as long as it's on.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::store::Store;

pub struct Caffeine {
    pub on: Rc<Store<bool>>,
    held: RefCell<Option<(gtk::Window, u32)>>,
}

impl Caffeine {
    pub fn new() -> Rc<Self> {
        Rc::new(Self { on: Store::new(false), held: RefCell::default() })
    }

    pub fn toggle(&self) {
        let on = *self.on.get();
        self.set(!on);
    }

    pub fn set(&self, on: bool) {
        if on == *self.on.get() {
            return;
        }
        let Some(app) = gio::Application::default().and_downcast::<gtk::Application>() else { return };
        if on {
            let window = gtk::Window::builder().application(&app).default_width(1).default_height(1).build();
            window.add_css_class("octopus-caffeine");
            window.init_layer_shell();
            window.set_namespace(Some("octopus-caffeine"));
            window.set_layer(Layer::Background);
            window.set_keyboard_mode(KeyboardMode::None);
            window.set_exclusive_zone(0);
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_size_request(1, 1);
            window.connect_realize(|w| {
                if let Some(surface) = w.surface() {
                    surface.set_input_region(Some(&gtk::cairo::Region::create()));
                }
            });
            window.present();
            let cookie = app.inhibit(Some(&window), gtk::ApplicationInhibitFlags::IDLE, Some("octopus: keep awake"));
            *self.held.borrow_mut() = Some((window, cookie));
        } else if let Some((window, cookie)) = self.held.borrow_mut().take() {
            app.uninhibit(cookie);
            window.destroy();
            glib::idle_add_local_once(crate::memory::release_soon);
        }
        self.on.set(on);
    }
}
