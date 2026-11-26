//! The battery on the desktop: a big pill that fills, with a bolt while
//! charging, the percentage and how long it lasts or takes to fill.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

use crate::i18n::{t, tf};
use crate::icons;

struct Reading {
    percent: f64,
    status: String,
    /// Hours to empty or to full, when the battery says.
    hours: Option<f64>,
}

fn read() -> Option<Reading> {
    let dir = std::fs::read_dir("/sys/class/power_supply").ok()?.flatten().map(|e| e.path()).find(|p| {
        p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("BAT"))
    })?;
    let get = |name: &str| std::fs::read_to_string(dir.join(name)).ok().and_then(|s| s.trim().parse::<f64>().ok());
    let status = std::fs::read_to_string(dir.join("status")).unwrap_or_default().trim().to_owned();
    let (now, full, rate) = match (get("charge_now"), get("charge_full")) {
        (Some(n), Some(f)) => (n, f, get("current_now")),
        _ => (get("energy_now")?, get("energy_full")?, get("power_now")),
    };
    let percent = (now / full).clamp(0.0, 1.0);
    let hours = rate.filter(|r| *r > 0.0).map(|r| match status.as_str() {
        "Charging" => (full - now) / r,
        _ => now / r,
    });
    Some(Reading { percent, status, hours })
}

pub fn build(show_time: bool) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-battery");

    let pill = gtk::Overlay::new();
    pill.add_css_class("battery-pill");
    pill.set_valign(gtk::Align::Center);
    let level = gtk::LevelBar::for_interval(0.0, 1.0);
    level.set_orientation(gtk::Orientation::Vertical);
    level.set_inverted(true);
    level.add_css_class("battery-level");
    pill.set_child(Some(&level));
    let bolt = icons::icon("bolt");
    bolt.add_css_class("battery-bolt");
    bolt.set_halign(gtk::Align::Center);
    bolt.set_valign(gtk::Align::Center);
    pill.add_overlay(&bolt);
    root.append(&pill);

    let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
    texts.set_valign(gtk::Align::Center);
    let percent = gtk::Label::new(Some("–"));
    percent.add_css_class("battery-percent");
    percent.set_xalign(0.0);
    let state = gtk::Label::new(None);
    state.add_css_class("desktop-desc");
    state.set_xalign(0.0);
    let time = gtk::Label::new(None);
    time.add_css_class("dim");
    time.set_xalign(0.0);
    texts.append(&percent);
    texts.append(&state);
    time.set_visible(show_time);
    texts.append(&time);
    root.append(&texts);

    let tick = Rc::new(move || {
        let Some(r) = read() else {
            percent.set_label(t("sem bateria"));
            return;
        };
        level.set_value(r.percent);
        percent.set_label(&format!("{:.0}%", r.percent * 100.0));
        let charging = r.status == "Charging";
        bolt.set_visible(charging);
        if r.percent <= 0.15 && !charging {
            level.add_css_class("low");
        } else {
            level.remove_css_class("low");
        }
        state.set_label(match r.status.as_str() {
            "Charging" => t("carregando"),
            "Full" => t("carregada"),
            "Not charging" => t("na tomada"),
            _ => t("na bateria"),
        });
        time.set_label(&match r.hours {
            Some(h) if h < 48.0 => {
                let (hh, mm) = (h.floor() as u32, ((h.fract()) * 60.0).round() as u32);
                if charging {
                    tf("cheia em {}", format!("{hh}h{mm:02}"))
                } else {
                    tf("dura mais {}", format!("{hh}h{mm:02}"))
                }
            }
            _ => String::new(),
        });
    });
    let timer: Rc<Cell<Option<glib::SourceId>>> = Rc::default();
    {
        let (tick, timer) = (tick.clone(), timer.clone());
        root.connect_map(move |_| {
            tick();
            let t = tick.clone();
            timer.set(Some(glib::timeout_add_local(Duration::from_secs(20), move || {
                t();
                glib::ControlFlow::Continue
            })));
        });
    }
    root.connect_unmap(move |_| {
        if let Some(id) = timer.take() {
            id.remove();
        }
    });
    root
}
