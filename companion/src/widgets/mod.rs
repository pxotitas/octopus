pub mod anim;
pub mod art;
pub mod cava;
pub mod media;
pub mod motion;
pub mod notification;
pub mod shape;
pub mod slider;
pub mod tile;
pub mod wave;

use gtk::prelude::*;

/// A titled section header with an optional trailing widget.
pub fn heading(text: &str, trailing: Option<&gtk::Widget>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("heading");
    let label = gtk::Label::new(Some(text));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    row.append(&label);
    if let Some(widget) = trailing {
        row.append(widget);
    }
    row
}

/// A material-style on/off switch row heading a menu.
pub fn switch_header(title: &str) -> (gtk::Box, gtk::Switch) {
    let switch = gtk::Switch::new();
    switch.set_valign(gtk::Align::Center);
    let row = heading(title, Some(switch.upcast_ref()));
    row.add_css_class("menu-header");
    (row, switch)
}

pub fn clear(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}
