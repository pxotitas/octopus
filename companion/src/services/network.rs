use std::collections::HashSet;
use std::rc::Rc;

use gtk::glib;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug, Default)]
pub struct Network {
    pub ssid: String,
    pub signal: u32,
    pub secure: bool,
    pub active: bool,
    pub known: bool,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub enabled: bool,
    pub networks: Vec<Network>,
    pub busy: Option<String>,
    pub error: Option<String>,
}

impl State {
    pub fn active(&self) -> Option<&Network> {
        self.networks.iter().find(|n| n.active)
    }
}

pub struct Wifi {
    pub state: Rc<Store<State>>,
    refresh: Box<dyn Fn()>,
    _events: Option<exec::Stream>,
}

impl Wifi {
    pub fn new() -> Rc<Self> {
        let state = Store::new(State::default());
        let refresh = Rc::new(exec::debounce(250, {
            let state = state.clone();
            move || {
                let state = state.clone();
                glib::spawn_future_local(async move {
                    let fresh = read().await;
                    state.update(|s| {
                        s.enabled = fresh.enabled;
                        s.networks = fresh.networks;
                    });
                });
            }
        }));
        refresh();
        let events = exec::stream("nmcli", &["monitor"], {
            let refresh = refresh.clone();
            move |_| refresh()
        });
        Rc::new(Self {
            state,
            refresh: Box::new(move || refresh()),
            _events: events,
        })
    }

    pub fn refresh(&self) {
        (self.refresh)();
    }

    pub fn rescan(&self) {
        exec::spawn("nmcli", &["device", "wifi", "rescan"]);
    }

    pub fn set_enabled(&self, on: bool) {
        exec::spawn("nmcli", &["radio", "wifi", if on { "on" } else { "off" }]);
        self.state.update(|s| s.enabled = on);
    }

    pub fn connect(self: &Rc<Self>, ssid: &str, password: Option<&str>) {
        let network = self
            .state
            .get()
            .networks
            .iter()
            .find(|n| n.ssid == ssid)
            .cloned();
        let args: Vec<String> = match (&network, password) {
            (Some(n), None) if n.known => {
                vec!["connection".into(), "up".into(), "id".into(), ssid.into()]
            }
            (_, Some(pass)) => ["device", "wifi", "connect", ssid, "password", pass]
                .map(String::from)
                .to_vec(),
            _ => ["device", "wifi", "connect", ssid]
                .map(String::from)
                .to_vec(),
        };
        self.run(ssid, args);
    }

    pub fn disconnect(self: &Rc<Self>, ssid: &str) {
        self.run(
            ssid,
            ["connection", "down", "id", ssid]
                .map(String::from)
                .to_vec(),
        );
    }

    fn run(self: &Rc<Self>, ssid: &str, args: Vec<String>) {
        let this = self.clone();
        let ssid = ssid.to_owned();
        self.state.update(|s| {
            s.busy = Some(ssid.clone());
            s.error = None;
        });
        glib::spawn_future_local(async move {
            let result = exec::run("nmcli", &args).await;
            this.state.update(|s| {
                s.busy = None;
                s.error = result.err().map(|err| short_error(&err));
            });
            this.refresh();
        });
    }
}

fn short_error(err: &str) -> String {
    err.lines()
        .last()
        .unwrap_or(err)
        .trim_start_matches("Error: ")
        .to_owned()
}

/// `nmcli -t` separates fields with ':' and escapes literal ones as '\:'.
fn fields(line: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let mut chars = line.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    out.last_mut().unwrap().push(next);
                }
            }
            ':' => out.push(String::new()),
            _ => out.last_mut().unwrap().push(c),
        }
    }
    out
}

async fn read() -> State {
    let enabled = exec::output("nmcli", &["-t", "-f", "WIFI", "general"])
        .await
        .is_some_and(|s| s.trim() == "enabled");

    let known: HashSet<String> =
        exec::output("nmcli", &["-t", "-f", "NAME,TYPE", "connection", "show"])
            .await
            .unwrap_or_default()
            .lines()
            .map(fields)
            .filter(|f| f.get(1).is_some_and(|t| t.contains("wireless")))
            .map(|f| f[0].clone())
            .collect();

    let list = exec::output(
        "nmcli",
        &[
            "-t",
            "-f",
            "ACTIVE,SSID,SIGNAL,SECURITY",
            "device",
            "wifi",
            "list",
            "--rescan",
            "no",
        ],
    )
    .await
    .unwrap_or_default();

    let mut networks: Vec<Network> = Vec::new();
    for f in list.lines().map(fields) {
        let [active, ssid, signal, security] = &f[..] else {
            continue;
        };
        if ssid.is_empty() {
            continue;
        }
        let network = Network {
            ssid: ssid.clone(),
            signal: signal.parse().unwrap_or(0),
            secure: !security.is_empty() && security != "--",
            active: active == "yes",
            known: known.contains(ssid),
        };
        match networks.iter_mut().find(|n| n.ssid == network.ssid) {
            Some(existing) => {
                existing.active |= network.active;
                existing.signal = existing.signal.max(network.signal);
            }
            None => networks.push(network),
        }
    }
    networks.sort_by(|a, b| {
        b.active
            .cmp(&a.active)
            .then(b.known.cmp(&a.known))
            .then(b.signal.cmp(&a.signal))
    });

    State {
        enabled,
        networks,
        busy: None,
        error: None,
    }
}
