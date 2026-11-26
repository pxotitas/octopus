//! Events from iCal addresses (scripts/ical.py), refreshed every quarter
//! hour. Google Calendar's "secret address in iCal format" works without
//! any account setup; so do Outlook's, Proton's and Nextcloud's.

use std::rc::Rc;
use std::time::Duration;

use chrono::{DateTime, FixedOffset, Local, NaiveDate};
use gtk::glib;
use serde::Deserialize;

use crate::exec;
use crate::store::Store;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub title: String,
    pub start: DateTime<FixedOffset>,
    pub end: DateTime<FixedOffset>,
    pub all_day: bool,
    pub location: String,
    pub calendar: String,
    pub index: usize,
}

impl Event {
    pub fn day(&self) -> NaiveDate {
        self.start.with_timezone(&Local).date_naive()
    }

    /// Whether it touches `day` (all-day events end at the next midnight).
    pub fn on(&self, day: NaiveDate) -> bool {
        let first = self.day();
        let last = if self.all_day {
            (self.end - chrono::Duration::seconds(1)).with_timezone(&Local).date_naive()
        } else {
            self.end.with_timezone(&Local).date_naive()
        };
        first <= day && day <= last.max(first)
    }

    /// "14:00" or "dia todo".
    pub fn when(&self) -> String {
        if self.all_day {
            crate::i18n::t("dia todo").to_owned()
        } else {
            self.start.with_timezone(&Local).format("%H:%M").to_string()
        }
    }
}

#[derive(Deserialize, Default)]
struct Answer {
    events: Vec<Event>,
    errors: Vec<String>,
}

pub struct Calendar {
    /// None until the first answer.
    pub events: Rc<Store<Option<Vec<Event>>>>,
    pub errors: Rc<Store<Vec<String>>>,
    urls: Vec<String>,
}

impl Calendar {
    pub fn new(urls: &[String]) -> Rc<Self> {
        let calendar = Rc::new(Self {
            events: Store::new(None),
            errors: Store::new(Vec::new()),
            urls: urls.iter().filter(|u| !u.trim().is_empty()).cloned().collect(),
        });
        if calendar.urls.is_empty() {
            return calendar;
        }
        calendar.refresh();
        let weak = Rc::downgrade(&calendar);
        glib::timeout_add_local(Duration::from_secs(15 * 60), move || match weak.upgrade() {
            Some(c) => {
                c.refresh();
                glib::ControlFlow::Continue
            }
            None => glib::ControlFlow::Break,
        });
        calendar
    }

    pub fn enabled(&self) -> bool {
        !self.urls.is_empty()
    }

    pub fn refresh(&self) {
        let (events, errors) = (self.events.clone(), self.errors.clone());
        let mut args = vec![exec::script("ical.py")];
        args.extend(self.urls.iter().cloned());
        glib::spawn_future_local(async move {
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            let Some(out) = exec::output("python3", &refs).await else { return };
            match serde_json::from_str::<Answer>(&out) {
                Ok(answer) => {
                    errors.set(answer.errors);
                    events.set(Some(answer.events));
                }
                Err(err) => eprintln!("octopus: calendar: {err}"),
            }
        });
    }

    /// The events touching `day`, all-day first.
    pub fn on(&self, day: NaiveDate) -> Vec<Event> {
        let events = self.events.get();
        let mut found: Vec<Event> = events.iter().flatten().filter(|e| e.on(day)).cloned().collect();
        found.sort_by_key(|e| (!e.all_day, e.start));
        found
    }

    /// What is still ahead, from now on, at most `count`.
    pub fn upcoming(&self, count: usize) -> Vec<Event> {
        let now = Local::now();
        let events = self.events.get();
        events.iter().flatten().filter(|e| e.end > now).take(count).cloned().collect()
    }
}
