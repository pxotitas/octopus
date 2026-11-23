use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

use crate::icons;
use crate::widgets::shape::{Kind, Shape};

/// Session actions in the middle of the screen. The ones that end the
/// session ask for a second press.
pub fn build(run: Rc<dyn Fn(&str)>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
    root.add_css_class("surface");
    root.add_css_class("power");

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.set_homogeneous(true);
    let hint = gtk::Label::new(Some("energia"));
    hint.add_css_class("power-hint");

    let armed: Rc<RefCell<Option<(gtk::Button, glib::SourceId)>>> = Rc::default();
    let actions = [
        ("lock", "lock", "bloquear", Kind::Cookie9, false),
        ("suspend", "bedtime", "suspender", Kind::Clover4, false),
        ("logout", "logout", "sair", Kind::Cookie6, true),
        ("reboot", "restart_alt", "reiniciar", Kind::Sunny, true),
        (
            "poweroff",
            "power_settings_new",
            "desligar",
            Kind::Cookie9,
            true,
        ),
    ];
    for (action, glyph, label, kind, confirm) in actions {
        let button = gtk::Button::new();
        button.add_css_class("power-action");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let shape = Shape::new(kind, 64, true, Some(&icons::icon(glyph)));
        content.append(&shape);
        let name = gtk::Label::new(Some(label));
        content.append(&name);
        button.set_child(Some(&content));

        let (run, armed, hint) = (run.clone(), armed.clone(), hint.clone());
        button.connect_clicked(move |button| {
            let again = armed.borrow().as_ref().is_some_and(|(b, _)| b == button);
            if let Some((b, timer)) = armed.borrow_mut().take() {
                timer.remove();
                b.remove_css_class("armed");
            }
            hint.set_label("energia");
            if !confirm || again {
                run(action);
                return;
            }
            button.add_css_class("armed");
            hint.set_label(&format!("aperte de novo pra {label}"));
            let (slot, b, hint) = (armed.clone(), button.clone(), hint.clone());
            let timer = glib::timeout_add_local_once(Duration::from_secs(3), move || {
                slot.borrow_mut().take();
                b.remove_css_class("armed");
                hint.set_label("energia");
            });
            *armed.borrow_mut() = Some((button.clone(), timer));
        });
        row.append(&button);
    }
    root.append(&hint);
    root.append(&row);
    root
}
