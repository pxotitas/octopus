//! The weather from scripts/weather.py (Open-Meteo), refreshed every so often.

use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use serde::Deserialize;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Hour {
    pub hour: u32,
    pub temperature: i32,
    pub icon: String,
    pub rain: u32,
    pub tomorrow: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Day {
    /// yyyy-mm-dd
    pub date: String,
    /// Days from today: -1 is yesterday.
    pub offset: i32,
    /// 0 is monday.
    pub weekday: u32,
    pub icon: String,
    pub description: String,
    pub high: i32,
    pub low: i32,
    pub rain: u32,
    pub rain_mm: f64,
    pub wind: u32,
    pub gusts: u32,
    pub wind_dir: u32,
    pub uv: f64,
    pub sunrise: String,
    pub sunset: String,
    /// The moon's age, 0 new to 0.5 full and back.
    pub moon: f64,
    pub moon_name: String,
    pub hours: Vec<Hour>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Air {
    pub index: u32,
    pub label: String,
    /// 0 good to 5 extremely poor.
    pub level: u32,
    pub pm25: u32,
    pub pm10: u32,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Reading {
    pub place: String,
    pub region: String,
    pub temperature: i32,
    pub feels_like: i32,
    pub description: String,
    pub icon: String,
    pub is_day: bool,
    pub humidity: u32,
    pub wind: u32,
    pub wind_dir: u32,
    pub gusts: u32,
    pub pressure: u32,
    pub uv: f64,
    pub visibility: f64,
    pub cloud: u32,
    pub precipitation: f64,
    pub low: i32,
    pub high: i32,
    pub sunrise: String,
    pub sunset: String,
    pub moon: f64,
    pub moon_name: String,
    pub air: Option<Air>,
    pub updated: String,
    /// The next 24 hours.
    pub hourly: Vec<Hour>,
    /// A week back to two weeks ahead.
    pub days: Vec<Day>,
}

impl Reading {
    pub fn day(&self, offset: i32) -> Option<&Day> {
        self.days.iter().find(|d| d.offset == offset)
    }

    pub fn first_offset(&self) -> i32 {
        self.days.first().map_or(0, |d| d.offset)
    }

    pub fn last_offset(&self) -> i32 {
        self.days.last().map_or(0, |d| d.offset)
    }
}

#[derive(Deserialize)]
struct Answer {
    available: bool,
}

pub struct Weather {
    pub reading: Rc<Store<Option<Reading>>>,
}

impl Weather {
    pub fn new(place: &str) -> Rc<Self> {
        let weather = Rc::new(Self {
            reading: Store::new(None),
        });
        let place = place.to_owned();
        let fetch = {
            let store = weather.reading.clone();
            move || {
                let store = store.clone();
                let place = place.clone();
                glib::spawn_future_local(async move {
                    let script = exec::script("weather.py");
                    let Some(out) = exec::output("python3", &[&script, &place]).await else {
                        return;
                    };
                    let available = serde_json::from_str::<Answer>(&out).is_ok_and(|a| a.available);
                    if !available {
                        return;
                    }
                    match serde_json::from_str::<Reading>(&out) {
                        Ok(reading) => store.set(Some(reading)),
                        Err(err) => eprintln!("octopus: weather: {err}"),
                    }
                });
            }
        };
        fetch();
        glib::timeout_add_local(Duration::from_secs(20 * 60), move || {
            fetch();
            glib::ControlFlow::Continue
        });
        weather
    }
}

/// A compass point for a wind direction in degrees, as an arrow glyph's turn.
pub fn compass(degrees: u32) -> &'static str {
    const PT: [&str; 8] = ["n", "ne", "l", "se", "s", "so", "o", "no"];
    const EN: [&str; 8] = ["n", "ne", "e", "se", "s", "sw", "w", "nw"];
    let index = (((degrees % 360) as f64 + 22.5) / 45.0) as usize % 8;
    if crate::i18n::english() {
        EN[index]
    } else {
        PT[index]
    }
}
