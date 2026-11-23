use serde::Deserialize;

use crate::exec;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub weather: Weather,
    pub brightness: Brightness,
    pub notifications: Notifications,
    pub media: Media,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Weather {
    /// Empty locates by IP.
    pub place: String,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Brightness {
    /// Empty picks the first backlight / the first `*kbd_backlight` LED.
    pub screen: String,
    pub keyboard: String,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Notifications {
    pub timeout_ms: u32,
    pub max_popups: usize,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_popups: 3,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Media {
    pub lyrics: bool,
}

impl Default for Media {
    fn default() -> Self {
        Self { lyrics: true }
    }
}

pub fn load() -> Config {
    let path = exec::config_dir().join("octopus.toml");
    match std::fs::read_to_string(&path) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|err| {
            eprintln!("octopus: {}: {err}", path.display());
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}
