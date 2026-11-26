//! A github user's contribution calendar, read from the profile's public
//! contributions page: no token, a year of days, each with a level from 0
//! (none) to 4 and how many contributions it had.

use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug)]
pub struct Day {
    pub date: chrono::NaiveDate,
    pub level: u8,
    pub count: u32,
}

pub struct Contributions {
    /// Oldest first.
    pub days: Rc<Store<Option<Vec<Day>>>>,
}

impl Contributions {
    pub fn new(user: &str) -> Rc<Self> {
        let service = Rc::new(Self {
            days: Store::new(None),
        });
        if user.is_empty() {
            return service;
        }
        let url = format!("https://github.com/users/{user}/contributions");
        let fetch = {
            let store = service.days.clone();
            move || {
                let (store, url) = (store.clone(), url.clone());
                glib::spawn_future_local(async move {
                    let html = exec::output("curl", &["-sS", "--fail", "--max-time", "20", &url]).await;
                    match html.map(|h| parse(&h)) {
                        Some(days) if !days.is_empty() => store.set(Some(days)),
                        _ => eprintln!("octopus: no contributions from {url}"),
                    }
                });
            }
        };
        fetch();
        glib::timeout_add_local(Duration::from_secs(30 * 60), move || {
            fetch();
            glib::ControlFlow::Continue
        });
        service
    }
}

fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let len = tag[start..].find('"')?;
    Some(&tag[start..start + len])
}

/// Days from `<td data-date data-level id>` cells, with counts from the
/// `<tool-tip for=id>` that describes each.
fn parse(html: &str) -> Vec<Day> {
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for chunk in html.split("<tool-tip").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let (tag, rest) = chunk.split_at(end);
        let Some(id) = attribute(tag, "for") else { continue };
        let text = rest[1..].split('<').next().unwrap_or("");
        let count = text
            .split_whitespace()
            .next()
            .and_then(|n| n.replace(',', "").parse().ok())
            .unwrap_or(0);
        counts.insert(id, count);
    }

    let mut days: Vec<Day> = html
        .split("<td")
        .skip(1)
        .filter_map(|chunk| {
            let tag = &chunk[..chunk.find('>')?];
            let date = chrono::NaiveDate::parse_from_str(attribute(tag, "data-date")?, "%Y-%m-%d").ok()?;
            let level = attribute(tag, "data-level")?.parse().ok()?;
            let count = attribute(tag, "id").and_then(|id| counts.get(id)).copied().unwrap_or(0);
            Some(Day { date, level, count })
        })
        .collect();
    days.sort_by_key(|d| d.date);
    days
}

/// Contributions in the whole calendar, and the run of days with some that
/// ends today (or yesterday, when today has none yet).
pub fn totals(days: &[Day]) -> (u32, u32) {
    let total = days.iter().map(|d| d.count).sum();
    let mut streak = 0;
    let today = chrono::Local::now().date_naive();
    let mut iter = days.iter().rev().skip_while(|d| d.date > today).peekable();
    if iter.peek().is_some_and(|d| d.date == today && d.count == 0) {
        iter.next();
    }
    for d in iter {
        if d.count == 0 && d.level == 0 {
            break;
        }
        streak += 1;
    }
    (total, streak)
}
