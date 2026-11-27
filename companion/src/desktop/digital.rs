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
    twelve: Cell<bool>,
}

fn arm(face: &Rc<Digital>, timer: &Rc<Cell<Option<glib::SourceId>>>) {
    if let Some(id) = timer.take() {
        id.remove();
    }
    let wait = 60 - chrono::Local::now().second() as u64;
    let (weak, t) = (Rc::downgrade(face), timer.clone());
    timer.set(Some(glib::timeout_add_local_once(Duration::from_millis(wait * 1000 + 50), move || {
        t.set(None);
        if let Some(face) = weak.upgrade() {
            face.show_time();
            arm(&face, &t);
        }
    })));
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
        let face = Rc::new(Self { shape, hour, minute, twelve: Cell::new(twelve) });
        face.resize(size);

        face.show_time();
        // wakes once a minute, on the minute, and only while on screen
        let timer: Rc<Cell<Option<glib::SourceId>>> = Rc::default();
        let weak = Rc::downgrade(&face);
        let t = timer.clone();
        face.shape.connect_map(move |_| {
            if let Some(face) = weak.upgrade() {
                face.show_time();
                arm(&face, &t);
            }
        });
        face.shape.connect_unmap(move |_| {
            if let Some(id) = timer.take() {
                id.remove();
            }
        });
        face
    }

    fn show_time(&self) {
        let now = chrono::Local::now();
        let h = if self.twelve.get() {
            match now.hour() % 12 {
                0 => 12,
                h => h,
            }
        } else {
            now.hour()
        };
        self.hour.set_label(&format!("{h:02}"));
        self.minute.set_label(&format!("{:02}", now.minute()));
    }

    pub fn resize(&self, size: i32) {
        self.shape.set_size(size);
        let px = (size as f32 * 0.27) as i32;
        sized(&self.hour, px);
        sized(&self.minute, px);
    }
}
