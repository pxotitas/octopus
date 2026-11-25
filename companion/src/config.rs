//! octopus.toml: what can be changed without touching code, with defaults for everything missing.

use serde::{Deserialize, Serialize};

use crate::exec;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    /// "auto" follows LANG; "pt" or "en".
    pub language: String,
    pub bar: Bar,
    pub weather: Weather,
    pub brightness: Brightness,
    pub notifications: Notifications,
    pub media: Media,
    pub dock: Dock,
    pub desktop: Desktop,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "auto".into(),
            bar: Bar::default(),
            weather: Weather::default(),
            brightness: Brightness::default(),
            notifications: Notifications::default(),
            media: Media::default(),
            dock: Dock::default(),
            desktop: Desktop::default(),
        }
    }
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
    /// "floating" (a rounded column with gaps) or "flush" (against the
    /// screen's edge, with concave corners).
    pub style: String,
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            position: "left".into(),
            width: 52,
            top: vec!["logo".into(), "workspaces".into()],
            center: vec!["clock".into()],
            bottom: vec!["tray".into(), "battery".into(), "controls".into()],
            style: "floating".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Dock {
    pub enabled: bool,
    /// "fixed" keeps its space, "autohide" shows on the screen's bottom edge,
    /// "smart" hides only while a window would sit under it.
    pub mode: String,
    /// Desktop ids, in order.
    pub pinned: Vec<String>,
    /// Icon size in pixels.
    pub size: u32,
    /// Icons swell under the pointer, after Velatura's dock.
    pub magnify: bool,
}

impl Default for Dock {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "smart".into(),
            pinned: Vec::new(),
            size: 44,
            magnify: true,
        }
    }
}

/// Widgets on the desktop, under the windows.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Desktop {
    pub enabled: bool,
    /// In order, top to bottom: "weather", "github".
    pub widgets: Vec<String>,
    /// "top-right", "top-left", "bottom-right" or "bottom-left".
    pub position: String,
    /// The github user whose commits to show; empty hides that widget.
    pub github: String,
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            enabled: true,
            widgets: vec!["weather".into(), "github".into()],
            position: "top-right".into(),
            github: String::new(),
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
    /// Moves the lyrics earlier (positive) or later (negative).
    pub lyrics_offset_ms: i64,
}

impl Default for Media {
    fn default() -> Self {
        Self {
            lyrics: true,
            lyrics_offset_ms: 0,
        }
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
