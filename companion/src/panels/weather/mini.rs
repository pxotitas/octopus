//! The weather, small: what the old weather tab showed, in a card.

use std::rc::Rc;

use gtk::prelude::*;

use crate::i18n::{t, tf};
use crate::icons;
use crate::services::weather::compass;
use crate::services::Services;
use crate::widgets::shape::{Kind, Shape};

fn label(text: &str, class: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class(class);
    label
}

/// `hours` is how many of the next hours to show.
pub fn card(s: &Rc<Services>, hours: usize) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.add_css_class("weather-mini");

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let glyph = icons::icon("cloud");
    let badge = Shape::new(Kind::Cookie9, 56, true, Some(&glyph));
    badge.add_css_class("mini-badge");
    top.append(&badge);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 0);
    text.set_valign(gtk::Align::Center);
    text.set_hexpand(true);
    let temp = label("--°", "mini-temp");
    temp.set_xalign(0.0);
    let desc = label("", "dim");
    desc.set_xalign(0.0);
    desc.set_ellipsize(gtk::pango::EllipsizeMode::End);
    desc.set_max_width_chars(1);
    text.append(&temp);
    text.append(&desc);
    top.append(&text);
    let range = label("", "mini-range");
    range.set_valign(gtk::Align::Center);
    top.append(&range);
    root.append(&top);

    let readings = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    readings.set_homogeneous(true);
    root.append(&readings);

    let strip = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    strip.set_homogeneous(true);
    strip.set_vexpand(true);
    strip.set_valign(gtk::Align::End);
    root.append(&strip);

    s.weather.reading.subscribe(move |reading| {
        crate::widgets::clear(&readings);
        crate::widgets::clear(&strip);
        let Some(r) = reading else {
            temp.set_label("--°");
            desc.set_label(t("sem dados do clima"));
            range.set_label("");
            return;
        };
        glyph.set_label(&r.icon);
        temp.set_label(&format!("{}°", r.temperature));
        desc.set_label(&format!("{} · {}", r.description, r.place.to_lowercase()));
        range.set_label(&format!("↑{}°  ↓{}°", r.high, r.low));
        for (icon, value) in [
            ("thermostat", tf("{}°", r.feels_like)),
            ("humidity_percentage", format!("{}%", r.humidity)),
            ("air", format!("{} {}", r.wind, compass(r.wind_dir))),
            ("umbrella", format!("{}%", r.day(0).map_or(0, |d| d.rain))),
        ] {
            let pill = gtk::Box::new(gtk::Orientation::Horizontal, 4);
            pill.add_css_class("mini-pill");
            pill.set_halign(gtk::Align::Fill);
            let glyph = icons::icon(icon);
            pill.append(&glyph);
            pill.append(&gtk::Label::new(Some(&value)));
            readings.append(&pill);
        }
        for (i, h) in r.hourly.iter().step_by(2).take(hours).enumerate() {
            let cell = gtk::Box::new(gtk::Orientation::Vertical, 2);
            cell.add_css_class("mini-hour");
            let hour = if i == 0 { t("agora").to_owned() } else { format!("{:02}h", h.hour) };
            cell.append(&label(&hour, "dim"));
            cell.append(&icons::icon(&h.icon));
            cell.append(&gtk::Label::new(Some(&format!("{}°", h.temperature))));
            strip.append(&cell);
        }
    });
    root
}
