use serde::{Deserialize, Serialize};

use crate::exec;

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Config {
    pub bar: Bar,
    pub weather: Weather,
    pub brightness: Brightness,
    pub notifications: Notifications,
    pub media: Media,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Bar {
    /// "left" or "right".
    pub position: String,
    pub width: u32,
    pub top: Vec<String>,
    pub center: Vec<String>,
    pub bottom: Vec<String>,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            position: "left".into(),
            width: 52,
            top: vec!["logo".into(), "workspaces".into()],
            center: vec!["clock".into()],
            bottom: vec!["tray".into(), "battery".into(), "controls".into()],
        }
    }
}

impl Bar {
    pub fn right(&self) -> bool {
        self.position == "right"
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Weather {
    /// Empty locates by IP.
    pub place: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct Brightness {
    /// Empty picks the first backlight / the first `*kbd_backlight` LED.
    pub screen: String,
    pub keyboard: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Media {
    pub lyrics: bool,
}

impl Default for Media {
    fn default() -> Self {
        Self { lyrics: true }
    }
}

fn path() -> std::path::PathBuf {
    exec::config_dir().join("octopus.toml")
}

pub fn load() -> Config {
    match std::fs::read_to_string(path()) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|err| {
            eprintln!("octopus: {}: {err}", path().display());
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

pub fn save(config: &Config) -> std::io::Result<()> {
    let text = toml::to_string_pretty(config).map_err(std::io::Error::other)?;
    std::fs::write(
        path(),
        format!("# Octopus settings; `octopus settings` edits this file.\n\n{text}"),
    )
}
