//! Lembretes: a small window for the alarms and reminders the shell keeps.
//! A line at the top adds one from a few words, the lists below change
//! them, and a switch turns the whole thing off.

mod row;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;

use crate::i18n::{t, tf};
use crate::icons;
use crate::services::reminders::{parse, when, Reminder, Reminders};
use crate::widgets::shape::{Kind, Shape};

pub struct App {
    window: gtk::Window,
}

impl App {
    /// `persist` saves the switch and the snooze into octopus.toml.
    pub fn new(app: &gtk::Application, service: &Rc<Reminders>, snooze: u32, persist: Rc<dyn Fn(bool, u32)>) -> Rc<Self> {
        let window = gtk::Window::builder()
            .application(app)
            .title(t("lembretes"))
            .default_width(440)
            .default_height(640)
            .hide_on_close(true)
            .build();
        window.add_css_class("reminders-app");

        let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
        root.add_css_class("reminders-root");

        // the header: a badge, the title, how many are on, the switch
        let head = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        let badge = Shape::new(Kind::Cookie12, 52, true, Some(&icons::icon("alarm")));
        badge.add_css_class("reminders-badge");
        head.append(&badge);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
        texts.set_hexpand(true);
        texts.set_valign(gtk::Align::Center);
        let title = gtk::Label::new(Some(t("lembretes")));
        title.add_css_class("reminders-title");
        title.set_xalign(0.0);
        let count = gtk::Label::new(None);
        count.add_css_class("reminders-count");
        count.set_xalign(0.0);
        texts.append(&title);
        texts.append(&count);
        head.append(&texts);
        let master = gtk::Switch::new();
        master.set_active(*service.enabled.get());
        master.set_valign(gtk::Align::Center);
        master.set_tooltip_text(Some(t("desligado, nada toca nem avisa")));
        head.append(&master);
        root.append(&head);

        // a few words make one
        let quick = gtk::Box::new(gtk::Orientation::Vertical, 6);
        quick.add_css_class("reminders-quick");
        let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let entry = gtk::Entry::new();
        entry.set_hexpand(true);
        entry.set_placeholder_text(Some(t("10 min chá · 18h pão · amanhã 9h dentista")));
        let as_alarm = gtk::ToggleButton::new();
        as_alarm.set_child(Some(&icons::icon("alarm")));
        as_alarm.add_css_class("reminders-alarm-toggle");
        as_alarm.set_tooltip_text(Some(t("como alarme: toca até você parar")));
        let add = icons::button("add", t("adicionar"));
        add.add_css_class("reminders-add");
        line.append(&entry);
        line.append(&as_alarm);
        line.append(&add);
        let preview = gtk::Label::new(Some(t("escreva quando e o quê; enter adiciona")));
        preview.add_css_class("reminders-preview");
        preview.set_xalign(0.0);
        quick.append(&line);
        quick.append(&preview);
        root.append(&quick);

        // the lists
        let lists = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let scroller = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&lists).vexpand(true).build();
        root.append(&scroller);

        // the snooze, at the foot
        let foot = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        foot.add_css_class("reminders-foot");
        let label = gtk::Label::new(Some(t("soneca")));
        label.add_css_class("reminders-sub");
        foot.append(&label);
        let snoozes = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        snoozes.add_css_class("segmented");
        snoozes.set_hexpand(true);
        snoozes.set_homogeneous(true);
        let current_snooze = Rc::new(std::cell::Cell::new(snooze));
        let mut first: Option<gtk::ToggleButton> = None;
        for minutes in [5u32, 10, 15, 30] {
            let b = gtk::ToggleButton::with_label(&tf("{} min", minutes));
            b.set_group(first.as_ref());
            b.set_active(minutes == snooze);
            let (s, persist, master, cs) = (service.clone(), persist.clone(), master.clone(), current_snooze.clone());
            b.connect_toggled(move |b| {
                if b.is_active() {
                    cs.set(minutes);
                    s.set_snooze(minutes);
                    persist(master.is_active(), minutes);
                }
            });
            first.get_or_insert(b.clone());
            snoozes.append(&b);
        }
        foot.append(&snoozes);
        root.append(&foot);

        window.set_child(Some(&root));

        {
            let (s, persist, cs) = (service.clone(), persist.clone(), current_snooze.clone());
            master.connect_active_notify(move |m| {
                s.set_enabled(m.is_active());
                persist(m.is_active(), cs.get());
            });
        }

        // the preview of what the line will make
        {
            let (preview, alarm_on) = (preview.clone(), as_alarm.clone());
            let show = move |text: &str| match parse::parse(text, chrono::Local::now()) {
                Some(q) => {
                    let kind = if q.alarm || alarm_on.is_active() { t("alarme") } else { t("lembrete") };
                    let what = if q.title.is_empty() { kind.to_owned() } else { q.title.clone() };
                    preview.set_label(&format!("{what} · {}", when(&q.at)));
                    preview.add_css_class("ready");
                }
                None => {
                    preview.set_label(t("escreva quando e o quê; enter adiciona"));
                    preview.remove_css_class("ready");
                }
            };
            let show = Rc::new(show);
            let f = show.clone();
            entry.connect_changed(move |e| f(&e.text()));
            let e = entry.clone();
            as_alarm.connect_toggled(move |_| show(&e.text()));
        }
        let submit = {
            let (s, entry, as_alarm) = (service.clone(), entry.clone(), as_alarm.clone());
            move || {
                let text = entry.text().to_string();
                let alarm = as_alarm.is_active().then_some(true);
                if s.quick(&text, alarm).is_some() {
                    entry.set_text("");
                    as_alarm.set_active(false);
                } else {
                    entry.add_css_class("error");
                    let e = entry.clone();
                    gtk::glib::timeout_add_local_once(std::time::Duration::from_millis(600), move || e.remove_css_class("error"));
                }
            }
        };
        let submit = Rc::new(submit);
        let f = submit.clone();
        entry.connect_activate(move |_| f());
        add.connect_clicked(move |_| submit());

        // rows are rebuilt only when the set of reminders changes; edits
        // update their own row, so an open editor stays open
        let rows: Rc<RefCell<(Vec<u64>, HashMap<u64, Rc<dyn Fn(&Reminder)>>)>> = Rc::default();
        {
            let (s, lists, count) = (service.clone(), lists.clone(), count.clone());
            let enabled = service.enabled.clone();
            let refresh = move |list: &Vec<Reminder>| {
                let on = list.iter().filter(|r| r.enabled).count();
                count.set_label(&if !*enabled.get() { t("desligados").to_owned() } else { tf("{} ativos", on) });
                let ids: Vec<u64> = list.iter().map(|r| r.id).collect();
                let mut state = rows.borrow_mut();
                if state.0 == ids {
                    for r in list {
                        if let Some(f) = state.1.get(&r.id) {
                            f(r);
                        }
                    }
                    return;
                }
                crate::widgets::clear(&lists);
                state.1.clear();
                state.0 = ids;
                if list.is_empty() {
                    lists.append(&empty());
                    return;
                }
                for (heading, alarms) in [(t("alarmes"), true), (t("lembretes"), false)] {
                    let mine: Vec<&Reminder> = list.iter().filter(|r| r.alarm == alarms).collect();
                    if mine.is_empty() {
                        continue;
                    }
                    let label = gtk::Label::new(Some(heading));
                    label.add_css_class("reminders-section");
                    label.set_xalign(0.0);
                    lists.append(&label);
                    for r in mine {
                        let (card, f) = row::build(&s, r);
                        lists.append(&card);
                        state.1.insert(r.id, f);
                    }
                }
            };
            let refresh = Rc::new(refresh);
            let f = refresh.clone();
            service.list.subscribe(move |l| f(l));
            let (s, f) = (service.clone(), refresh.clone());
            service.enabled.subscribe(move |_| f(&s.list.get()));
        }

        Rc::new(Self { window })
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn empty() -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    column.add_css_class("reminders-empty");
    column.set_vexpand(true);
    column.set_valign(gtk::Align::Center);
    let shape = Shape::new(Kind::Sunny, 88, true, Some(&icons::icon("alarm_add")));
    shape.add_css_class("reminders-empty-shape");
    shape.set_halign(gtk::Align::Center);
    column.append(&shape);
    let title = gtk::Label::new(Some(t("nada marcado")));
    title.add_css_class("reminders-title");
    column.append(&title);
    let hint = gtk::Label::new(Some(t("escreva em cima, ou no spotlight depois de um !")));
    hint.add_css_class("reminders-sub");
    hint.set_wrap(true);
    hint.set_justify(gtk::Justification::Center);
    column.append(&hint);
    column
}
