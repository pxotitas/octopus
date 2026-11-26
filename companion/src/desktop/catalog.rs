//! Every widget the desktop can hold: what the case shows about it, and how
//! to build it.

use std::rc::Rc;

use gtk::prelude::*;

use crate::config;
use crate::services::Services;
use crate::widgets::clock::Clock;
use crate::widgets::shape::Kind;

pub struct Entry {
    pub id: &'static str,
    pub icon: &'static str,
    pub shape: Kind,
    pub title: &'static str,
    pub about: &'static str,
    pub min: (i32, i32),
    pub default: (i32, i32),
}

pub const ENTRIES: [Entry; 10] = [
    Entry {
        id: "weather",
        icon: "partly_cloudy_day",
        shape: Kind::Sunny,
        title: "clima",
        about: "agora num cookie, e os próximos dias",
        min: (300, 220),
        default: (380, 270),
    },
    Entry {
        id: "contributions",
        icon: "commit",
        shape: Kind::SoftBurst,
        title: "contribuições",
        about: "os quadradinhos do seu perfil no github",
        min: (280, 170),
        default: (560, 230),
    },
    Entry {
        id: "clock",
        icon: "schedule",
        shape: Kind::Cookie9,
        title: "relógio",
        about: "analógico, num cookie",
        min: (140, 140),
        default: (220, 220),
    },
    Entry {
        id: "media",
        icon: "music_note",
        shape: Kind::Puffy,
        title: "música",
        about: "o que está tocando, com os controles",
        min: (300, 110),
        default: (380, 140),
    },
    Entry {
        id: "calendar",
        icon: "calendar_month",
        shape: Kind::Clover4,
        title: "calendário",
        about: "o mês, com os seus eventos",
        min: (260, 260),
        default: (300, 360),
    },
    Entry {
        id: "agenda",
        icon: "event_upcoming",
        shape: Kind::Cookie6,
        title: "agenda",
        about: "os próximos eventos do seu calendário",
        min: (260, 160),
        default: (320, 260),
    },
    Entry {
        id: "system",
        icon: "monitor_heart",
        shape: Kind::Cookie12,
        title: "sistema",
        about: "processador, memória e disco",
        min: (240, 170),
        default: (300, 200),
    },
    Entry {
        id: "battery",
        icon: "battery_charging_full",
        shape: Kind::Pentagon,
        title: "bateria",
        about: "quanto tem, e quanto dura",
        min: (220, 120),
        default: (280, 150),
    },
    Entry {
        id: "note",
        icon: "sticky_note_2",
        shape: Kind::Clover8,
        title: "nota",
        about: "um bilhete que se guarda sozinho",
        min: (200, 140),
        default: (280, 220),
    },
    Entry {
        id: "photo",
        icon: "photo",
        shape: Kind::Flower,
        title: "foto",
        about: "uma imagem sua, recortada no card",
        min: (160, 140),
        default: (300, 220),
    },
];

pub fn entry(id: &str) -> Option<&'static Entry> {
    let id = if id == "github" { "contributions" } else { id };
    ENTRIES.iter().find(|e| e.id == id)
}

/// Whether the widget can be placed with this configuration.
pub fn available(id: &str, desktop: &config::Desktop) -> bool {
    id != "contributions" || !desktop.github.is_empty()
}

pub struct Built {
    pub content: gtk::Widget,
    pub resize: Box<dyn Fn(i32, i32)>,
    pub click: Option<Box<dyn Fn()>>,
}

pub fn build(id: &str, s: &Rc<Services>, desktop: &config::Desktop) -> Option<Built> {
    let entry = entry(id)?;
    if !available(entry.id, desktop) {
        return None;
    }
    Some(match entry.id {
        "weather" => {
            let w = super::weather::Weather::new(s);
            Built {
                content: w.root.clone().upcast(),
                resize: Box::new(move |width, height| w.resize(width, height)),
                click: Some(Box::new(|| crate::exec::spawn("octopus", &["open", "weather"]))),
            }
        }
        "contributions" => Built {
            content: super::contributions::build(&desktop.github, desktop.contributions_colors != "theme").upcast(),
            resize: Box::new(|_, _| {}),
            click: None,
        },
        "clock" => {
            let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
            root.add_css_class("desktop-bare");
            root.add_css_class("desktop-clock");
            // the face inside the card's padding
            let clock = Clock::new(entry.default.0 - 28);
            clock.set_hexpand(true);
            clock.set_vexpand(true);
            root.append(&clock);
            Built {
                content: root.upcast(),
                resize: Box::new(move |w, h| clock.set_size((w.min(h) - 28).max(60))),
                click: None,
            }
        }
        "media" => {
            let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
            root.add_css_class("desktop-widget");
            root.add_css_class("desktop-media");
            let player = crate::widgets::media::player(&s.media, crate::widgets::media::Size::Card, None);
            player.set_vexpand(true);
            player.set_valign(gtk::Align::Center);
            root.append(&player);
            Built {
                content: root.upcast(),
                resize: Box::new(|_, _| {}),
                click: None,
            }
        }
        "calendar" => {
            let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
            root.add_css_class("desktop-widget");
            root.add_css_class("desktop-calendar");
            let calendar = crate::widgets::agenda::calendar(&s.calendar, "dash-calendar");
            calendar.set_vexpand(true);
            root.append(&calendar);
            Built {
                content: root.upcast(),
                resize: Box::new(|_, _| {}),
                click: None,
            }
        }
        "agenda" => {
            let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
            root.add_css_class("desktop-widget");
            root.add_css_class("desktop-agenda");
            let title = gtk::Label::new(Some(crate::i18n::t("próximos")));
            title.add_css_class("desktop-title");
            title.set_xalign(0.0);
            root.append(&title);
            let list = crate::widgets::agenda::upcoming(&s.calendar, 6);
            let scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&list).build();
            scroll.set_vexpand(true);
            root.append(&scroll);
            Built { content: root.upcast(), resize: Box::new(|_, _| {}), click: None }
        }
        "system" => Built { content: super::system::build().upcast(), resize: Box::new(|_, _| {}), click: None },
        "battery" => Built { content: super::battery::build().upcast(), resize: Box::new(|_, _| {}), click: None },
        "note" => Built { content: super::note::build().upcast(), resize: Box::new(|_, _| {}), click: None },
        "photo" => Built { content: super::photo::build().upcast(), resize: Box::new(|_, _| {}), click: None },
        _ => return None,
    })
}
