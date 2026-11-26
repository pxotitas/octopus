//! CPU, memory and disk on the desktop: three bars that fill up. Read
//! from /proc and the root filesystem, only while the widget is on screen.

use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;

/// (busy, total) jiffies from the first line of /proc/stat.
fn cpu_times() -> Option<(u64, u64)> {
    let stat = std::fs::read_to_string("/proc/stat").ok()?;
    let values: Vec<u64> = stat.lines().next()?.split_whitespace().skip(1).filter_map(|v| v.parse().ok()).collect();
    let idle = values.get(3)? + values.get(4).unwrap_or(&0);
    let total: u64 = values.iter().sum();
    Some((total - idle, total))
}

fn memory() -> Option<f64> {
    let info = std::fs::read_to_string("/proc/meminfo").ok()?;
    let get = |key: &str| info.lines().find(|l| l.starts_with(key)).and_then(|l| l.split_whitespace().nth(1)?.parse::<f64>().ok());
    let (total, available) = (get("MemTotal:")?, get("MemAvailable:")?);
    Some(1.0 - available / total)
}

extern "C" {
    fn statvfs(path: *const std::ffi::c_char, buf: *mut [u64; 16]) -> i32;
}

/// The root filesystem's use, from statvfs (blocks, free, available).
fn disk() -> Option<f64> {
    let mut buf = [0u64; 16];
    let path = std::ffi::CString::new("/").ok()?;
    if unsafe { statvfs(path.as_ptr(), &mut buf) } != 0 {
        return None;
    }
    // struct statvfs on 64-bit linux: bsize, frsize, blocks, bfree, bavail, …
    let (blocks, available) = (buf[2] as f64, buf[4] as f64);
    (blocks > 0.0).then(|| 1.0 - available / blocks)
}

fn meter(icon: &str, name: &str) -> (gtk::Box, gtk::LevelBar, gtk::Label) {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    line.add_css_class("system-meter");
    let glyph = icons::icon(icon);
    glyph.add_css_class("system-icon");
    line.append(&glyph);
    let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
    column.set_hexpand(true);
    column.set_valign(gtk::Align::Center);
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let label = gtk::Label::new(Some(name));
    label.set_xalign(0.0);
    label.set_hexpand(true);
    label.add_css_class("system-name");
    let value = gtk::Label::new(Some("–"));
    value.add_css_class("system-value");
    head.append(&label);
    head.append(&value);
    column.append(&head);
    let bar = gtk::LevelBar::for_interval(0.0, 1.0);
    bar.add_css_class("system-bar");
    bar.set_valign(gtk::Align::Center);
    column.append(&bar);
    line.append(&column);
    (line, bar, value)
}

pub fn build() -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-system");
    let (cpu, cpu_bar, cpu_value) = meter("memory", t("processador"));
    let (ram, ram_bar, ram_value) = meter("memory_alt", t("memória"));
    let (hd, hd_bar, hd_value) = meter("hard_drive", t("disco"));
    for line in [&cpu, &ram, &hd] {
        line.set_vexpand(true);
        root.append(line);
    }

    let last = Rc::new(Cell::new(cpu_times().unwrap_or((0, 0))));
    let set = |bar: &gtk::LevelBar, label: &gtk::Label, v: f64| {
        bar.set_value(v.clamp(0.0, 1.0));
        label.set_label(&format!("{:.0}%", v * 100.0));
    };
    let tick = move || {
        if let Some((busy, total)) = cpu_times() {
            let (b, t) = last.replace((busy, total));
            if total > t {
                set(&cpu_bar, &cpu_value, (busy - b) as f64 / (total - t) as f64);
            }
        }
        if let Some(m) = memory() {
            set(&ram_bar, &ram_value, m);
        }
        if let Some(d) = disk() {
            set(&hd_bar, &hd_value, d);
        }
    };
    let tick = Rc::new(tick);
    let timer: Rc<Cell<Option<glib::SourceId>>> = Rc::default();
    {
        let (tick, timer) = (tick.clone(), timer.clone());
        root.connect_map(move |_| {
            tick();
            let t = tick.clone();
            timer.set(Some(glib::timeout_add_local(Duration::from_secs(3), move || {
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
