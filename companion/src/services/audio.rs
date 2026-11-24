//! PulseAudio/PipeWire through pactl: volumes, mutes and devices, followed with `pactl subscribe`.

use std::rc::Rc;

use gtk::glib;
use serde_json::Value;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Device {
    pub name: String,
    pub description: String,
    pub volume: u32,
    pub muted: bool,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub sinks: Vec<Device>,
    pub sources: Vec<Device>,
    pub default_sink: String,
    pub default_source: String,
}

impl State {
    pub fn sink(&self) -> Option<&Device> {
        self.sinks
            .iter()
            .find(|d| d.name == self.default_sink)
            .or(self.sinks.first())
    }

    pub fn source(&self) -> Option<&Device> {
        self.sources
            .iter()
            .find(|d| d.name == self.default_source)
            .or(self.sources.first())
    }
}

#[derive(Clone, Copy)]
pub enum Kind {
    Sink,
    Source,
}

impl Kind {
    fn target(self) -> &'static str {
        match self {
            Kind::Sink => "@DEFAULT_SINK@",
            Kind::Source => "@DEFAULT_SOURCE@",
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Kind::Sink => "sink",
            Kind::Source => "source",
        }
    }
}

pub struct Audio {
    pub state: Rc<Store<State>>,
    _events: Option<exec::Stream>,
}

impl Audio {
    pub fn new() -> Rc<Self> {
        let state = Store::new(State::default());
        let refresh = exec::debounce(60, {
            let state = state.clone();
            move || {
                let state = state.clone();
                glib::spawn_future_local(async move {
                    if let Some(fresh) = read().await {
                        state.set(fresh);
                    }
                });
            }
        });
        refresh();
        let events = exec::stream("pactl", &["subscribe"], move |line| {
            if line.contains("sink") || line.contains("source") || line.contains("server") {
                refresh();
            }
        });
        Rc::new(Self {
            state,
            _events: events,
        })
    }

    pub fn set_volume(&self, kind: Kind, percent: u32) {
        let value = format!("{}%", percent.min(150));
        exec::spawn(
            "pactl",
            &[
                &format!("set-{}-volume", kind.noun()),
                kind.target(),
                &value,
            ],
        );
    }

    pub fn step(&self, kind: Kind, delta: i32) {
        let current = match kind {
            Kind::Sink => self.state.get().sink().map(|d| d.volume),
            Kind::Source => self.state.get().source().map(|d| d.volume),
        }
        .unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, 100) as u32;
        self.set_volume(kind, next);
        self.state.update(|state| {
            let name = match kind {
                Kind::Sink => state.default_sink.clone(),
                Kind::Source => state.default_source.clone(),
            };
            let list = match kind {
                Kind::Sink => &mut state.sinks,
                Kind::Source => &mut state.sources,
            };
            if let Some(device) = list.iter_mut().find(|d| d.name == name) {
                device.volume = next;
            }
        });
    }

    pub fn toggle_mute(&self, kind: Kind) {
        exec::spawn(
            "pactl",
            &[
                &format!("set-{}-mute", kind.noun()),
                kind.target(),
                "toggle",
            ],
        );
    }

    pub fn set_default(&self, kind: Kind, name: &str) {
        exec::spawn("pactl", &[&format!("set-default-{}", kind.noun()), name]);
    }
}

async fn read() -> Option<State> {
    let sinks = exec::output("pactl", &["-f", "json", "list", "sinks"]).await?;
    let sources = exec::output("pactl", &["-f", "json", "list", "sources"]).await?;
    let default_sink = exec::output("pactl", &["get-default-sink"])
        .await
        .unwrap_or_default();
    let default_source = exec::output("pactl", &["get-default-source"])
        .await
        .unwrap_or_default();
    Some(State {
        sinks: devices(&sinks, false),
        sources: devices(&sources, true),
        default_sink: default_sink.trim().to_owned(),
        default_source: default_source.trim().to_owned(),
    })
}

fn devices(json: &str, skip_monitors: bool) -> Vec<Device> {
    let Ok(Value::Array(list)) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    list.iter()
        .filter(|d| {
            !(skip_monitors
                && d["monitor_of_sink"]
                    .as_str()
                    .is_some_and(|s| !s.is_empty() && s != "n/a"))
        })
        .filter_map(|d| {
            let volume = d["volume"]
                .as_object()
                .and_then(|channels| channels.values().next())
                .and_then(|channel| channel["value_percent"].as_str())
                .and_then(|p| p.trim_end_matches('%').trim().parse().ok())
                .unwrap_or(0);
            Some(Device {
                name: d["name"].as_str()?.to_owned(),
                description: d["description"].as_str().unwrap_or_default().to_owned(),
                volume,
                muted: d["mute"].as_bool().unwrap_or(false),
            })
        })
        .collect()
}
