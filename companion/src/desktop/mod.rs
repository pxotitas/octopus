//! Widgets on the desktop: a layer under the windows, so they show where no
//! window covers them.

mod github;
mod weather;

use std::rc::Rc;

use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::config;
use crate::services::Services;

pub struct Desktop {
    _window: gtk::Window,
}

impl Desktop {
    pub fn new(app: &gtk::Application, s: &Rc<Services>, config: &config::Config) -> Option<Rc<Self>> {
        let desktop = &config.desktop;
        if !desktop.enabled || desktop.widgets.is_empty() {
            return None;
        }

        let column = gtk::Box::new(gtk::Orientation::Vertical, 16);
        column.add_css_class("desktop-widgets");
        for name in &desktop.widgets {
            match name.as_str() {
                "weather" => column.append(&weather::build(s)),
                "github" if !desktop.github.is_empty() => column.append(&github::build(&desktop.github)),
                _ => {}
            }
        }
        column.first_child()?;

        let window = gtk::Window::builder().application(app).child(&column).build();
        window.add_css_class("octopus-desktop");
        window.init_layer_shell();
        window.set_namespace(Some("octopus-desktop"));
        window.set_layer(Layer::Bottom);
        window.set_keyboard_mode(KeyboardMode::None);

        let (vertical, horizontal) = desktop.position.split_once('-').unwrap_or(("top", "right"));
        let vertical = if vertical == "bottom" { Edge::Bottom } else { Edge::Top };
        let horizontal = if horizontal == "left" { Edge::Left } else { Edge::Right };
        window.set_anchor(vertical, true);
        window.set_anchor(horizontal, true);
        window.set_margin(vertical, 28);
        // Clear of the bar when it sits on the same side.
        let bar_side = if config.bar.right() { Edge::Right } else { Edge::Left };
        let margin = if horizontal == bar_side { config.bar.width as i32 + 40 } else { 28 };
        window.set_margin(horizontal, margin);
        window.present();

        Some(Rc::new(Self { _window: window }))
    }
}
