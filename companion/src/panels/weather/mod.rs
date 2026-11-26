//! The dash's weather tab: right now by default, and any day from a week
//! back to two ahead through the `>` line.

mod cards;
pub mod mini;
pub mod query;
mod sky;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::i18n::{t, tf};
use crate::icons;
use crate::services::weather::{Day, Reading};
use crate::services::Services;
use crate::widgets::shapebox::Outline;
use query::Query;

pub use cards::short_name as day_short_name;


pub struct WeatherPanel {
    pub root: gtk::Box,
    entry: gtk::Entry,
    status: gtk::Label,
    content: gtk::Box,
    reading: RefCell<Option<Reading>>,
    query: RefCell<Query>,
}

/// The chips write their own label in the language of the shell; the parser
/// understands both.
const CHIPS: [&str; 6] = ["agora", "ontem", "amanhã", "fim de semana", "semana", "próxima semana"];

impl WeatherPanel {
    pub fn new(s: &Rc<Services>) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
        root.add_css_class("weather-panel");

        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bar.add_css_class("weather-query");
        let prompt = icons::icon("chevron_right");
        prompt.add_css_class("prompt");
        let entry = gtk::Entry::new();
        entry.set_hexpand(true);
        entry.set_placeholder_text(Some(t("amanhã, ontem, quinta, +3, 28/11, semana…")));
        bar.append(&prompt);
        bar.append(&entry);
        root.append(&bar);

        let chips = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        chips.add_css_class("chips");
        root.append(&chips);

        let status = gtk::Label::new(None);
        status.add_css_class("weather-status");
        status.set_xalign(0.0);
        status.set_wrap(true);
        status.set_visible(false);
        root.append(&status);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&content)
            .vexpand(true)
            .build();
        scroller.set_size_request(-1, 452);
        root.append(&scroller);

        let panel = Rc::new(Self {
            root,
            entry: entry.clone(),
            status,
            content,
            reading: RefCell::new(None),
            query: RefCell::new(Query::Now),
        });

        for label in CHIPS {
            let text = format!("> {}", t(label));
            let chip = gtk::Button::with_label(t(label));
            chip.add_css_class("chip");
            let weak = Rc::downgrade(&panel);
            chip.connect_clicked(move |_| {
                if let Some(p) = weak.upgrade() {
                    p.ask(&text);
                }
            });
            chips.append(&chip);
        }

        {
            let weak = Rc::downgrade(&panel);
            entry.connect_changed(move |e| {
                let Some(p) = weak.upgrade() else { return };
                let query = query::parse(&e.text(), today());
                if query != Query::Unknown {
                    p.apply(query);
                }
            });
        }
        {
            let weak = Rc::downgrade(&panel);
            entry.connect_activate(move |e| {
                let Some(p) = weak.upgrade() else { return };
                let query = query::parse(&e.text(), today());
                if query == Query::Unknown {
                    p.say(&tf("não entendi «{}». tenta: amanhã, ontem, quinta, sexta passada, +3, 28/11, semana, fim de semana", e.text().trim_start_matches('>').trim()));
                } else {
                    p.apply(query);
                }
            });
        }
        {
            let keys = gtk::EventControllerKey::new();
            let weak = Rc::downgrade(&panel);
            keys.connect_key_pressed(move |_, key, _, _| {
                if key == gtk::gdk::Key::Escape && !weak.upgrade().is_some_and(|p| p.entry.text().is_empty()) {
                    if let Some(p) = weak.upgrade() {
                        p.entry.set_text("");
                        p.apply(Query::Now);
                    }
                    return gtk::glib::Propagation::Stop;
                }
                gtk::glib::Propagation::Proceed
            });
            entry.add_controller(keys);
        }

        let weak = Rc::downgrade(&panel);
        s.weather.reading.subscribe(move |reading| {
            if let Some(p) = weak.upgrade() {
                *p.reading.borrow_mut() = reading.clone();
                p.render();
            }
        });
        panel
    }

    /// Writes `text` in the `>` line and shows it.
    pub fn ask(&self, text: &str) {
        let text = text.trim_start_matches('>').trim();
        self.entry.set_text(text);
        self.entry.set_position(-1);
        let query = query::parse(text, today());
        self.apply(query);
    }

    fn apply(&self, query: Query) {
        *self.query.borrow_mut() = query;
        self.render();
    }

    fn say(&self, text: &str) {
        self.status.set_label(text);
        self.status.set_visible(!text.is_empty());
    }

    fn render(&self) {
        crate::widgets::clear(&self.content);
        self.say("");
        let reading = self.reading.borrow();
        let Some(r) = reading.as_ref() else {
            let empty = gtk::Label::new(Some(t("sem dados do clima")));
            empty.add_css_class("empty");
            self.content.append(&empty);
            return;
        };
        let open = self.weak_open();
        match *self.query.borrow() {
            Query::Now | Query::Unknown => self.now(r, open),
            Query::Day(offset) => self.day(r, offset),
            Query::Week(start) => self.range(r, start, 7, open, if start < 0 { t("semana passada") } else if start > 0 { t("próxima semana") } else { t("os próximos 7 dias") }),
            Query::Weekend(start) => self.range(r, start, 2, open, t("fim de semana")),
        }
    }

    /// Opens a day by writing its offset in the `>` line: a weekday name
    /// would mean the next one, not a past one.
    fn weak_open(&self) -> impl Fn(i32) + Clone + 'static {
        let entry = self.entry.downgrade();
        move |offset: i32| {
            if let Some(entry) = entry.upgrade() {
                entry.set_text(&format!("{offset:+}"));
            }
        }
    }

    fn now(&self, r: &Reading, open: impl Fn(i32) + Clone + 'static) {
        self.content.append(&cards::hero(r));
        let middle = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        middle.append(&cards::now_metrics(r));
        let sky = cards::sky_card(r);
        sky.set_hexpand(true);
        middle.append(&sky);
        self.content.append(&middle);
        self.content.append(&cards::section(
            t("próximas 24 horas"),
            &cards::hours(&r.hourly, true),
            Outline::Rounded(24.0),
        ));
        self.content.append(&cards::section(
            t("dias"),
            &cards::days(r, open),
            Outline::Rounded(24.0),
        ));
    }

    fn in_range(&self, r: &Reading, offset: i32) -> bool {
        if (r.first_offset()..=r.last_offset()).contains(&offset) {
            return true;
        }
        self.say(&tf(
            "dá pra ver de {} dias atrás até {} dias pra frente",
            -r.first_offset(),
        )
        .replacen("{}", &r.last_offset().to_string(), 1));
        false
    }

    fn day(&self, r: &Reading, offset: i32) {
        if !self.in_range(r, offset) {
            return;
        }
        let Some(d) = r.day(offset) else { return };
        self.content.append(&title(&cards::name(d), &cards::long_date(d)));
        self.content.append(&cards::day_hero(r, d));
        self.content.append(&cards::day_metrics(d));
        if !d.hours.is_empty() {
            let hours = cards::hours(&d.hours, false);
            self.content.append(&cards::section(t("hora a hora"), &hours, Outline::Rounded(24.0)));
        }
    }

    fn range(&self, r: &Reading, start: i32, count: i32, open: impl Fn(i32) + Clone + 'static, heading: &str) {
        let (first, last) = (start.max(r.first_offset()), (start + count - 1).min(r.last_offset()));
        if first > last {
            self.in_range(r, start);
            return;
        }
        let days: Vec<&Day> = (first..=last).filter_map(|o| r.day(o)).collect();
        let span = match (days.first(), days.last()) {
            (Some(a), Some(b)) => format!("{} – {}", cards::long_date(a), cards::long_date(b)),
            _ => String::new(),
        };
        self.content.append(&title(heading, &span));
        self.content.append(&cards::week(r, first, last - first + 1, open));
        self.content.append(&insights(&days));
    }
}

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

fn title(main: &str, sub: &str) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    column.add_css_class("weather-title");
    let main = gtk::Label::new(Some(main));
    main.add_css_class("title");
    main.set_xalign(0.0);
    let sub = gtk::Label::new(Some(sub));
    sub.add_css_class("dim");
    sub.set_xalign(0.0);
    column.append(&main);
    column.append(&sub);
    column
}

/// The warmest, the coolest and the rainiest of a few days.
fn insights(days: &[&Day]) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("insights");
    row.set_homogeneous(true);
    let pick = |icon: &str, title: &str, day: Option<&&Day>, value: &dyn Fn(&Day) -> String| {
        let card = gtk::Box::new(gtk::Orientation::Vertical, 2);
        card.add_css_class("insight");
        let head = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        head.append(&icons::icon(icon));
        let caption = gtk::Label::new(Some(title));
        caption.add_css_class("dim");
        head.append(&caption);
        card.append(&head);
        if let Some(d) = day {
            let name = gtk::Label::new(Some(&cards::name(d)));
            name.add_css_class("insight-day");
            name.set_xalign(0.0);
            let v = gtk::Label::new(Some(&value(d)));
            v.set_xalign(0.0);
            card.append(&name);
            card.append(&v);
        }
        card
    };
    if days.len() > 1 {
        row.append(&pick("local_fire_department", t("mais quente"), days.iter().max_by_key(|d| d.high), &|d| format!("{}°", d.high)));
        row.append(&pick("ac_unit", t("mais fresco"), days.iter().min_by_key(|d| d.low), &|d| format!("{}°", d.low)));
        row.append(&pick("umbrella", t("mais chuvoso"), days.iter().max_by(|a, b| a.rain_mm.total_cmp(&b.rain_mm)), &|d| format!("{} mm · {}%", d.rain_mm, d.rain)));
    }
    row
}
