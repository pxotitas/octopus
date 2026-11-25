use crate::i18n::t;
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
    let hint = gtk::Label::new(Some(t("energia")));
    hint.add_css_class("power-hint");

    let armed: Rc<RefCell<Option<(gtk::Button, glib::SourceId)>>> = Rc::default();
    let actions = [
        ("lock", "lock", t("bloquear"), Kind::Cookie9, Kind::Flower, false),
        ("suspend", "bedtime", t("suspender"), Kind::Clover4, Kind::Puffy, false),
        ("logout", "logout", t("sair"), Kind::Cookie6, Kind::SoftBurst, true),
        ("reboot", "restart_alt", t("reiniciar"), Kind::Sunny, Kind::Clover8, true),
        (
            "poweroff",
            "power_settings_new",
            t("desligar"),
            Kind::Cookie12,
            Kind::Burst,
            true,
        ),
    ];
    for (action, glyph, label, kind, hover, confirm) in actions {
        let button = gtk::Button::new();
        button.add_css_class("power-action");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let shape = Shape::new(kind, 64, true, Some(&icons::icon(glyph)));
        shape.morph_on_hover(&button, hover);
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
            hint.set_label(t("energia"));
            if !confirm || again {
                run(action);
                return;
            }
            button.add_css_class("armed");
            hint.set_label(&crate::i18n::tf("aperte de novo pra {}", label));
            let (slot, b, hint) = (armed.clone(), button.clone(), hint.clone());
            let timer = glib::timeout_add_local_once(Duration::from_secs(3), move || {
                slot.borrow_mut().take();
                b.remove_css_class("armed");
                hint.set_label(t("energia"));
            });
            *armed.borrow_mut() = Some((button.clone(), timer));
        });
        row.append(&button);
    }
    root.append(&hint);
    root.append(&row);
    root
}
