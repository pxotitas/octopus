use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::icons;

/// An icon button beside a 0–100 scale. Programmatic updates never echo back
/// into `on_change`.
pub struct Slider {
    pub root: gtk::Box,
    button: gtk::Button,
    scale: gtk::Scale,
    handler: glib::SignalHandlerId,
}

impl Slider {
    pub fn new(icon: &str, tooltip: &str, on_change: impl Fn(u32) + 'static) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        root.add_css_class("slider-row");
        let button = icons::button(icon, tooltip);
        let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 1.0);
        scale.set_hexpand(true);
        scale.set_draw_value(false);
        let handler =
            scale.connect_value_changed(move |scale| on_change(scale.value().round() as u32));
        root.append(&button);
        root.append(&scale);
        Rc::new(Self {
            root,
            button,
            scale,
            handler,
        })
    }

    pub fn on_icon(&self, f: impl Fn() + 'static) {
        self.button.connect_clicked(move |_| f());
    }

    pub fn set(&self, value: u32, icon: &str) {
        icons::set(&self.button, icon);
        if (self.scale.value() - value as f64).abs() >= 1.0 {
            self.scale.block_signal(&self.handler);
            self.scale.set_value(value as f64);
            self.scale.unblock_signal(&self.handler);
        }
    }

    pub fn set_muted(&self, muted: bool) {
        if muted {
            self.root.add_css_class("muted");
        } else {
            self.root.remove_css_class("muted");
        }
    }
}
