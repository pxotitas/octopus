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
    level: gtk::DrawingArea,
    fraction: Rc<std::cell::Cell<f64>>,
    value: gtk::Label,
    hide_at: RefCell<Option<glib::SourceId>>,
}

impl Osd {
    pub fn new(app: &gtk::Application) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        root.add_css_class("surface");
        root.add_css_class("osd");
        let icon = icons::icon("volume_up");
        let fraction = Rc::new(std::cell::Cell::new(0.0));
        let level = gtk::DrawingArea::new();
        level.add_css_class("osd-level");
        level.set_hexpand(true);
        level.set_content_height(10);
        level.set_valign(gtk::Align::Center);
        {
            let fraction = fraction.clone();
            level.set_draw_func(move |area, cr, w, h| {
                let c = area.color();
                let (w, h) = (w as f64, h as f64);
                let r = h / 2.0;
                let pill = |x: f64, width: f64| {
                    cr.new_sub_path();
                    cr.arc(x + width - r, r, r, -std::f64::consts::FRAC_PI_2, std::f64::consts::FRAC_PI_2);
                    cr.arc(x + r, r, r, std::f64::consts::FRAC_PI_2, 3.0 * std::f64::consts::FRAC_PI_2);
                    cr.close_path();
                };
                cr.set_source_rgba(c.red() as f64, c.green() as f64, c.blue() as f64, 0.2);
                pill(0.0, w);
                let _ = cr.fill();
                let filled = (w * fraction.get()).max(h);
                cr.set_source_rgba(c.red() as f64, c.green() as f64, c.blue() as f64, 1.0);
                pill(0.0, filled);
                let _ = cr.fill();
            });
        }
        let value = gtk::Label::new(None);
        value.add_css_class("percent");
        root.append(&icon);
        root.append(&level);
        root.append(&value);
        let popup = Popup::new(app, "osd", Place::BottomEdge, false, &root);
        Rc::new(Self {
            popup,
            icon,
            level,
            fraction,
            value,
            hide_at: RefCell::new(None),
        })
    }

    pub fn show(self: &Rc<Self>, icon: &str, percent: u32) {
        self.icon.set_label(icon);
        self.fraction.set(percent.min(100) as f64 / 100.0);
        self.level.queue_draw();
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
