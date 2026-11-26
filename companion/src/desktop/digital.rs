//! The clock widget's digital face: hours over minutes on a cookie, like the
//! bar's clock, growing with the widget.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use chrono::Timelike;
use gtk::glib;
use gtk::pango;
use gtk::prelude::*;

use crate::widgets::shape::{Kind, Shape};

pub struct Digital {
    pub shape: Shape,
    hour: gtk::Label,
    minute: gtk::Label,
}

fn sized(label: &gtk::Label, px: i32) {
    let attrs = pango::AttrList::new();
    attrs.insert(pango::AttrSize::new_size_absolute(px * pango::SCALE));
    label.set_attributes(Some(&attrs));
}

impl Digital {
    pub fn new(size: i32, twelve: bool) -> Rc<Self> {
        let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
        column.set_halign(gtk::Align::Center);
        column.set_valign(gtk::Align::Center);
        let hour = gtk::Label::new(None);
        hour.add_css_class("digital-hour");
        let minute = gtk::Label::new(None);
        minute.add_css_class("digital-minute");
        column.append(&hour);
        column.append(&minute);
        let shape = Shape::new(Kind::Cookie12, size, true, Some(&column));
        shape.add_css_class("digital-face");
        let face = Rc::new(Self { shape, hour, minute });
        face.resize(size);

        let last = Rc::new(Cell::new(u32::MAX));
        let weak = Rc::downgrade(&face);
        let tick = move || {
            let Some(face) = weak.upgrade() else { return false };
            let now = chrono::Local::now();
            if last.replace(now.minute()) != now.minute() {
                let h = if twelve {
                    match now.hour() % 12 {
                        0 => 12,
                        h => h,
                    }
                } else {
                    now.hour()
                };
                face.hour.set_label(&format!("{h:02}"));
                face.minute.set_label(&format!("{:02}", now.minute()));
            }
            true
        };
        tick();
        glib::timeout_add_local(Duration::from_secs(1), move || {
            if tick() { glib::ControlFlow::Continue } else { glib::ControlFlow::Break }
        });
        face
    }

    pub fn resize(&self, size: i32) {
        self.shape.set_size(size);
        let px = (size as f32 * 0.27) as i32;
        sized(&self.hour, px);
        sized(&self.minute, px);
    }
}
