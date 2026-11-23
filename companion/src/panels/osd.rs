use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

use crate::icons;
use crate::popup::{Place, Popup};

/// The volume / brightness pill shown for media keys.
pub struct Osd {
    popup: Rc<Popup>,
    icon: gtk::Label,
    level: gtk::LevelBar,
    value: gtk::Label,
    hide_at: RefCell<Option<glib::SourceId>>,
}

impl Osd {
    pub fn new(app: &gtk::Application) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        root.add_css_class("surface");
        root.add_css_class("osd");
        let icon = icons::icon("volume_up");
        let level = gtk::LevelBar::for_interval(0.0, 100.0);
        level.set_hexpand(true);
        level.set_valign(gtk::Align::Center);
        let value = gtk::Label::new(None);
        value.add_css_class("percent");
        root.append(&icon);
        root.append(&level);
        root.append(&value);
        let popup = Popup::new(app, "osd", Place::BottomCenter, false, &root);
        Rc::new(Self {
            popup,
            icon,
            level,
            value,
            hide_at: RefCell::new(None),
        })
    }

    pub fn show(self: &Rc<Self>, icon: &str, percent: u32) {
        self.icon.set_label(icon);
        self.level.set_value(percent.min(100) as f64);
        self.value.set_label(&format!("{percent}"));
        self.popup.show();
        if let Some(id) = self.hide_at.borrow_mut().take() {
            id.remove();
        }
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(Duration::from_millis(1400), move || {
            if let Some(osd) = weak.upgrade() {
                osd.hide_at.borrow_mut().take();
                osd.popup.hide();
            }
        });
        *self.hide_at.borrow_mut() = Some(id);
    }
}
