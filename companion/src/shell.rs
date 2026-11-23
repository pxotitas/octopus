use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

use crate::config;
use crate::exec;
use crate::icons;
use crate::panels::{
    self, clipboard::Clipboard, dash::Dash, osd::Osd, settings::Settings, toasts::Toasts,
    tray::TrayPanel,
};
use crate::popup::{self, Place, Popup};
use crate::services::audio::Kind;
use crate::services::Services;

pub struct Shell {
    app: gtk::Application,
    config: Rc<std::cell::RefCell<config::Config>>,
    settings: std::cell::RefCell<Option<Rc<Settings>>>,
    services: Rc<Services>,
    popups: HashMap<&'static str, Rc<Popup>>,
    backdrop: gtk::Window,
    osd: Rc<Osd>,
    lock: Rc<crate::lock::Lock>,
    _toasts: Rc<Toasts>,
}

impl Shell {
    pub fn new(app: &gtk::Application) -> Rc<Self> {
        crate::theme::install();
        let config = config::load();
        crate::bar::ensure(&config.bar);
        let services = Services::new(&config);

        Rc::new_cyclic(|weak: &std::rc::Weak<Shell>| {
            let open: Rc<dyn Fn(&str)> = {
                let weak = weak.clone();
                Rc::new(move |name: &str| {
                    if let Some(shell) = weak.upgrade() {
                        shell.handle(&format!("open {name}"));
                    }
                })
            };

            let cc = panels::control_center::build(&services, open.clone());
            let dash = Dash::new(&services, config.media.lyrics, {
                let open = open.clone();
                move || open("wallpapers")
            });

            let mut popups = HashMap::new();
            popups.insert(
                "control-center",
                Popup::new(app, "control-center", Place::BarBottom, true, &cc),
            );
            popups.insert(
                "dash",
                Popup::new(app, "dash", Place::BarMiddle, true, &dash.root),
            );
            popups.insert(
                "wifi",
                Popup::new(
                    app,
                    "wifi",
                    Place::BarBottom,
                    true,
                    &panels::wifi::build(&services.wifi),
                ),
            );
            popups.insert(
                "bluetooth",
                Popup::new(
                    app,
                    "bluetooth",
                    Place::BarBottom,
                    true,
                    &panels::bluetooth::build(&services.bluetooth),
                ),
            );
            popups.insert(
                "audio",
                Popup::new(
                    app,
                    "audio",
                    Place::BarBottom,
                    true,
                    &panels::audio::build(&services.audio),
                ),
            );

            let close: Rc<dyn Fn()> = {
                let weak = weak.clone();
                Rc::new(move || {
                    if let Some(shell) = weak.upgrade() {
                        shell.close_all();
                    }
                })
            };
            let tray = TrayPanel::new(&services.tray, close.clone());
            popups.insert(
                "tray",
                Popup::new(app, "tray", Place::BarBottom, true, &tray.root),
            );
            let clipboard = Clipboard::new(close.clone());
            popups.insert(
                "clipboard",
                Popup::new(app, "clipboard", Place::Center, true, &clipboard.root),
            );
            let power = panels::power::build({
                let weak = weak.clone();
                Rc::new(move |action: &str| {
                    if let Some(shell) = weak.upgrade() {
                        shell.power(action);
                    }
                })
            });
            popups.insert(
                "power",
                Popup::new(app, "power", Place::Center, true, &power),
            );
            for popup in popups.values() {
                popup.set_side(config.bar.right());
            }

            services
                .tray
                .items
                .subscribe(|_| exec::spawn("pkill", &["-RTMIN+8", "-x", "waybar"]));

            let backdrop = popup::backdrop(app, {
                let weak = weak.clone();
                move || {
                    if let Some(shell) = weak.upgrade() {
                        shell.close_all();
                    }
                }
            });

            for (name, popup) in &popups {
                let s = services.clone();
                let weak = weak.clone();
                let dash = dash.clone();
                let tray = tray.clone();
                let clipboard = clipboard.clone();
                let name = *name;
                popup.on_visibility(move |open| {
                    match name {
                        "control-center" => {
                            s.cava.watch(open);
                            s.media.watch(open);
                            if open {
                                let s = s.clone();
                                glib::timeout_add_local_once(
                                    Duration::from_millis(320),
                                    move || {
                                        s.wifi.refresh();
                                        s.bluetooth.refresh();
                                        s.screen.refresh();
                                        s.keyboard.refresh();
                                    },
                                );
                            }
                        }
                        "dash" => {
                            s.media.watch(open);
                            dash.set_open(open);
                        }
                        "wifi" if open => {
                            let s = s.clone();
                            glib::timeout_add_local_once(Duration::from_millis(320), move || {
                                s.wifi.refresh();
                                s.wifi.rescan();
                            });
                        }
                        "bluetooth" if open => s.bluetooth.refresh(),
                        "tray" if open => tray.reset(),
                        "clipboard" if open => clipboard.load(),
                        _ => {}
                    }
                    if let Some(shell) = weak.upgrade() {
                        shell.sync_backdrop();
                    }
                });
            }

            let osd = Osd::new(app);
            let toasts = Toasts::new(
                app,
                &services.notifications,
                config.notifications.timeout_ms,
                config.notifications.max_popups,
            );

            for name in ["control-center", "dash"] {
                if let Some(popup) = popups.get(name) {
                    let popup = popup.clone();
                    glib::timeout_add_local_once(Duration::from_millis(600), move || popup.warm());
                }
            }

            let lock = crate::lock::Lock::new(app, &services, config.media.lyrics);

            Shell {
                app: app.clone(),
                config: Rc::new(std::cell::RefCell::new(config.clone())),
                settings: std::cell::RefCell::new(None),
                services,
                popups,
                backdrop,
                osd,
                lock,
                _toasts: toasts,
            }
        })
    }

    fn sync_backdrop(&self) {
        let any = self.popups.values().any(|p| p.is_open());
        self.backdrop.set_visible(any);
    }

    fn close_all(&self) {
        for popup in self.popups.values() {
            popup.hide();
        }
    }

    fn toggle(&self, name: &str) -> String {
        let Some(popup) = self.popups.get(name) else {
            return format!("unknown panel: {name}");
        };
        let opening = !popup.is_open();
        self.close_all();
        if opening {
            popup.show();
        }
        String::new()
    }

    pub fn handle(&self, command: &str) -> String {
        let words: Vec<&str> = command.split_whitespace().collect();
        let audio = &self.services.audio;
        match words.as_slice() {
            ["toggle", name] => self.toggle(alias(name)),
            ["settings"] => {
                self.close_all();
                self.open_settings();
                String::new()
            }
            ["tray-status"] => {
                let count = self
                    .services
                    .tray
                    .items
                    .get()
                    .iter()
                    .filter(|i| !i.passive)
                    .count();
                if count == 0 {
                    r#"{"text": ""}"#.to_owned()
                } else {
                    format!(r#"{{"text": "apps", "tooltip": "{count} na bandeja"}}"#)
                }
            }
            ["open", "wallpapers"] => {
                self.close_all();
                exec::spawn(&exec::script("wallpaper.sh"), &[]);
                String::new()
            }
            ["open", name] => {
                let name = alias(name);
                if self.popups.get(name).is_some_and(|p| p.is_open()) {
                    return String::new();
                }
                self.toggle(name)
            }
            ["lock"] => {
                self.close_all();
                self.lock.start();
                String::new()
            }
            ["close"] => {
                self.close_all();
                String::new()
            }
            ["volume", dir @ ("up" | "down")] => {
                audio.step(Kind::Sink, if *dir == "up" { 5 } else { -5 });
                self.show_volume();
                String::new()
            }
            ["volume", "mute"] => {
                audio.toggle_mute(Kind::Sink);
                let (volume, muted) = audio
                    .state
                    .get()
                    .sink()
                    .map(|d| (d.volume, !d.muted))
                    .unwrap_or((0, true));
                self.osd
                    .show(icons::volume(volume, muted), if muted { 0 } else { volume });
                String::new()
            }
            ["mic", "mute"] => {
                audio.toggle_mute(Kind::Source);
                let (volume, muted) = audio
                    .state
                    .get()
                    .source()
                    .map(|d| (d.volume, !d.muted))
                    .unwrap_or((0, true));
                self.osd.show(
                    if muted { "mic_off" } else { "mic" },
                    if muted { 0 } else { volume },
                );
                String::new()
            }
            [which @ ("brightness" | "kbd"), dir @ ("up" | "down")] => {
                let (light, icon) = if *which == "brightness" {
                    (&self.services.screen, "brightness_6")
                } else {
                    (&self.services.keyboard, "keyboard")
                };
                light.step(if *dir == "up" { 5 } else { -5 });
                if let Some(p) = *light.percent.get() {
                    self.osd.show(icon, p);
                }
                String::new()
            }
            ["dnd"] => {
                let on = !*self.services.notifications.dnd.get();
                self.services.notifications.dnd.set(on);
                if on { "on" } else { "off" }.to_owned()
            }
            ["reload"] => String::new(),
            _ => format!("unknown command: {command}"),
        }
    }

    fn power(&self, action: &str) {
        self.close_all();
        match action {
            "lock" => self.lock.start(),
            "suspend" => exec::spawn("systemctl", &["suspend"]),
            "logout" => exec::spawn("niri", &["msg", "action", "quit", "--skip-confirmation"]),
            "reboot" => exec::spawn("systemctl", &["reboot"]),
            "poweroff" => exec::spawn("systemctl", &["poweroff"]),
            _ => {}
        }
    }

    fn open_settings(&self) {
        let existing = self.settings.borrow().clone();
        let settings = match existing {
            Some(s) => s,
            None => {
                let popups: Vec<Rc<Popup>> = self.popups.values().cloned().collect();
                let current = self.config.clone();
                let apply: Rc<dyn Fn(config::Config)> = Rc::new(move |next: config::Config| {
                    if let Err(err) = config::save(&next) {
                        eprintln!("octopus: cannot save settings: {err}");
                    }
                    crate::bar::apply(&next.bar);
                    for popup in &popups {
                        popup.set_side(next.bar.right());
                    }
                    *current.borrow_mut() = next;
                });
                let s = Settings::new(
                    &self.app,
                    &self.services,
                    self.config.borrow().clone(),
                    apply,
                );
                *self.settings.borrow_mut() = Some(s.clone());
                s
            }
        };
        settings.present();
    }

    fn show_volume(&self) {
        if let Some(d) = self.services.audio.state.get().sink() {
            self.osd.show(icons::volume(d.volume, d.muted), d.volume);
        }
    }
}

fn alias(name: &str) -> &str {
    match name {
        "cc" | "control-center" | "controlcenter" => "control-center",
        "sound" | "volume" => "audio",
        "bt" => "bluetooth",
        "network" => "wifi",
        other => other,
    }
}

/// Calls `f` now and at the start of every minute.
pub fn every_minute(f: impl Fn(chrono::DateTime<chrono::Local>) + 'static) {
    let f = Rc::new(f);
    f(chrono::Local::now());
    let seconds = chrono::Timelike::second(&chrono::Local::now());
    glib::timeout_add_local_once(Duration::from_secs(60 - seconds as u64), move || {
        f(chrono::Local::now());
        glib::timeout_add_local(Duration::from_secs(60), move || {
            f(chrono::Local::now());
            glib::ControlFlow::Continue
        });
    });
}

/// Like `every_minute`, but stops once `owner` is gone.
pub fn every_minute_for(
    owner: &impl IsA<gtk::Widget>,
    f: impl Fn(chrono::DateTime<chrono::Local>) + 'static,
) {
    let owner = owner.upcast_ref::<gtk::Widget>().downgrade();
    f(chrono::Local::now());
    let seconds = chrono::Timelike::second(&chrono::Local::now());
    glib::timeout_add_local_once(Duration::from_secs(60 - seconds as u64), move || {
        if owner.upgrade().is_none() {
            return;
        }
        f(chrono::Local::now());
        glib::timeout_add_local(Duration::from_secs(60), move || {
            if owner.upgrade().is_none() {
                return glib::ControlFlow::Break;
            }
            f(chrono::Local::now());
            glib::ControlFlow::Continue
        });
    });
}
