use std::collections::HashSet;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug, Default)]
pub struct Device {
    pub address: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
}

#[derive(Clone, Debug, Default)]
pub struct State {
    pub available: bool,
    pub powered: bool,
    pub scanning: bool,
    pub devices: Vec<Device>,
    pub busy: Option<String>,
    pub error: Option<String>,
}

impl State {
    pub fn connected(&self) -> Option<&Device> {
        self.devices.iter().find(|d| d.connected)
    }
}

pub struct Bluetooth {
    pub state: Rc<Store<State>>,
}

impl Bluetooth {
    pub fn new() -> Rc<Self> {
        let bt = Rc::new(Self {
            state: Store::new(State::default()),
        });
        bt.refresh();
        bt
    }

    pub fn refresh(&self) {
        let state = self.state.clone();
        glib::spawn_future_local(async move {
            let fresh = read().await;
            state.update(|s| {
                s.available = fresh.available;
                s.powered = fresh.powered;
                s.devices = fresh.devices;
            });
        });
    }

    pub fn set_powered(self: &Rc<Self>, on: bool) {
        self.state.update(|s| s.powered = on);
        self.run(
            None,
            vec![vec!["power".into(), if on { "on" } else { "off" }.into()]],
        );
    }

    pub fn scan(self: &Rc<Self>) {
        if self.state.get().scanning {
            return;
        }
        self.state.update(|s| s.scanning = true);
        let this = self.clone();
        glib::spawn_future_local(async move {
            let args: Vec<String> = ["--timeout", "12", "scan", "on"].map(String::from).to_vec();
            let scan = exec::run("bluetoothctl", &args);
            let poll = {
                let this = this.clone();
                glib::timeout_add_local(Duration::from_secs(2), move || {
                    this.refresh();
                    glib::ControlFlow::Continue
                })
            };
            let _ = scan.await;
            poll.remove();
            this.state.update(|s| s.scanning = false);
            this.refresh();
        });
    }

    pub fn toggle(self: &Rc<Self>, address: &str) {
        let Some(device) = self
            .state
            .get()
            .devices
            .iter()
            .find(|d| d.address == address)
            .cloned()
        else {
            return;
        };
        let addr = address.to_owned();
        let steps: Vec<Vec<String>> = if device.connected {
            vec![vec!["disconnect".into(), addr]]
        } else if device.paired {
            vec![vec!["connect".into(), addr]]
        } else {
            vec![
                vec!["pair".into(), addr.clone()],
                vec!["trust".into(), addr.clone()],
                vec!["connect".into(), addr],
            ]
        };
        self.run(Some(address.to_owned()), steps);
    }

    fn run(self: &Rc<Self>, target: Option<String>, steps: Vec<Vec<String>>) {
        let this = self.clone();
        self.state.update(|s| {
            s.busy = target.clone();
            s.error = None;
        });
        glib::spawn_future_local(async move {
            let mut error = None;
            for step in steps {
                if let Err(err) = exec::run("bluetoothctl", &step).await {
                    error = Some(err.lines().last().unwrap_or("falhou").to_owned());
                    break;
                }
            }
            this.state.update(|s| {
                s.busy = None;
                s.error = error;
            });
            this.refresh();
        });
    }
}

fn addresses(list: &str) -> Vec<(String, String)> {
    list.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("Device ")?;
            let (address, name) = rest.split_once(' ').unwrap_or((rest, rest));
            Some((address.to_owned(), name.to_owned()))
        })
        .collect()
}

async fn read() -> State {
    let Some(show) = exec::output("bluetoothctl", &["show"]).await else {
        return State::default();
    };
    let available = show.contains("Controller");
    let powered = show.lines().any(|l| l.trim() == "Powered: yes");

    let all = exec::output("bluetoothctl", &["devices"])
        .await
        .unwrap_or_default();
    let paired: HashSet<String> = addresses(
        &exec::output("bluetoothctl", &["devices", "Paired"])
            .await
            .unwrap_or_default(),
    )
    .into_iter()
    .map(|(a, _)| a)
    .collect();
    let connected: HashSet<String> = addresses(
        &exec::output("bluetoothctl", &["devices", "Connected"])
            .await
            .unwrap_or_default(),
    )
    .into_iter()
    .map(|(a, _)| a)
    .collect();

    let mut devices: Vec<Device> = addresses(&all)
        .into_iter()
        .filter(|(address, name)| paired.contains(address) || name.replace('-', ":") != *address)
        .map(|(address, name)| Device {
            paired: paired.contains(&address),
            connected: connected.contains(&address),
            address,
            name,
        })
        .collect();
    devices.sort_by(|a, b| {
        b.connected
            .cmp(&a.connected)
            .then(b.paired.cmp(&a.paired))
            .then(a.name.cmp(&b.name))
    });

    State {
        available,
        powered,
        scanning: false,
        devices,
        busy: None,
        error: None,
    }
}
