//! Every module the bar can hold: what the settings say about it, what it
//! lets you change, and the Waybar config and CSS it becomes.

use serde_json::{json, Map, Value};

use crate::config::{Bar, Module};
use crate::i18n::t;

#[derive(Clone, Copy, PartialEq)]
pub enum Group {
    Essentials,
    System,
    Media,
    Shortcuts,
}

pub struct Info {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub about: &'static str,
    pub group: Group,
}

const fn info(id: &'static str, name: &'static str, icon: &'static str, about: &'static str, group: Group) -> Info {
    Info { id, name, icon, about, group }
}

pub const CATALOG: [Info; 15] = [
    info("logo", "logo", "change_history", "abre o launcher, o spotlight ou o dash", Group::Essentials),
    info("workspaces", "workspaces", "more_vert", "os workspaces do niri, em pontos ou números", Group::Essentials),
    info("clock", "relógio", "schedule", "horas sobre minutos; abre o dash", Group::Essentials),
    info("tray", "bandeja", "keyboard_arrow_down", "os ícones dos apps em segundo plano", Group::Essentials),
    info("battery", "bateria", "battery_full", "uma pílula que enche de baixo pra cima", Group::Essentials),
    info("controls", "controles", "tune", "central de controle, wi-fi, bluetooth, som e energia", Group::Essentials),
    info("cpu", "processador", "memory", "o uso da cpu", Group::System),
    info("memory", "memória", "memory_alt", "o uso da ram", Group::System),
    info("temperature", "temperatura", "thermostat", "a temperatura da cpu", Group::System),
    info("brightness", "brilho", "light_mode", "o brilho da tela; role pra mudar", Group::System),
    info("idle", "manter acordado", "coffee", "impede a tela de apagar", Group::System),
    info("media", "música", "play_arrow", "tocar e pausar o que está tocando", Group::Media),
    info("volume", "volume", "volume_up", "o volume, solto; role pra mudar", Group::Media),
    info("notifications", "notificações", "notifications", "abre a central de controle", Group::Shortcuts),
    info("power", "energia", "power_settings_new", "o menu de energia, solto", Group::Shortcuts),
];

pub fn find(id: &str) -> Option<&'static Info> {
    CATALOG.iter().find(|i| i.id == id)
}

/// A choice in the inspector: the value saved and its label.
pub type Choice = (&'static str, &'static str);

pub enum Opt {
    /// A module made in the editor: name, icon, command, click.
    Custom,
    Background,
    /// The shape behind the module; the clock also offers its bun.
    Shape { bun: bool },
    Variant(&'static str, &'static [Choice]),
    Text,
    Action(&'static [Choice]),
    Items(&'static [Choice]),
}

pub const BACKGROUNDS: [Choice; 4] = [("none", "nenhum"), ("tonal", "tonal"), ("primary", "primário"), ("surface", "superfície")];
/// Rounded rectangles; the M3 shapes come from shape::Kind::NAMED.
pub const CORNERS: [Choice; 3] = [("round", "pílula"), ("soft", "arredondado"), ("square", "quadrado")];

/// How the logo's glyph is drawn.
pub enum Glyph {
    /// A Nerd Font character.
    Nerd(&'static str),
    /// A Material Symbols name.
    Symbol(&'static str),
    /// A picture octopus writes in the theme's colors.
    Art(&'static str),
}

pub const GLYPHS: [(&str, &str, Glyph); 20] = [
    ("octopus", "octopus", Glyph::Art("logo.svg")),
    ("octopus-line", "octopus, sem o cookie", Glyph::Art("octopus.svg")),
    ("arch", "arch", Glyph::Nerd("\u{f303}")),
    ("debian", "debian", Glyph::Nerd("\u{f306}")),
    ("fedora", "fedora", Glyph::Nerd("\u{f30a}")),
    ("nixos", "nixos", Glyph::Nerd("\u{f313}")),
    ("ubuntu", "ubuntu", Glyph::Nerd("\u{f31b}")),
    ("gentoo", "gentoo", Glyph::Nerd("\u{f30d}")),
    ("linux", "linux", Glyph::Nerd("\u{f31a}")),
    ("apps", "apps", Glyph::Symbol("apps")),
    ("search", "lupa", Glyph::Symbol("search")),
    ("grid_view", "grade", Glyph::Symbol("grid_view")),
    ("change_history", "triângulo", Glyph::Symbol("change_history")),
    ("blur_on", "pontos", Glyph::Symbol("blur_on")),
    ("auto_awesome", "brilhos", Glyph::Symbol("auto_awesome")),
    ("pets", "patinha", Glyph::Symbol("pets")),
    ("rocket_launch", "foguete", Glyph::Symbol("rocket_launch")),
    ("spa", "flor", Glyph::Symbol("spa")),
    ("hexagon", "hexágono", Glyph::Symbol("hexagon")),
    ("menu", "menu", Glyph::Symbol("menu")),
];

pub fn glyph(variant: &str) -> &'static Glyph {
    GLYPHS.iter().find(|(v, _, _)| *v == variant).map(|(_, _, g)| g).unwrap_or(&GLYPHS[2].2)
}

const ACTIONS: [Choice; 3] = [("launcher", "launcher (rofi)"), ("spotlight", "spotlight"), ("dash", "dash")];
const HOURS: [Choice; 2] = [("24h", "24 horas"), ("12h", "12 horas")];
pub const WORKSPACES: [Choice; 5] = [("dots", "pontos"), ("numbers", "números"), ("pills", "pílulas"), ("squares", "quadrados"), ("bars", "traços")];
const DRAWER: [Choice; 2] = [("drawer", "numa gaveta"), ("open", "sempre à mostra")];
pub const CONTROLS: [Choice; 5] = [
    ("control-center", "central de controle"),
    ("network", "wi-fi"),
    ("bluetooth", "bluetooth"),
    ("volume", "som"),
    ("power", "energia"),
];

/// A whole look for one module, in a tap: (name, background, shape).
pub const LOOKS: [(&str, &str, &str); 8] = [
    ("limpo", "none", "soft"),
    ("tonal", "tonal", "soft"),
    ("pílula", "tonal", "round"),
    ("cookie", "primary", "cookie9"),
    ("flor", "tonal", "flower"),
    ("trevo", "surface", "clover4"),
    ("sol", "primary", "sunny"),
    ("explosão", "tonal", "softburst"),
];

/// What the inspector offers for a module.
pub fn options(id: &str) -> Vec<Opt> {
    if id.starts_with("user-") {
        return vec![Opt::Custom, Opt::Text, Opt::Background, Opt::Shape { bun: false }];
    }
    match id {
        "logo" => vec![Opt::Variant("ícone", &[]), Opt::Action(&ACTIONS), Opt::Background, Opt::Shape { bun: false }],
        "workspaces" => vec![Opt::Variant("estilo", &WORKSPACES)],
        "clock" => vec![Opt::Variant("formato", &HOURS), Opt::Background, Opt::Shape { bun: true }],
        "battery" => vec![Opt::Text],
        "controls" => vec![Opt::Variant("botões", &DRAWER), Opt::Items(&CONTROLS)],
        "tray" => vec![],
        "media" | "idle" | "notifications" | "power" => vec![Opt::Background, Opt::Shape { bun: false }],
        _ => vec![Opt::Text, Opt::Background, Opt::Shape { bun: false }],
    }
}

/// Whether the look presets make sense for a module.
pub fn takes_looks(id: &str) -> bool {
    !matches!(id, "workspaces" | "battery" | "controls" | "tray")
}

/// The arrow tray button: built by install.sh, or shipped by the package.
pub fn arrow() -> Option<String> {
    let built = crate::exec::cache_dir().join("tray-arrow.so");
    let shipped = std::path::PathBuf::from("/usr/lib/octopus/tray-arrow.so");
    [built, shipped].into_iter().find(|p| p.exists()).map(|p| p.to_string_lossy().into_owned())
}

fn icon_module(format: &str, tooltip: &str, click: &str) -> Value {
    json!({ "icon-size": 0, "icon-spacing": 0, "format": format, "tooltip-format": tooltip, "on-click": click })
}

/// The icon in the symbols font, and the value under it when asked.
fn stacked(icon: &str, value: &str, text: bool) -> String {
    let glyph = format!("<span font_family='Material Symbols Rounded' font_size='12pt'>{icon}</span>");
    if text {
        format!("{glyph}\n{value}")
    } else {
        glyph
    }
}

/// Adds a module's Waybar config and gives back its name in the bar.
pub fn waybar(id: &str, m: &Module, bar: &Bar, config: &mut Map<String, Value>) -> Option<String> {
    let name: &str = match id {
        "logo" => {
            let click = match m.action.as_str() {
                "spotlight" => "octopus toggle launcher",
                "dash" => "octopus toggle dash",
                _ => "~/.config/octopus/scripts/rofi.sh",
            };
            let format = match glyph(&m.variant) {
                Glyph::Nerd(c) | Glyph::Symbol(c) => *c,
                // a picture behind a blank: waybar hides a module with no text
                Glyph::Art(_) => " ",
            };
            let mut launcher = icon_module(format, "", click);
            launcher["tooltip"] = json!(false);
            config.insert("custom/launcher".into(), launcher);
            "custom/launcher"
        }
        "workspaces" => {
            let numbers = m.variant == "numbers";
            config.insert(
                "niri/workspaces".into(),
                if numbers {
                    json!({ "format": "{index}" })
                } else {
                    json!({ "format": "{icon}", "format-icons": { "default": "", "active": "" } })
                },
            );
            "niri/workspaces"
        }
        "clock" => {
            let english = crate::i18n::english();
            config.insert(
                "clock".into(),
                json!({
                    "format": if m.variant == "12h" { "{:%I\n%M}" } else { "{:%H\n%M}" },
                    "justify": "center",
                    "locale": if english { "en_US.UTF-8" } else { "pt_BR.UTF-8" },
                    "tooltip-format": if english { "{:%A, %B %d}" } else { "{:%A, %d de %B}" },
                    "on-click": "octopus toggle dash"
                }),
            );
            "clock"
        }
        "tray" => {
            config.insert(
                "custom/tray".into(),
                json!({
                    "icon-size": 0, "icon-spacing": 0,
                    "exec": "octopus tray-status", "return-type": "json",
                    "signal": 8, "interval": "once", "format": "{}",
                    "on-click": "octopus toggle tray"
                }),
            );
            match arrow() {
                Some(path) => {
                    config.insert("cffi/tray".into(), json!({ "module_path": path, "position": bar.position }));
                    "cffi/tray"
                }
                None => "custom/tray",
            }
        }
        "battery" => {
            let modules: Vec<&str> = if m.text { vec!["custom/battery", "battery"] } else { vec!["custom/battery"] };
            config.insert("group/charge".into(), json!({ "orientation": "inherit", "modules": modules }));
            config.insert("battery".into(), json!({ "format": "{capacity}", "interval": 60, "tooltip": false }));
            config.insert(
                "custom/battery".into(),
                json!({
                    "icon-size": 0, "icon-spacing": 0,
                    "exec": "~/.config/octopus/scripts/battery.sh", "return-type": "json",
                    "interval": 20, "signal": 9, "format": "{}"
                }),
            );
            "group/charge"
        }
        "controls" => {
            let names: Vec<&str> = m
                .items
                .iter()
                .filter_map(|item| {
                    Some(match item.as_str() {
                        "control-center" => "custom/control-center",
                        "network" => "network",
                        "bluetooth" => "bluetooth",
                        "volume" => "wireplumber",
                        "power" => "custom/power",
                        _ => return None,
                    })
                })
                .collect();
            let mut group = json!({ "orientation": "inherit", "modules": names });
            if m.variant != "open" {
                group["drawer"] = json!({
                    "transition-duration": 320,
                    "children-class": "control-child",
                    "transition-left-to-right": false
                });
            }
            config.insert("group/controls".into(), group);
            config.insert("custom/control-center".into(), icon_module("tune", t("central de controle"), "octopus toggle control-center"));
            config.insert(
                "network".into(),
                json!({
                    "format-wifi": "{icon}", "format-ethernet": "lan",
                    "format-disconnected": "wifi_off", "format-disabled": "wifi_off",
                    "format-icons": ["network_wifi_1_bar", "network_wifi_2_bar", "network_wifi_3_bar", "signal_wifi_4_bar"],
                    "tooltip-format-wifi": "{essid} · {signalStrength}%",
                    "tooltip-format-disconnected": t("desconectado"),
                    "on-click": "octopus toggle wifi"
                }),
            );
            config.insert(
                "bluetooth".into(),
                json!({
                    "format": "bluetooth", "format-disabled": "bluetooth_disabled",
                    "format-off": "bluetooth_disabled", "format-connected": "bluetooth_connected",
                    "tooltip-format-connected": "{device_alias}",
                    "on-click": "octopus toggle bluetooth"
                }),
            );
            config.insert(
                "wireplumber".into(),
                json!({
                    "format": "{icon}", "format-muted": "volume_off",
                    "format-icons": ["volume_mute", "volume_down", "volume_up"],
                    "tooltip-format": "{volume}% · {node_name}",
                    "on-click": "octopus toggle audio", "scroll-step": 5
                }),
            );
            config.insert("custom/power".into(), icon_module("power_settings_new", t("energia"), "octopus toggle power"));
            "group/controls"
        }
        "cpu" => {
            config.insert("cpu".into(), json!({ "format": stacked("memory", "{usage}", m.text), "justify": "center", "interval": 5, "tooltip-format": "cpu {usage}%" }));
            "cpu"
        }
        "memory" => {
            config.insert(
                "memory".into(),
                json!({ "format": stacked("memory_alt", "{percentage}", m.text), "justify": "center", "interval": 10, "tooltip-format": "{used:0.1f} / {total:0.1f} GiB" }),
            );
            "memory"
        }
        "temperature" => {
            config.insert(
                "temperature".into(),
                json!({ "format": stacked("thermostat", "{temperatureC}", m.text), "justify": "center", "critical-threshold": 85, "interval": 10, "tooltip-format": "{temperatureC}°C" }),
            );
            "temperature"
        }
        "brightness" => {
            config.insert(
                "backlight".into(),
                json!({
                    "format": stacked("{icon}", "{percent}", m.text), "justify": "center",
                    "format-icons": ["brightness_low", "brightness_medium", "brightness_high"],
                    "tooltip-format": "{percent}%"
                }),
            );
            "backlight"
        }
        "idle" => {
            config.insert(
                "idle_inhibitor".into(),
                json!({
                    "format": "{icon}",
                    "format-icons": { "activated": "coffee", "deactivated": "bedtime" },
                    "tooltip-format-activated": t("a tela não vai apagar"),
                    "tooltip-format-deactivated": t("a tela apaga quando ocioso")
                }),
            );
            "idle_inhibitor"
        }
        "media" => {
            config.insert(
                "mpris".into(),
                json!({
                    "format": "{status_icon}",
                    "status-icons": { "playing": "pause", "paused": "play_arrow", "stopped": "music_note" },
                    "tooltip-format": "{title} · {artist}"
                }),
            );
            "mpris"
        }
        "volume" => {
            config.insert(
                "wireplumber#bar".into(),
                json!({
                    "format": stacked("{icon}", "{volume}", m.text), "justify": "center",
                    "format-muted": stacked("volume_off", "{volume}", m.text),
                    "format-icons": ["volume_mute", "volume_down", "volume_up"],
                    "tooltip-format": "{volume}% · {node_name}",
                    "on-click": "octopus toggle audio", "scroll-step": 5
                }),
            );
            "wireplumber#bar"
        }
        "notifications" => {
            config.insert("custom/notifications".into(), icon_module("notifications", t("notificações"), "octopus toggle control-center"));
            "custom/notifications"
        }
        "power" => {
            config.insert("custom/power#bar".into(), icon_module("power_settings_new", t("energia"), "octopus toggle power"));
            "custom/power#bar"
        }
        _ if id.starts_with("user-") => {
            let custom = bar.custom(id)?;
            let shows_value = m.text && !custom.exec.trim().is_empty();
            let mut module = json!({
                "format": stacked(&custom.icon, "{}", shows_value),
                "justify": "center",
                "tooltip-format": custom.name,
            });
            if !custom.exec.trim().is_empty() {
                module["exec"] = json!(custom.exec);
                module["interval"] = json!(custom.interval.max(1));
            }
            if !custom.click.trim().is_empty() {
                module["on-click"] = json!(custom.click);
            }
            config.insert(format!("custom/{id}"), module);
            return Some(format!("custom/{id}"));
        }
        _ => return None,
    };
    Some(name.to_owned())
}

/// The CSS selector a module's look applies to.
pub fn selector(id: &str) -> Option<String> {
    if id.starts_with("user-") {
        return Some(format!("#custom-{id}"));
    }
    Some(match id {
        "logo" => "#custom-launcher",
        "clock" => "#clock",
        "cpu" => "#cpu",
        "memory" => "#memory",
        "temperature" => "#temperature",
        "brightness" => "#backlight",
        "idle" => "#idle_inhibitor",
        "media" => "#mpris",
        "volume" => "#wireplumber.bar",
        "notifications" => "#custom-notifications",
        "power" => "#custom-power.bar",
        _ => return None,
    }
    .to_owned())
}

fn colors(background: &str) -> Option<(&'static str, &'static str)> {
    Some(match background {
        "tonal" => ("@secondary_container", "@on_secondary_container"),
        "primary" => ("@primary_container", "@on_primary_container"),
        "surface" => ("@surface_container_highest", "@on_surface"),
        _ => return None,
    })
}

fn radius(shape: &str) -> &'static str {
    match shape {
        "round" => "999px",
        "square" => "4px",
        _ => "12px",
    }
}

fn workspaces_css(style: &str, width: u32) -> String {
    let side = |inner: u32| width.saturating_sub(inner) / 2;
    let (w, h, active_h, radius) = match style {
        "numbers" => {
            return format!(
                "#workspaces button {{ min-height: 20px; margin: 2px {m}px; border-radius: 8px; background: transparent; }}\n\
#workspaces button label {{ font-size: 11px; font-weight: 600; color: @on_surface_variant; }}\n\
#workspaces button.active {{ min-height: 26px; background: @primary_container; }}\n\
#workspaces button.active label {{ color: @on_primary_container; }}\n",
                m = side(26)
            )
        }
        "pills" => (18, 10, 24, "999px"),
        "squares" => (12, 12, 12, "4px"),
        "bars" => (4, 14, 30, "999px"),
        _ => return String::new(),
    };
    let mut css = format!(
        "#workspaces button {{ min-width: {w}px; min-height: {h}px; margin: 3px {m}px; border-radius: {radius}; }}\n",
        m = side(w)
    );
    if style == "squares" {
        css.push_str(&format!(
            "#workspaces button.active {{ min-width: 16px; min-height: 16px; margin: 3px {m}px; }}\n",
            m = side(16)
        ));
    } else {
        css.push_str(&format!("#workspaces button.active {{ min-height: {active_h}px; }}\n"));
    }
    css
}

/// The module's own look, on top of style.css.
pub fn css(id: &str, m: &Module, bar: &Bar) -> String {
    let mut out = String::new();
    if id == "logo" {
        match glyph(&m.variant) {
            Glyph::Symbol(_) => out.push_str("#custom-launcher { font-family: \"Material Symbols Rounded\"; font-size: 24px; padding: 6px 0; }\n"),
            Glyph::Art(file) => out.push_str(&format!(
                "#custom-launcher, #custom-launcher:hover {{ background-image: url(\"art/{file}\"); background-size: contain; \
background-position: center; background-repeat: no-repeat; min-height: 34px; font-size: 1px; color: transparent; padding: 0; margin-top: 2px; margin-bottom: 4px; }}\n"
            )),
            Glyph::Nerd(_) => {}
        }
    }
    if id == "workspaces" {
        out.push_str(&workspaces_css(&m.variant, bar.width));
    }
    let Some(selector) = selector(id) else { return out };
    if id.starts_with("user-") {
        out.push_str(&format!(
            "{selector} {{ font-family: \"Rubik\", sans-serif; font-size: 10px; font-weight: 600; color: @on_surface; padding: 6px 0; min-width: {}px; margin-left: 8px; margin-right: 8px; }}\n\
{selector}:hover {{ background-color: @surface_container_highest; }}\n",
            bar.width.saturating_sub(16)
        ));
    }
    let shaped = crate::widgets::shape::Kind::from_name(&m.shape).is_some();
    if id == "clock" && m.shape == "bun" {
        return out;
    }
    if shaped {
        // an M3 shape, drawn by bar::art in the theme's colors
        let (_, fg) = colors(if m.background == "none" { "tonal" } else { &m.background }).unwrap_or(("", "@on_surface"));
        let hover = if id == "clock" { ", #clock:hover".to_owned() } else { format!(", {selector}:hover") };
        out.push_str(&format!(
            "{selector}{hover} {{ background-color: transparent; background-image: url(\"art/{id}.svg\"); background-size: contain; \
background-position: center; background-repeat: no-repeat; border-radius: 0; color: {fg}; min-height: 34px; }}\n"
        ));
        if id == "clock" {
            // the whole column's width, so the shape is as big as it can be
            out.push_str("#clock { margin-left: 0; margin-right: 0; min-height: 48px; padding: 0; }\n");
        }
        return out;
    }
    if id == "clock" {
        let (bg, fg) = colors(&m.background).unwrap_or(("transparent", "@on_surface"));
        out.push_str(&format!(
            "#clock, #clock:hover {{ background-image: none; background-color: {bg}; color: {fg}; border-radius: {}; }}\n",
            radius(&m.shape)
        ));
        return out;
    }
    match colors(&m.background) {
        Some((bg, fg)) => out.push_str(&format!(
            "{selector} {{ background: {bg}; color: {fg}; border-radius: {}; }}\n",
            radius(&m.shape)
        )),
        None => out.push_str(&format!("{selector} {{ border-radius: {}; }}\n", radius(&m.shape))),
    }
    out
}
