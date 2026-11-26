//! One alarm or reminder: the time, what it is and how it repeats, a switch;
//! a tap opens its editor right under it.

use std::rc::Rc;

use chrono::{Datelike, Duration, Local, NaiveTime, TimeZone, Timelike};
use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;
use crate::services::reminders::{when, Reminder, Reminders};
use crate::widgets::shape::{Kind, Shape};

const REPEATS: [(&str, &str); 4] = [("", "uma vez"), ("daily", "todo dia"), ("weekdays", "dias úteis"), ("weekly", "toda semana")];

fn repeat_label(repeat: &str) -> &'static str {
    REPEATS.iter().find(|(k, _)| *k == repeat).map(|(_, l)| t(l)).unwrap_or("")
}

/// The card, and what brings it up to date when its reminder changes.
pub fn build(service: &Rc<Reminders>, r: &Reminder) -> (gtk::Box, Rc<dyn Fn(&Reminder)>) {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("reminder");
    if !r.enabled {
        card.add_css_class("off");
    }

    let head = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    head.add_css_class("reminder-head");
    let glyph = icons::icon(if r.alarm { "alarm" } else { "notifications_active" });
    let badge = Shape::new(if r.alarm { Kind::Cookie9 } else { Kind::Clover4 }, 44, true, Some(&glyph));
    badge.add_css_class("reminder-badge");
    head.append(&badge);

    let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
    texts.set_hexpand(true);
    texts.set_valign(gtk::Align::Center);
    let time = gtk::Label::new(Some(&r.at.format("%H:%M").to_string()));
    time.add_css_class("reminder-time");
    time.set_xalign(0.0);
    let title = gtk::Label::new(Some(&r.label()));
    title.add_css_class("reminder-title");
    title.set_xalign(0.0);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let detail = if r.repeat.is_empty() { when(&r.at) } else { repeat_label(&r.repeat).to_owned() };
    let sub = gtk::Label::new(Some(&detail));
    sub.add_css_class("reminder-sub");
    sub.set_xalign(0.0);
    texts.append(&time);
    texts.append(&title);
    texts.append(&sub);
    head.append(&texts);

    let switch = gtk::Switch::new();
    switch.set_active(r.enabled);
    switch.set_valign(gtk::Align::Center);
    {
        let (s, id) = (service.clone(), r.id);
        switch.connect_active_notify(move |sw| {
            let on = sw.is_active();
            // set by the row's own refresh (the list is being sent out right
            // now): nothing to save, and saving here would change it mid-send
            if s.list.get().iter().any(|r| r.id == id && r.enabled == on) {
                return;
            }
            s.update(id, |r| {
                r.enabled = on;
                // turned back on, a one-off in the past moves to its next time
                if on && r.at <= Local::now() {
                    while r.at <= Local::now() {
                        r.at += Duration::days(1);
                    }
                }
            });
        });
    }
    head.append(&switch);

    let toggle = gtk::Button::new();
    toggle.add_css_class("reminder-open");
    toggle.set_child(Some(&head));
    card.append(&toggle);

    let revealer = gtk::Revealer::builder().transition_type(gtk::RevealerTransitionType::SlideDown).transition_duration(220).build();
    let editor_built = std::cell::Cell::new(false);
    {
        let (revealer, service, r) = (revealer.clone(), service.clone(), r.clone());
        toggle.connect_clicked(move |_| {
            if !editor_built.replace(true) {
                revealer.set_child(Some(&editor(&service, &r)));
            }
            revealer.set_reveal_child(!revealer.reveals_child());
        });
    }
    card.append(&revealer);

    let refresh: Rc<dyn Fn(&Reminder)> = {
        let card = card.clone();
        Rc::new(move |r: &Reminder| {
            if r.enabled {
                card.remove_css_class("off");
            } else {
                card.add_css_class("off");
            }
            glyph.set_label(if r.alarm { "alarm" } else { "notifications_active" });
            badge.morph(if r.alarm { Kind::Cookie9 } else { Kind::Clover4 });
            time.set_label(&r.at.format("%H:%M").to_string());
            title.set_label(&r.label());
            sub.set_label(&if r.repeat.is_empty() { when(&r.at) } else { repeat_label(&r.repeat).to_owned() });
            switch.set_active(r.enabled);
        })
    };
    (card, refresh)
}

fn editor(service: &Rc<Reminders>, r: &Reminder) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 12);
    column.add_css_class("reminder-editor");
    let id = r.id;

    let name = gtk::Entry::new();
    name.set_text(&r.title);
    name.set_placeholder_text(Some(t("o quê")));
    {
        let s = service.clone();
        name.connect_activate(move |e| {
            let text = e.text().to_string();
            s.update(id, |r| r.title = text);
        });
        let s = service.clone();
        let focus = gtk::EventControllerFocus::new();
        let e = name.clone();
        focus.connect_leave(move |_| {
            let text = e.text().to_string();
            if s.list.get().iter().any(|r| r.id == id && r.title != text) {
                s.update(id, |r| r.title = text);
            }
        });
        name.add_controller(focus);
    }
    column.append(&name);

    // the time: two big spinners, like a clock
    let clock = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    clock.add_css_class("reminder-clock");
    clock.set_halign(gtk::Align::Center);
    let hour = gtk::SpinButton::with_range(0.0, 23.0, 1.0);
    let minute = gtk::SpinButton::with_range(0.0, 59.0, 1.0);
    for (spin, value) in [(&hour, r.at.hour()), (&minute, r.at.minute())] {
        spin.set_value(value as f64);
        spin.set_wrap(true);
        spin.set_orientation(gtk::Orientation::Vertical);
        spin.connect_output(|s| {
            s.set_text(&format!("{:02}", s.value() as u32));
            gtk::glib::Propagation::Stop
        });
    }
    let colon = gtk::Label::new(Some(":"));
    colon.add_css_class("reminder-colon");
    clock.append(&hour);
    clock.append(&colon);
    clock.append(&minute);
    column.append(&clock);

    let calendar = gtk::Calendar::new();
    calendar.add_css_class("reminder-calendar");
    if let Ok(date) = gtk::glib::DateTime::from_local(r.at.year(), r.at.month() as i32, r.at.day() as i32, 0, 0, 0.0) {
        calendar.select_day(&date);
    }
    let day = gtk::MenuButton::new();
    day.add_css_class("reminder-day");
    day.set_label(&when(&r.at));
    let pop = gtk::Popover::new();
    pop.set_child(Some(&calendar));
    day.set_popover(Some(&pop));
    day.set_halign(gtk::Align::Center);
    column.append(&day);

    let set_time = {
        let (s, hour, minute, calendar, day) = (service.clone(), hour.clone(), minute.clone(), calendar.clone(), day.clone());
        move || {
            let date = calendar.date();
            let Some(d) = chrono::NaiveDate::from_ymd_opt(date.year(), date.month() as u32, date.day_of_month() as u32) else { return };
            let Some(time) = NaiveTime::from_hms_opt(hour.value() as u32, minute.value() as u32, 0) else { return };
            let Some(at) = Local.from_local_datetime(&d.and_time(time)).single() else { return };
            day.set_label(&when(&at));
            s.update(id, |r| {
                r.at = at;
                r.enabled = true;
            });
        }
    };
    let set_time = Rc::new(set_time);
    for spin in [&hour, &minute] {
        let f = set_time.clone();
        spin.connect_value_changed(move |_| f());
    }
    {
        let (f, pop) = (set_time.clone(), pop.clone());
        calendar.connect_day_selected(move |_| {
            f();
            pop.popdown();
        });
    }

    let repeat = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    repeat.add_css_class("segmented");
    repeat.set_homogeneous(true);
    let mut first: Option<gtk::ToggleButton> = None;
    for (key, label) in REPEATS {
        let b = gtk::ToggleButton::with_label(t(label));
        b.set_group(first.as_ref());
        b.set_active(r.repeat == key);
        let s = service.clone();
        b.connect_toggled(move |b| {
            if b.is_active() {
                s.update(id, |r| r.repeat = key.to_owned());
            }
        });
        first.get_or_insert(b.clone());
        repeat.append(&b);
    }
    column.append(&repeat);

    let bottom = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let alarm = gtk::Switch::new();
    alarm.set_active(r.alarm);
    alarm.set_valign(gtk::Align::Center);
    {
        let s = service.clone();
        alarm.connect_active_notify(move |sw| {
            let on = sw.is_active();
            s.update(id, |r| r.alarm = on);
        });
    }
    let alarm_label = gtk::Label::new(Some(t("toca até você parar")));
    alarm_label.add_css_class("reminder-sub");
    alarm_label.set_hexpand(true);
    alarm_label.set_xalign(0.0);
    bottom.append(&alarm);
    bottom.append(&alarm_label);
    let delete = icons::button("delete", t("apagar"));
    delete.add_css_class("reminder-delete");
    {
        let s = service.clone();
        delete.connect_clicked(move |_| s.remove(id));
    }
    bottom.append(&delete);
    column.append(&bottom);
    column
}
