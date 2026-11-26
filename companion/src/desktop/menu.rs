//! The desktop's right-click menu.

use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::i18n::t;

pub struct Row {
    pub icon: &'static str,
    pub label: &'static str,
    pub danger: bool,
    pub action: Rc<dyn Fn()>,
}

/// Opens at (x, y) of `parent`, and goes away on its own when closed.
pub fn open(parent: &impl IsA<gtk::Widget>, x: f64, y: f64, rows: Vec<Row>, on_closed: impl Fn() + 'static) {
    let popover = gtk::Popover::new();
    popover.add_css_class("desktop-menu");
    popover.set_has_arrow(false);
    popover.set_position(gtk::PositionType::Bottom);
    popover.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));

    let column = gtk::Box::new(gtk::Orientation::Vertical, 2);
    for row in rows {
        let button = gtk::Button::new();
        button.add_css_class("menu-row");
        if row.danger {
            button.add_css_class("danger");
        }
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        content.append(&crate::icons::icon(row.icon));
        let label = gtk::Label::new(Some(t(row.label)));
        label.set_xalign(0.0);
        content.append(&label);
        button.set_child(Some(&content));
        let (pop, action) = (popover.clone(), row.action.clone());
        button.connect_clicked(move |_| {
            pop.popdown();
            action();
        });
        column.append(&button);
    }
    popover.set_child(Some(&column));
    popover.set_parent(parent);
    popover.connect_closed(move |pop| {
        let pop = pop.clone();
        glib::idle_add_local_once(move || pop.unparent());
        on_closed();
    });
    popover.popup();
}
