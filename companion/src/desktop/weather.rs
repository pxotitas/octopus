//! The weather on the desktop: now in a big cookie, the next days in small shapes.

use std::rc::Rc;

use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;
use crate::panels::weather::day_short_name;
use crate::services::Services;
use crate::widgets::shape::{Kind, Shape};

const DAY_SHAPES: [Kind; 4] = [Kind::Clover4, Kind::Puffy, Kind::Cookie6, Kind::Sunny];

pub fn build(s: &Rc<Services>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-weather");

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 18);
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
    root.append(&days);

    let click = gtk::GestureClick::new();
    click.connect_released(|_, _, _, _| crate::exec::spawn("octopus", &["open", "weather"]));
    root.add_controller(click);
    root.set_tooltip_text(Some(t("abrir o clima")));

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
        for (i, d) in r.days.iter().filter(|d| d.offset >= 1).take(4).enumerate() {
            let cell = gtk::Box::new(gtk::Orientation::Vertical, 0);
            cell.set_halign(gtk::Align::Center);
            cell.set_valign(gtk::Align::Center);
            let name = gtk::Label::new(Some(&day_short_name(d)));
            name.add_css_class("desktop-day");
            cell.append(&name);
            cell.append(&icons::icon(&d.icon));
            cell.append(&gtk::Label::new(Some(&format!("{}°", d.high))));
            let shape = Shape::new(DAY_SHAPES[i % DAY_SHAPES.len()], 74, true, Some(&cell));
            shape.add_css_class("desktop-day-shape");
            days.append(&shape);
        }
    });
    root
}
