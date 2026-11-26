//! Alarms and reminders, kept in ~/.local/state/octopus/reminders.json and
//! fired by the shell itself: an alarm rings until it's answered, a
//! reminder chimes once. Both arrive as notifications with snooze.

pub mod parse;
mod sound;

use std::cell::{Cell, RefCell};
use std::process::Child;
use std::rc::Rc;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Datelike, Duration, Local, Weekday};
use gtk::glib;
use serde::{Deserialize, Serialize};

use crate::i18n::{t, tf};
use crate::services::notifications::Notifications;
use crate::store::Store;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reminder {
    pub id: u64,
    pub title: String,
    pub at: DateTime<Local>,
    /// "", "daily", "weekdays" or "weekly".
    #[serde(default)]
    pub repeat: String,
    #[serde(default)]
    pub alarm: bool,
    #[serde(default = "yes")]
    pub enabled: bool,
    /// A snooze: gone once it fires.
    #[serde(default)]
    pub once: bool,
}

fn yes() -> bool {
    true
}

impl Reminder {
    pub fn new(title: &str, at: DateTime<Local>, alarm: bool) -> Self {
        Self { id: 0, title: title.trim().to_owned(), at, repeat: String::new(), alarm, enabled: true, once: false }
    }

    /// What to show when there's no title.
    pub fn label(&self) -> String {
        if !self.title.is_empty() {
            self.title.clone()
        } else if self.alarm {
            t("alarme").to_owned()
        } else {
            t("lembrete").to_owned()
        }
    }

    /// The first time after `now` this happens again.
    fn next_after(&self, now: DateTime<Local>) -> Option<DateTime<Local>> {
        let mut at = self.at;
        let step = match self.repeat.as_str() {
            "daily" | "weekdays" => Duration::days(1),
            "weekly" => Duration::weeks(1),
            _ => return None,
        };
        while at <= now || (self.repeat == "weekdays" && matches!(at.weekday(), Weekday::Sat | Weekday::Sun)) {
            at += step;
        }
        Some(at)
    }
}

fn path() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| glib::home_dir().join(".local/state"))
        .join("octopus/reminders.json")
}

pub struct Reminders {
    pub list: Rc<Store<Vec<Reminder>>>,
    pub enabled: Rc<Store<bool>>,
    notifications: Rc<Notifications>,
    snooze: Cell<u32>,
    timer: RefCell<Option<glib::SourceId>>,
    ringing: RefCell<Option<Child>>,
}

impl Reminders {
    pub fn new(notifications: &Rc<Notifications>, config: &crate::config::RemindersConfig) -> Rc<Self> {
        let list: Vec<Reminder> = std::fs::read_to_string(path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();
        let this = Rc::new(Self {
            list: Store::new(list),
            enabled: Store::new(config.enabled),
            notifications: notifications.clone(),
            snooze: Cell::new(config.snooze.max(1)),
            timer: RefCell::default(),
            ringing: RefCell::default(),
        });
        this.arm();
        this
    }

    pub fn set_enabled(self: &Rc<Self>, on: bool) {
        self.enabled.set(on);
        self.arm();
    }

    pub fn set_snooze(&self, minutes: u32) {
        self.snooze.set(minutes.max(1));
    }

    pub fn add(self: &Rc<Self>, mut reminder: Reminder) -> u64 {
        let id = self.list.get().iter().map(|r| r.id).max().unwrap_or(0) + 1;
        reminder.id = id;
        self.list.update(|l| l.push(reminder));
        self.changed();
        id
    }

    pub fn update(self: &Rc<Self>, id: u64, change: impl FnOnce(&mut Reminder)) {
        self.list.update(|l| {
            if let Some(r) = l.iter_mut().find(|r| r.id == id) {
                change(r);
            }
        });
        self.changed();
    }

    pub fn remove(self: &Rc<Self>, id: u64) {
        self.list.update(|l| l.retain(|r| r.id != id));
        self.changed();
    }

    fn changed(self: &Rc<Self>) {
        self.list.update(|l| l.sort_by_key(|r| r.at));
        let path = path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(&*self.list.get()) {
            let _ = std::fs::write(path, json);
        }
        self.arm();
    }

    /// Wakes up at the next one, or within half a minute, which also
    /// catches the clock moving (a suspend, a new time zone).
    fn arm(self: &Rc<Self>) {
        if let Some(id) = self.timer.borrow_mut().take() {
            id.remove();
        }
        if !*self.enabled.get() {
            return;
        }
        let now = Local::now();
        let Some(next) = self.list.get().iter().filter(|r| r.enabled).map(|r| r.at).min() else { return };
        let wait = (next - now).num_milliseconds().clamp(0, 30_000) as u64;
        let weak = Rc::downgrade(self);
        let id = glib::timeout_add_local_once(StdDuration::from_millis(wait + 50), move || {
            if let Some(this) = weak.upgrade() {
                this.timer.borrow_mut().take();
                this.tick();
            }
        });
        *self.timer.borrow_mut() = Some(id);
    }

    fn tick(self: &Rc<Self>) {
        let now = Local::now();
        let due: Vec<Reminder> = self.list.get().iter().filter(|r| r.enabled && r.at <= now).cloned().collect();
        for r in &due {
            self.fire(r, now);
        }
        if !due.is_empty() {
            self.list.update(|l| {
                l.retain(|r| !(r.once && due.iter().any(|d| d.id == r.id)));
                for r in l.iter_mut().filter(|r| due.iter().any(|d| d.id == r.id)) {
                    match r.next_after(now) {
                        Some(next) => r.at = next,
                        None => r.enabled = false,
                    }
                }
            });
            self.changed();
        } else {
            self.arm();
        }
    }

    fn fire(self: &Rc<Self>, r: &Reminder, now: DateTime<Local>) {
        let late = now - r.at > Duration::minutes(10);
        let when = r.at.format("%H:%M").to_string();
        let body = if late {
            tf("perdido, era às {}", when)
        } else if r.alarm {
            tf("alarme · {}", when)
        } else {
            tf("lembrete · {}", when)
        };
        let ring = r.alarm && !late;
        if ring {
            self.stop_ringing();
            *self.ringing.borrow_mut() = sound::ring();
        } else if !late {
            sound::chime();
        }
        let snooze_label = tf("+{} min", self.snooze.get());
        let weak = Rc::downgrade(self);
        let fired = r.clone();
        let done = if r.alarm { t("parar") } else { t("feito") };
        self.notifications.post(
            if r.alarm { "alarm-symbolic" } else { "appointment-soon-symbolic" },
            &r.label(),
            &body,
            &[("snooze", &snooze_label), ("done", done)],
            ring,
            move |action| {
                let Some(this) = weak.upgrade() else { return };
                if fired.alarm {
                    this.stop_ringing();
                }
                if action == "snooze" {
                    let mut again = fired.clone();
                    again.at = Local::now() + Duration::minutes(this.snooze.get() as i64);
                    again.repeat.clear();
                    again.once = true;
                    this.add(again);
                }
            },
        );
    }

    fn stop_ringing(&self) {
        if let Some(mut child) = self.ringing.borrow_mut().take() {
            sound::stop(&mut child);
        }
    }

    /// Adds one from a few typed words; None when they have no time in them.
    pub fn quick(self: &Rc<Self>, text: &str, alarm: Option<bool>) -> Option<Reminder> {
        let q = parse::parse(text, Local::now())?;
        let reminder = Reminder::new(&q.title, q.at, alarm.unwrap_or(q.alarm));
        let id = self.add(reminder);
        self.list.get().iter().find(|r| r.id == id).cloned()
    }
}

/// "today 18:00", "tomorrow 07:30", "fri 27, 09:00".
pub fn when(at: &DateTime<Local>) -> String {
    let today = Local::now().date_naive();
    let day = at.date_naive();
    let time = at.format("%H:%M").to_string();
    if day == today {
        tf("hoje {}", time)
    } else if day == today.succ_opt().unwrap_or(today) {
        tf("amanhã {}", time)
    } else {
        format!("{} · {}", at.format_localized("%a %-d", crate::i18n::locale()), time)
    }
}
