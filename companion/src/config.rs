//! octopus.toml: what can be changed without touching code, with defaults for everything missing.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::exec;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// "auto" follows LANG; "pt" or "en".
    pub language: String,
    pub look: Look,
    pub bar: Bar,
    pub dash: Dash,
    pub weather: Weather,
    pub calendar: CalendarConfig,
    pub brightness: Brightness,
    pub notifications: Notifications,
    pub media: Media,
    pub lock: Lock,
    pub dock: Dock,
    pub desktop: Desktop,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: "auto".into(),
            look: Look::default(),
            bar: Bar::default(),
            dash: Dash::default(),
            weather: Weather::default(),
            calendar: CalendarConfig::default(),
            brightness: Brightness::default(),
            notifications: Notifications::default(),
            media: Media::default(),
            lock: Lock::default(),
            dock: Dock::default(),
            desktop: Desktop::default(),
        }
    }
}

impl Config {
    /// Whether going from `self` to `next` needs the daemon to start over:
    /// everything but the bar, the colors and the lyrics' timing is read once.
    pub fn restart_for(&self, next: &Config) -> bool {
        let mut a = self.clone();
        let mut b = next.clone();
        for c in [&mut a, &mut b] {
            c.bar = Bar::default();
            c.look = Look::default();
            c.media.lyrics_offset_ms = 0;
            c.dock.pinned.clear();
        }
        a != b
    }
}

/// The colors matugen makes from the wallpaper.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Look {
    /// matugen's scheme, without the "scheme-": "tonal-spot", "expressive",
    /// "vibrant", "fidelity", "content", "neutral", "monochrome",
    /// "rainbow", "fruit-salad".
    pub scheme: String,
    /// -1 (softer) to 1 (stronger).
    pub contrast: f64,
}

impl Default for Look {
    fn default() -> Self {
        Self {
            scheme: "tonal-spot".into(),
            contrast: 0.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Bar {
    /// "left" or "right".
    pub position: String,
    pub width: u32,
    /// Pixels between modules.
    pub spacing: u32,
    /// The layer it sits on: "top", "overlay" (over fullscreen windows too)
    /// or "bottom" (under floating windows).
    pub layer: String,
    /// Whether windows keep out of its way.
    pub exclusive: bool,
    /// Gaps to the screen's edges when floating, in pixels.
    pub margin_top: u32,
    pub margin_bottom: u32,
    pub margin_edge: u32,
    pub top: Vec<String>,
    pub center: Vec<String>,
    pub bottom: Vec<String>,
    /// "floating" (a rounded column with gaps) or "flush" (against the
    /// screen's edge, with concave corners).
    pub style: String,
    /// How each module looks and acts, by module name.
    pub modules: BTreeMap<String, Module>,
    /// Modules made in the editor; placed by their id ("user-1"…).
    pub custom: Vec<Custom>,
}

/// A module made in the editor: an icon, and optionally a command whose
/// output shows under it.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Custom {
    pub id: String,
    pub name: String,
    /// A Material Symbols name.
    pub icon: String,
    /// Runs every `interval` seconds; its output is the value. Empty shows
    /// only the icon.
    pub exec: String,
    pub interval: u32,
    /// Runs on a click.
    pub click: String,
}

impl Default for Custom {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "meu módulo".into(),
            icon: "star".into(),
            exec: String::new(),
            interval: 30,
            click: String::new(),
        }
    }
}

impl Default for Bar {
    fn default() -> Self {
        Self {
            position: "left".into(),
            width: 52,
            spacing: 4,
            layer: "top".into(),
            exclusive: true,
            margin_top: 8,
            margin_bottom: 8,
            margin_edge: 8,
            top: vec!["logo".into(), "workspaces".into()],
            center: vec!["clock".into()],
            bottom: vec!["tray".into(), "battery".into(), "controls".into()],
            style: "floating".into(),
            modules: BTreeMap::new(),
            custom: Vec::new(),
        }
    }
}

impl Bar {
    pub fn right(&self) -> bool {
        self.position == "right"
    }

    /// The options of a module, or its defaults.
    pub fn module(&self, id: &str) -> Module {
        self.modules.get(id).cloned().unwrap_or_else(|| Module::default_for(id))
    }

    pub fn placed(&self) -> impl Iterator<Item = &String> {
        self.top.iter().chain(&self.center).chain(&self.bottom)
    }

    pub fn custom(&self, id: &str) -> Option<&Custom> {
        self.custom.iter().find(|c| c.id == id)
    }

    /// A fresh id for a module made in the editor.
    pub fn next_custom_id(&self) -> String {
        let n = (1..).find(|n| self.custom(&format!("user-{n}")).is_none()).unwrap_or(1);
        format!("user-{n}")
    }
}

/// One module's options; each module reads the ones that make sense for it.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Module {
    /// "none", "tonal", "primary" or "surface".
    pub background: String,
    /// "round", "soft" or "square", an M3 shape ("cookie9", "flower"…),
    /// and for the clock also "bun".
    pub shape: String,
    /// Module-specific: the logo's glyph, the clock's "24h"/"12h", the
    /// workspaces' "dots"/"numbers", the controls' "drawer"/"open".
    pub variant: String,
    /// Shows the value under the icon (battery, cpu, memory…).
    pub text: bool,
    /// What a click does, for the logo: "launcher", "spotlight" or "dash".
    pub action: String,
    /// The controls' buttons, in order.
    pub items: Vec<String>,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            background: "none".into(),
            shape: "soft".into(),
            variant: String::new(),
            text: false,
            action: String::new(),
            items: Vec::new(),
        }
    }
}

impl Module {
    pub fn default_for(id: &str) -> Self {
        let base = Self::default();
        match id {
            "logo" => Self { variant: "arch".into(), action: "launcher".into(), ..base },
            "workspaces" => Self { variant: "dots".into(), ..base },
            "clock" => Self { background: "primary".into(), shape: "bun".into(), variant: "24h".into(), ..base },
            "battery" => Self { text: true, ..base },
            "controls" => Self {
                variant: "drawer".into(),
                items: ["control-center", "network", "bluetooth", "volume", "power"].map(String::from).to_vec(),
                ..base
            },
            "cpu" | "memory" | "temperature" => Self { text: true, ..base },
            _ => base,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Dash {
    /// The tab it opens on: "overview", "media", "weather" or "wallpapers".
    pub page: String,
}

impl Default for Dash {
    fn default() -> Self {
        Self { page: "overview".into() }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Lock {
    /// The line being sung, under the clock.
    pub lyrics: bool,
}

impl Default for Lock {
    fn default() -> Self {
        Self { lyrics: true }
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
    /// "weather", "contributions", "clock", "media", "calendar". Placed and
    /// put away from the desktop's "arrange widgets"; where they sit is kept
    /// in ~/.local/state/octopus.
    pub widgets: Vec<String>,
    /// The github user whose contributions to show; empty hides that widget.
    pub github: String,
    /// "github" for the profile's greens, "theme" for the wallpaper's color.
    pub contributions_colors: String,
}

impl Default for Desktop {
    fn default() -> Self {
        Self {
            enabled: true,
            widgets: vec!["weather".into(), "contributions".into()],
            github: String::new(),
            contributions_colors: "github".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Weather {
    /// Empty locates by IP.
    pub place: String,
    /// "celsius" or "fahrenheit".
    pub unit: String,
}

impl Default for Weather {
    fn default() -> Self {
        Self {
            place: String::new(),
            unit: "celsius".into(),
        }
    }
}

/// Calendars to show, by their iCal address: Google Calendar's "secret
/// address in iCal format" (Settings → the calendar → Integrate calendar),
/// or any other .ics link.
#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(default)]
pub struct CalendarConfig {
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(default)]
pub struct Brightness {
    /// Empty picks the first backlight / the first `*kbd_backlight` LED.
    pub screen: String,
    pub keyboard: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Notifications {
    pub timeout_ms: u32,
    pub max_popups: usize,
    /// "top-right", "top-left", "bottom-right" or "bottom-left".
    pub position: String,
}

impl Default for Notifications {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            max_popups: 3,
            position: "top-right".into(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(default)]
pub struct Media {
    /// Lyrics in the dash.
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

/// What theme.sh reads for matugen: the scheme and the contrast.
pub fn write_look(look: &Look) {
    let dir = std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| gtk::glib::home_dir().join(".local/state"))
        .join("octopus");
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(dir.join("scheme"), &look.scheme);
    let _ = std::fs::write(dir.join("contrast"), format!("{:.2}", look.contrast));
}
