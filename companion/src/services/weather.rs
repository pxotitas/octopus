use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use serde::Deserialize;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug, Deserialize)]
pub struct Hour {
    pub hour: u32,
    pub temperature: i32,
    pub icon: String,
    pub rain: u32,
    pub tomorrow: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub place: String,
    pub temperature: i32,
    pub feels_like: i32,
    pub description: String,
    pub icon: String,
    pub humidity: u32,
    pub wind: u32,
    pub low: i32,
    pub high: i32,
    pub hourly: Vec<Hour>,
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
                    let out = exec::output("python3", &[&script, &place]).await;
                    if let Some(reading) =
                        out.and_then(|o| serde_json::from_str::<Reading>(&o).ok())
                    {
                        store.set(Some(reading));
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
