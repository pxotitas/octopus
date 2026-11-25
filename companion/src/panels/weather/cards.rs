//! The weather panel's pieces, built from a reading.

use chrono::{Local, NaiveDate, Timelike};
use gtk::prelude::*;

use super::sky::{self, Figure, Sky};
use crate::i18n::{t, tf};
use crate::icons;
use crate::services::weather::{compass, Day, Hour, Reading};
use crate::widgets::shape::{Kind, Shape};
use crate::widgets::shapebox::{Outline, ShapeBox};

pub fn date(day: &Day) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").ok()
}

/// "ontem", "hoje", "amanhã", or the weekday.
pub fn name(day: &Day) -> String {
    match day.offset {
        -1 => t("ontem").to_owned(),
        0 => t("hoje").to_owned(),
        1 => t("amanhã").to_owned(),
        _ => date(day)
            .map(|d| d.format_localized("%A", crate::i18n::locale()).to_string().to_lowercase())
            .unwrap_or_default(),
    }
}

pub fn short_name(day: &Day) -> String {
    match day.offset {
        0 => t("hoje").to_owned(),
        _ => date(day)
            .map(|d| d.format_localized("%a", crate::i18n::locale()).to_string().to_lowercase())
            .unwrap_or_default(),
    }
}

/// "27 de novembro"
pub fn long_date(day: &Day) -> String {
    date(day)
        .map(|d| d.format_localized(t("%-d de %B"), crate::i18n::locale()).to_string())
        .unwrap_or_default()
}

pub fn uv_level(uv: f64) -> &'static str {
    match uv {
        u if u < 3.0 => t("baixo"),
        u if u < 6.0 => t("moderado"),
        u if u < 8.0 => t("alto"),
        u if u < 11.0 => t("muito alto"),
        _ => t("extremo"),
    }
}

fn label(text: &str, class: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class(class);
    label
}

/// A value in an M3 shape: an icon, the value, what it is.
pub fn metric(kind: Kind, tone: &str, icon: &str, value: &str, caption: &str, size: i32) -> Shape {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.set_halign(gtk::Align::Center);
    content.set_valign(gtk::Align::Center);
    content.append(&icons::icon(icon));
    let value = label(value, "metric-value");
    value.set_justify(gtk::Justification::Center);
    content.append(&value);
    let caption = label(caption, "metric-caption");
    caption.set_justify(gtk::Justification::Center);
    caption.set_wrap(true);
    caption.set_max_width_chars(10);
    content.append(&caption);
    let shape = Shape::new(kind, size, true, Some(&content));
    shape.add_css_class("metric");
    shape.add_css_class(tone);
    shape
}

/// A grid of metric shapes, `columns` wide.
pub fn metrics(items: Vec<Shape>, columns: i32) -> gtk::Grid {
    let grid = gtk::Grid::new();
    grid.add_css_class("metrics");
    grid.set_row_spacing(8);
    grid.set_column_spacing(8);
    grid.set_halign(gtk::Align::Center);
    for (i, item) in items.into_iter().enumerate() {
        grid.attach(&item, i as i32 % columns, i as i32 / columns, 1, 1);
    }
    grid
}

/// The current conditions, big.
pub fn hero(r: &Reading) -> ShapeBox {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let badge = Shape::new(Kind::Cookie9, 104, true, Some(&icons::icon(&r.icon)));
    badge.add_css_class("hero-badge");
    row.append(&badge);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 0);
    text.set_valign(gtk::Align::Center);
    text.set_hexpand(true);
    let temp = label(&format!("{}°", r.temperature), "hero-temp");
    temp.set_xalign(0.0);
    let desc = label(&r.description, "hero-desc");
    desc.set_xalign(0.0);
    let place = if r.region.is_empty() {
        r.place.clone()
    } else {
        format!("{}, {}", r.place, r.region.to_lowercase())
    };
    let place = label(&place.to_lowercase(), "dim");
    place.set_xalign(0.0);
    text.append(&temp);
    text.append(&desc);
    text.append(&place);
    row.append(&text);

    let side = gtk::Box::new(gtk::Orientation::Vertical, 6);
    side.set_valign(gtk::Align::Center);
    for (icon, text) in [
        ("arrow_upward", format!("{}°", r.high)),
        ("arrow_downward", format!("{}°", r.low)),
        ("thermostat", tf("sensação {}°", r.feels_like)),
    ] {
        let line = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        line.append(&icons::icon(icon));
        line.append(&gtk::Label::new(Some(&text)));
        side.append(&line);
    }
    let updated = label(&tf("atualizado às {}", &r.updated), "dim");
    updated.set_xalign(0.0);
    side.append(&updated);
    row.append(&side);

    let card = ShapeBox::new(Outline::Scallop, 22, &row);
    card.add_css_class("weather-hero");
    card
}

/// The sun's arc and the moon, for today.
pub fn sky_card(r: &Reading) -> ShapeBox {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 6);
    column.set_valign(gtk::Align::Center);
    let now = Local::now();
    let minute = (now.hour() * 60 + now.minute()) as i32;
    let progress = match (sky::minutes(&r.sunrise), sky::minutes(&r.sunset)) {
        (Some(rise), Some(set)) if set > rise => (minute - rise) as f32 / (set - rise) as f32,
        _ => -1.0,
    };
    let arc = Sky::new(Figure::Sun { progress }, 200, 92);
    arc.add_css_class("sun-arc");
    arc.set_halign(gtk::Align::Center);
    column.append(&arc);

    let times = gtk::CenterBox::new();
    times.set_start_widget(Some(&time_label("wb_twilight", &r.sunrise)));
    times.set_end_widget(Some(&time_label("bedtime", &r.sunset)));
    column.append(&times);

    let moon_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    moon_row.set_halign(gtk::Align::Center);
    moon_row.set_margin_top(6);
    let moon = Sky::new(Figure::Moon { age: r.moon as f32 }, 34, 34);
    moon.add_css_class("moon");
    moon_row.append(&moon);
    moon_row.append(&label(&r.moon_name, "moon-name"));
    column.append(&moon_row);

    let card = ShapeBox::new(Outline::Arch, 16, &column);
    card.add_css_class("sky-card");
    card
}

fn time_label(icon: &str, time: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    row.append(&icons::icon(icon));
    row.append(&gtk::Label::new(Some(time)));
    row
}

/// The metrics for right now.
pub fn now_metrics(r: &Reading) -> gtk::Grid {
    let rain = r.day(0).map_or(0, |d| d.rain);
    let (air_value, air_caption) = match &r.air {
        Some(a) => (a.index.to_string(), format!("{} · {}", t("ar"), a.label)),
        None => ("--".into(), t("ar").to_owned()),
    };
    metrics(
        vec![
            metric(Kind::Puffy, "tone-a", "humidity_percentage", &format!("{}%", r.humidity), t("umidade"), 104),
            metric(Kind::Clover4, "tone-b", "air", &format!("{} km/h", r.wind), &format!("{} · {}", t("vento"), compass(r.wind_dir)), 104),
            metric(Kind::Sunny, "tone-c", "sunny", &format!("{}", r.uv), &format!("uv · {}", uv_level(r.uv)), 104),
            metric(Kind::Flower, "tone-a", "umbrella", &format!("{rain}%"), t("chuva hoje"), 104),
            metric(Kind::Cookie12, "tone-b", "compress", &format!("{}", r.pressure), "hPa", 104),
            metric(Kind::Pentagon, "tone-c", "visibility", &format!("{} km", r.visibility), t("visibilidade"), 104),
            metric(Kind::SoftBurst, "tone-a", "cloud", &format!("{}%", r.cloud), t("nuvens"), 104),
            metric(Kind::Cookie7, "tone-b", "eco", &air_value, &air_caption, 104),
        ],
        4,
    )
}

/// One day's metrics.
pub fn day_metrics(d: &Day) -> gtk::Grid {
    let light = match (sky::minutes(&d.sunrise), sky::minutes(&d.sunset)) {
        (Some(rise), Some(set)) if set > rise => format!("{}h{:02}", (set - rise) / 60, (set - rise) % 60),
        _ => "--".into(),
    };
    metrics(
        vec![
            metric(Kind::Flower, "tone-a", "umbrella", &format!("{}%", d.rain), t("chance de chuva"), 104),
            metric(Kind::Puffy, "tone-b", "water_drop", &format!("{} mm", d.rain_mm), t("de chuva"), 104),
            metric(Kind::Clover4, "tone-c", "air", &format!("{} km/h", d.wind), &tf("rajadas {}", d.gusts), 104),
            metric(Kind::Sunny, "tone-a", "sunny", &format!("{}", d.uv), &format!("uv · {}", uv_level(d.uv)), 104),
            metric(Kind::Cookie6, "tone-b", "wb_twilight", &d.sunrise, t("nascer do sol"), 104),
            metric(Kind::Cookie7, "tone-c", "bedtime", &d.sunset, t("pôr do sol"), 104),
            metric(Kind::Pentagon, "tone-a", "light_mode", &light, t("de luz"), 104),
            metric(Kind::Circle, "tone-b", "dark_mode", &format!("{:.0}%", sky::lit(d.moon) * 100.0), &d.moon_name, 104),
        ],
        4,
    )
}

/// A row of hours, scrolled sideways.
pub fn hours(list: &[Hour], highlight_first: bool) -> gtk::ScrolledWindow {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    for (i, h) in list.iter().enumerate() {
        let cell = gtk::Box::new(gtk::Orientation::Vertical, 4);
        cell.add_css_class("hour");
        if highlight_first && i == 0 {
            cell.add_css_class("now");
        }
        let hour = if highlight_first && i == 0 { t("agora").to_owned() } else { format!("{:02}h", h.hour) };
        cell.append(&label(&hour, "dim"));
        cell.append(&icons::icon(&h.icon));
        cell.append(&gtk::Label::new(Some(&format!("{}°", h.temperature))));
        let rain = label(&format!("{}%", h.rain), "rain");
        rain.set_visible(h.rain > 0);
        cell.append(&rain);
        row.append(&cell);
    }
    let scroller = gtk::ScrolledWindow::builder()
        .vscrollbar_policy(gtk::PolicyType::Never)
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&row)
        .build();
    scroller.add_css_class("hours");
    scroller
}

/// A titled card.
pub fn section(title: &str, child: &impl IsA<gtk::Widget>, outline: Outline) -> ShapeBox {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 10);
    column.append(&crate::widgets::heading(title, None));
    column.append(child);
    let card = ShapeBox::new(outline, 16, &column);
    card.add_css_class("weather-section");
    card
}

/// Every day as a row: name, icon, chance of rain, low, a bar, high.
pub fn days(r: &Reading, open: impl Fn(i32) + Clone + 'static) -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let shown: Vec<&Day> = r.days.iter().filter(|d| d.offset >= -1).collect();
    let low = shown.iter().map(|d| d.low).min().unwrap_or(0) as f32;
    let high = shown.iter().map(|d| d.high).max().unwrap_or(1) as f32;
    let span = (high - low).max(1.0);
    for d in shown {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let name = label(&name(d), "day-name");
        name.set_xalign(0.0);
        name.set_width_chars(13);
        name.set_max_width_chars(13);
        name.set_ellipsize(gtk::pango::EllipsizeMode::End);
        row.append(&name);
        row.append(&icons::icon(&d.icon));
        let rain = label(&format!("{}%", d.rain), "rain");
        rain.set_width_chars(4);
        rain.set_opacity(if d.rain >= 20 { 1.0 } else { 0.0 });
        row.append(&rain);
        let lo = label(&format!("{}°", d.low), "dim");
        lo.set_width_chars(4);
        lo.set_xalign(1.0);
        row.append(&lo);
        let now = (d.offset == 0).then(|| (r.temperature as f32 - low) / span);
        let bar = Sky::new(
            Figure::Range { low: (d.low as f32 - low) / span, high: (d.high as f32 - low) / span, now },
            140,
            8,
        );
        bar.add_css_class("range");
        bar.set_hexpand(true);
        row.append(&bar);
        let hi = label(&format!("{}°", d.high), "day-high");
        hi.set_width_chars(4);
        row.append(&hi);

        let button = gtk::Button::new();
        button.add_css_class("day-row");
        if d.offset < 0 {
            button.add_css_class("past");
        }
        if d.offset == 0 {
            button.add_css_class("today");
        }
        button.set_child(Some(&row));
        let (open, offset) = (open.clone(), d.offset);
        button.connect_clicked(move |_| open(offset));
        list.append(&button);
    }
    list
}

const WEEK_SHAPES: [Kind; 7] = [
    Kind::Cookie9,
    Kind::Clover4,
    Kind::Puffy,
    Kind::Sunny,
    Kind::Cookie6,
    Kind::Flower,
    Kind::Cookie12,
];

/// Days from `start`, each in its own shape.
pub fn week(r: &Reading, start: i32, count: i32, open: impl Fn(i32) + Clone + 'static) -> gtk::FlowBox {
    let flow = gtk::FlowBox::new();
    flow.add_css_class("week");
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_homogeneous(true);
    flow.set_max_children_per_line(4);
    flow.set_min_children_per_line(4);
    flow.set_row_spacing(10);
    flow.set_column_spacing(10);
    for (i, offset) in (start..start + count).enumerate() {
        let Some(d) = r.day(offset) else { continue };
        let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
        content.set_halign(gtk::Align::Center);
        content.set_valign(gtk::Align::Center);
        content.append(&label(&short_name(d), "week-day"));
        let icon = icons::icon(&d.icon);
        icon.add_css_class("week-icon");
        content.append(&icon);
        content.append(&label(&format!("{}° · {}°", d.high, d.low), "week-temp"));
        let rain = label(&format!("{} {}%", t("chuva"), d.rain), "metric-caption");
        rain.set_visible(d.rain >= 10);
        content.append(&rain);
        let shape = Shape::new(WEEK_SHAPES[i % WEEK_SHAPES.len()], 132, true, Some(&content));
        shape.add_css_class("metric");
        shape.add_css_class(["tone-a", "tone-b", "tone-c"][i % 3]);
        let button = gtk::Button::new();
        button.add_css_class("shape-button");
        button.set_child(Some(&shape));
        button.set_tooltip_text(Some(&format!("{} · {}", name(d), d.description)));
        let open = open.clone();
        button.connect_clicked(move |_| open(offset));
        flow.insert(&button, -1);
    }
    flow
}

/// One day, big: its icon, high and low, and how it compares to today.
pub fn day_hero(r: &Reading, d: &Day) -> ShapeBox {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    let badge = Shape::new(Kind::Sunny, 100, true, Some(&icons::icon(&d.icon)));
    badge.add_css_class("hero-badge");
    row.append(&badge);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_valign(gtk::Align::Center);
    let temps = label(&format!("{}° / {}°", d.high, d.low), "hero-temp");
    temps.set_xalign(0.0);
    let desc = label(&d.description, "hero-desc");
    desc.set_xalign(0.0);
    text.append(&temps);
    text.append(&desc);
    if let Some(today) = r.day(0).filter(|_| d.offset != 0) {
        let diff = d.high - today.high;
        let words = match diff.signum() {
            1 => tf("{}° mais quente que hoje", diff),
            -1 => tf("{}° mais fresco que hoje", -diff),
            _ => t("a mesma máxima de hoje").to_owned(),
        };
        let compare = label(&words, "dim");
        compare.set_xalign(0.0);
        text.append(&compare);
    }
    row.append(&text);
    let card = ShapeBox::new(Outline::Ghost, 22, &row);
    card.add_css_class("day-hero");
    card
}
