//! The weather on the desktop: now in a big cookie, the next days in small
//! shapes, both growing with the widget.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;
use crate::panels::weather::day_short_name;
use crate::services::Services;
use crate::widgets::shape::{Kind, Shape};

const DAY_SHAPES: [Kind; 4] = [Kind::Clover4, Kind::Puffy, Kind::Cookie6, Kind::Sunny];

pub struct Weather {
    pub root: gtk::Box,
    big: Shape,
    days: gtk::Box,
    day_size: Rc<Cell<i32>>,
    count: i32,
}

impl Weather {
    pub fn new(s: &Rc<Services>, options: &crate::config::DeskWeather) -> Rc<Self> {
        let count = options.days.min(6) as usize;
        let details = options.details;
        let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
        root.add_css_class("desktop-widget");
        root.add_css_class("desktop-weather");

        let top = gtk::Box::new(gtk::Orientation::Horizontal, 18);
        top.set_vexpand(true);
        let inside = gtk::Box::new(gtk::Orientation::Vertical, 0);
        inside.set_halign(gtk::Align::Center);
        inside.set_valign(gtk::Align::Center);
        let glyph = icons::icon("cloud");
        glyph.add_css_class("desktop-weather-icon");
        let temp = gtk::Label::new(Some("--°"));
        temp.add_css_class("desktop-temp");
        inside.append(&glyph);
        inside.append(&temp);
        let big = Shape::new(Kind::Cookie9, 150, true, Some(&inside));
        big.add_css_class("desktop-badge");
        top.append(&big);

        let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
        text.set_valign(gtk::Align::Center);
        let desc = gtk::Label::new(None);
        desc.add_css_class("desktop-desc");
        desc.set_xalign(0.0);
        let place = gtk::Label::new(None);
        place.add_css_class("dim");
        place.set_xalign(0.0);
        let range = gtk::Label::new(None);
        range.add_css_class("desktop-range");
        range.set_xalign(0.0);
        let feels = gtk::Label::new(None);
        feels.add_css_class("dim");
        feels.set_xalign(0.0);
        text.append(&desc);
        text.append(&place);
        text.append(&range);
        text.append(&feels);
        top.append(&text);
        root.append(&top);

        let days = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        days.set_homogeneous(true);
        days.set_visible(count > 0);
        root.append(&days);
        place.set_visible(details);
        feels.set_visible(details);

        let widget = Rc::new(Self {
            root,
            big,
            days: days.clone(),
            day_size: Rc::new(Cell::new(74)),
            count: count.max(1) as i32,
        });

        let day_size = widget.day_size.clone();
        s.weather.reading.subscribe(move |reading| {
            crate::widgets::clear(&days);
            let Some(r) = reading else {
                temp.set_label("--°");
                desc.set_label(t("sem dados do clima"));
                return;
            };
            glyph.set_label(&r.icon);
            temp.set_label(&format!("{}°", r.temperature));
            desc.set_label(&r.description);
            place.set_label(&r.place.to_lowercase());
            range.set_label(&format!("↑{}°  ↓{}°", r.high, r.low));
            feels.set_label(&crate::i18n::tf("sensação {}°", r.feels_like));
            for (i, d) in r.days.iter().filter(|d| d.offset >= 1).take(count).enumerate() {
                let cell = gtk::Box::new(gtk::Orientation::Vertical, 0);
                cell.set_halign(gtk::Align::Center);
                cell.set_valign(gtk::Align::Center);
                let name = gtk::Label::new(Some(&day_short_name(d)));
                name.add_css_class("desktop-day");
                cell.append(&name);
                cell.append(&icons::icon(&d.icon));
                cell.append(&gtk::Label::new(Some(&format!("{}°", d.high))));
                let shape = Shape::new(DAY_SHAPES[i % DAY_SHAPES.len()], day_size.get(), true, Some(&cell));
                shape.add_css_class("desktop-day-shape");
                days.append(&shape);
            }
        });
        widget
    }

    /// The cookie and the days follow the widget's size.
    pub fn resize(&self, w: i32, h: i32) {
        self.big.set_size((h as f32 * 0.5).clamp(100.0, 280.0) as i32);
        let day = ((w - 36 - 6 * (self.count - 1)) / self.count).clamp(44, 130);
        self.day_size.set(day);
        let mut child = self.days.first_child();
        while let Some(widget) = child {
            if let Some(shape) = widget.downcast_ref::<Shape>() {
                shape.set_size(day);
            }
            child = widget.next_sibling();
        }
    }
}
