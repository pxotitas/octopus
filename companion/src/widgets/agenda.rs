//! The calendar with its events: days that have something get a mark, and
//! the chosen day's events show under it. And a plain list of what is next,
//! for the desktop.

use std::rc::Rc;

use chrono::{Datelike, Local, NaiveDate};
use gtk::prelude::*;

use crate::i18n::t;
use crate::services::calendar::{Calendar, Event};

fn row(event: &Event, with_day: bool) -> gtk::Box {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    line.add_css_class("agenda-row");
    if !event.calendar.is_empty() {
        line.set_tooltip_text(Some(&event.calendar));
    }
    let dot = gtk::Box::new(gtk::Orientation::Vertical, 0);
    dot.add_css_class("agenda-dot");
    dot.add_css_class(&format!("tone-{}", event.index % 4));
    dot.set_valign(gtk::Align::Fill);
    line.append(&dot);
    let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
    texts.set_hexpand(true);
    let title = gtk::Label::new(Some(&event.title));
    title.add_css_class("agenda-title");
    title.set_xalign(0.0);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    texts.append(&title);
    let mut when = event.when();
    if with_day {
        let day = event.day();
        let today = Local::now().date_naive();
        let label = if day == today {
            t("hoje").to_owned()
        } else if day == today.succ_opt().unwrap_or(today) {
            t("amanhã").to_owned()
        } else {
            day.format_localized(t("%a, %-d de %b"), crate::i18n::locale()).to_string()
        };
        when = format!("{label} · {when}");
    }
    if !event.location.is_empty() {
        when = format!("{when} · {}", event.location);
    }
    let sub = gtk::Label::new(Some(&when));
    sub.add_css_class("dim");
    sub.set_xalign(0.0);
    sub.set_ellipsize(gtk::pango::EllipsizeMode::End);
    texts.append(&sub);
    line.append(&texts);
    line
}

fn date_of(calendar: &gtk::Calendar) -> NaiveDate {
    let d = calendar.date();
    NaiveDate::from_ymd_opt(d.year(), d.month() as u32, d.day_of_month() as u32).unwrap_or_else(|| Local::now().date_naive())
}

/// A calendar marking the days with events, and the chosen day's list.
pub fn calendar(service: &Rc<Calendar>, class: &str) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let calendar = gtk::Calendar::new();
    calendar.add_css_class(class);
    root.append(&calendar);
    if !service.enabled() {
        return root;
    }
    let list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    list.add_css_class("agenda-day");
    root.append(&list);

    let mark = {
        let service = service.clone();
        move |calendar: &gtk::Calendar| {
            calendar.clear_marks();
            let shown = date_of(calendar);
            let events = service.events.get();
            for event in events.iter().flatten() {
                let mut day = event.day();
                // multi-day events mark every day they cover
                while event.on(day) {
                    if day.year() == shown.year() && day.month() == shown.month() {
                        calendar.mark_day(day.day());
                    }
                    let Some(next) = day.succ_opt() else { break };
                    day = next;
                }
            }
        }
    };
    let fill = {
        let (service, list) = (service.clone(), list.clone());
        move |calendar: &gtk::Calendar| {
            crate::widgets::clear(&list);
            let day = date_of(calendar);
            let events = service.on(day);
            if events.is_empty() {
                let empty = gtk::Label::new(Some(t("nada marcado nesse dia")));
                empty.add_css_class("dim");
                empty.set_xalign(0.0);
                list.append(&empty);
            }
            for event in events.iter().take(4) {
                list.append(&row(event, false));
            }
            if events.len() > 4 {
                let more = gtk::Label::new(Some(&crate::i18n::tf("e mais {}", events.len() - 4)));
                more.add_css_class("dim");
                more.set_xalign(0.0);
                list.append(&more);
            }
        }
    };
    let (m, f) = (Rc::new(mark), Rc::new(fill));
    {
        let (m, f, cal) = (m.clone(), f.clone(), calendar.clone());
        service.events.subscribe(move |_| {
            m(&cal);
            f(&cal);
        });
    }
    {
        let f = f.clone();
        calendar.connect_day_selected(move |c| f(c));
    }
    for signal in ["next-month", "prev-month", "next-year", "prev-year"] {
        let (m, f) = (m.clone(), f.clone());
        calendar.connect_local(signal, false, move |values| {
            if let Some(c) = values[0].get::<gtk::Calendar>().ok() {
                m(&c);
                f(&c);
            }
            None
        });
    }
    root
}

/// What comes next, grouped as one list.
pub fn upcoming(service: &Rc<Calendar>, count: usize) -> gtk::Box {
    let list = gtk::Box::new(gtk::Orientation::Vertical, 6);
    list.add_css_class("agenda-list");
    let s = service.clone();
    let l = list.clone();
    service.events.subscribe(move |events| {
        crate::widgets::clear(&l);
        if !s.enabled() {
            let empty = gtk::Label::new(Some(t("nenhum calendário: cole o endereço iCal nas configurações")));
            empty.add_css_class("dim");
            empty.set_wrap(true);
            empty.set_xalign(0.0);
            l.append(&empty);
            return;
        }
        if events.is_none() {
            let wait = gtk::Label::new(Some(t("buscando…")));
            wait.add_css_class("dim");
            wait.set_xalign(0.0);
            l.append(&wait);
            return;
        }
        let next = s.upcoming(count);
        if next.is_empty() {
            let empty = gtk::Label::new(Some(t("nada pela frente")));
            empty.add_css_class("dim");
            empty.set_xalign(0.0);
            l.append(&empty);
        }
        for event in &next {
            l.append(&row(event, true));
        }
    });
    list
}
