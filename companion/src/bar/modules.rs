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

pub const GROUPS: [(Group, &str); 4] = [
    (Group::Essentials, "essenciais"),
    (Group::System, "sistema"),
    (Group::Media, "áudio e mídia"),
    (Group::Shortcuts, "atalhos"),
];

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
    Background,
    Shape(&'static [Choice]),
    Variant(&'static str, &'static [Choice]),
    Text,
    Action(&'static [Choice]),
    Items(&'static [Choice]),
}

pub const BACKGROUNDS: [Choice; 4] = [("none", "nenhum"), ("tonal", "tonal"), ("primary", "primário"), ("surface", "superfície")];
const SHAPES: [Choice; 3] = [("round", "pílula"), ("soft", "arredondado"), ("square", "quadrado")];
const CLOCK_SHAPES: [Choice; 4] = [("bun", "bun"), ("round", "pílula"), ("soft", "arredondado"), ("square", "quadrado")];
pub const GLYPHS: [Choice; 6] = [
    ("arch", "arch"),
    ("apps", "apps"),
    ("search", "lupa"),
    ("grid_view", "grade"),
    ("change_history", "triângulo"),
    ("blur_on", "pontos"),
];
const ACTIONS: [Choice; 3] = [("launcher", "launcher (rofi)"), ("spotlight", "spotlight"), ("dash", "dash")];
const HOURS: [Choice; 2] = [("24h", "24 horas"), ("12h", "12 horas")];
const WORKSPACES: [Choice; 2] = [("dots", "pontos"), ("numbers", "números")];
const DRAWER: [Choice; 2] = [("drawer", "numa gaveta"), ("open", "sempre à mostra")];
pub const CONTROLS: [Choice; 5] = [
    ("control-center", "central de controle"),
    ("network", "wi-fi"),
    ("bluetooth", "bluetooth"),
    ("volume", "som"),
    ("power", "energia"),
];

/// What the inspector offers for a module.
pub fn options(id: &str) -> Vec<Opt> {
    match id {
        "logo" => vec![Opt::Variant("ícone", &GLYPHS), Opt::Action(&ACTIONS), Opt::Background, Opt::Shape(&SHAPES)],
        "workspaces" => vec![Opt::Variant("estilo", &WORKSPACES)],
        "clock" => vec![Opt::Variant("formato", &HOURS), Opt::Shape(&CLOCK_SHAPES), Opt::Background],
        "battery" => vec![Opt::Text],
        "controls" => vec![Opt::Variant("botões", &DRAWER), Opt::Items(&CONTROLS)],
        "tray" => vec![],
        "media" | "idle" | "notifications" | "power" => vec![Opt::Background, Opt::Shape(&SHAPES)],
        _ => vec![Opt::Text, Opt::Background, Opt::Shape(&SHAPES)],
    }
}

/// The arrow tray button, when install.sh could build it.
pub fn arrow() -> Option<String> {
    let path = crate::exec::cache_dir().join("tray-arrow.so");
    path.exists().then(|| path.to_string_lossy().into_owned())
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
pub fn waybar(id: &str, m: &Module, bar: &Bar, config: &mut Map<String, Value>) -> Option<&'static str> {
    Some(match id {
        "logo" => {
            let click = match m.action.as_str() {
                "spotlight" => "octopus toggle launcher",
                "dash" => "octopus toggle dash",
                _ => "~/.config/octopus/scripts/rofi.sh",
            };
            let glyph = if m.variant == "arch" || m.variant.is_empty() { "\u{f303}" } else { m.variant.as_str() };
            let mut launcher = icon_module(glyph, "", click);
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
        _ => return None,
    })
}

/// The CSS selector a module's look applies to.
pub fn selector(id: &str) -> Option<&'static str> {
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
    })
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

/// The module's own look, on top of style.css.
pub fn css(id: &str, m: &Module) -> String {
    let mut out = String::new();
    if id == "logo" && m.variant != "arch" && !m.variant.is_empty() {
        out.push_str("#custom-launcher { font-family: \"Material Symbols Rounded\"; font-size: 24px; padding: 6px 0; }\n");
    }
    if id == "workspaces" && m.variant == "numbers" {
        out.push_str(
            "#workspaces button { min-height: 20px; border-radius: 8px; background: transparent; }\n\
#workspaces button label { font-size: 11px; font-weight: 600; color: @on_surface_variant; }\n\
#workspaces button.active { min-height: 26px; background: @primary_container; }\n\
#workspaces button.active label { color: @on_primary_container; }\n",
        );
    }
    let Some(selector) = selector(id) else { return out };
    if id == "clock" {
        if m.shape == "bun" {
            return out;
        }
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
