use serde_json::{json, Map, Value};

use crate::config::Bar;
use crate::exec;
use crate::i18n::t;

/// The bar's modules by the names the settings use.
pub const MODULES: [(&str, &str); 6] = [
    ("logo", "logo do arch"),
    ("workspaces", "workspaces"),
    ("clock", "relógio"),
    ("tray", "bandeja"),
    ("battery", "bateria"),
    ("controls", "controles"),
];

fn waybar_name(module: &str) -> Option<&'static str> {
    Some(match module {
        "logo" => "custom/launcher",
        "workspaces" => "niri/workspaces",
        "clock" => "clock",
        "tray" if arrow().is_some() => "cffi/tray",
        "tray" => "custom/tray",
        "battery" => "group/charge",
        "controls" => "group/controls",
        _ => return None,
    })
}

/// Radius of the flush bar's inward corners.
const CONCAVE: u32 = 14;

/// The flush bar: square against the screen's edge, with two inward corners
/// where it meets the top and the bottom, so the rest of the screen reads as
/// a rounded card. Drawn with gradients, since GTK 3 has no masks.
fn flush_css(bar: &Bar) -> String {
    let (w, r) = (bar.width, CONCAVE);
    let fill = "@surface_container";
    let (body, top, bottom, pad, top_at, bottom_at) = if bar.right() {
        (
            "right top",
            "0% 100%",
            "0% 0%",
            "padding-left",
            "0 0".to_owned(),
            "0 100%".to_owned(),
        )
    } else {
        (
            "left top",
            "100% 100%",
            "100% 0%",
            "padding-right",
            format!("{w}px 0"),
            format!("{w}px 100%"),
        )
    };
    format!(
        "window#waybar > box {{ border-radius: 0; {pad}: {r}px; background-color: transparent; \
background-image: linear-gradient({fill}, {fill}), \
radial-gradient(circle at {top}, transparent {inner}px, {fill} {r}px), \
radial-gradient(circle at {bottom}, transparent {inner}px, {fill} {r}px); \
background-size: {w}px 100%, {r}px {r}px, {r}px {r}px; \
background-position: {body}, {top_at}, {bottom_at}; background-repeat: no-repeat; }}\n",
        inner = r as f64 - 0.5,
    )
}

/// The animated tray button, when install.sh could build it.
fn arrow() -> Option<String> {
    let path = crate::exec::cache_dir().join("tray-arrow.so");
    path.exists().then(|| path.to_string_lossy().into_owned())
}

fn icon_module(format: &str, tooltip: &str, click: &str) -> Value {
    json!({ "icon-size": 0, "icon-spacing": 0, "format": format, "tooltip-format": tooltip, "on-click": click })
}

/// Waybar's config, generated from octopus.toml so the settings window can
/// move things around.
pub fn waybar_config(bar: &Bar) -> String {
    let names = |list: &[String]| -> Vec<Value> {
        list.iter()
            .filter_map(|m| waybar_name(m))
            .map(Value::from)
            .collect()
    };
    let side_margin = if bar.position == "right" {
        "margin-right"
    } else {
        "margin-left"
    };
    let mut config = Map::new();
    config.insert("layer".into(), json!("top"));
    config.insert("position".into(), json!(bar.position));
    let flush = bar.style == "flush";
    let (width, margin) = if flush {
        (bar.width + CONCAVE, 0)
    } else {
        (bar.width, 8)
    };
    config.insert("width".into(), json!(width));
    config.insert("margin-top".into(), json!(margin));
    config.insert("margin-bottom".into(), json!(margin));
    config.insert(side_margin.into(), json!(margin));
    config.insert("spacing".into(), json!(4));
    config.insert("reload_style_on_change".into(), json!(true));
    config.insert("modules-left".into(), Value::Array(names(&bar.top)));
    config.insert("modules-center".into(), Value::Array(names(&bar.center)));
    config.insert("modules-right".into(), Value::Array(names(&bar.bottom)));

    let mut launcher = icon_module("\u{f303}", "", "~/.config/octopus/scripts/rofi.sh");
    launcher["tooltip"] = json!(false);
    config.insert("custom/launcher".into(), launcher);
    config.insert(
        "niri/workspaces".into(),
        json!({ "format": "{icon}", "format-icons": { "default": "", "active": "" } }),
    );
    config.insert(
        "clock".into(),
        json!({
            "format": "{:%H\n%M}",
            "justify": "center",
            "locale": if crate::i18n::english() { "en_US.UTF-8" } else { "pt_BR.UTF-8" },
            "tooltip-format": if crate::i18n::english() { "{:%A, %B %d}" } else { "{:%A, %d de %B}" },
            "on-click": "octopus toggle dash"
        }),
    );
    if let Some(path) = arrow() {
        config.insert(
            "cffi/tray".into(),
            json!({ "module_path": path, "position": bar.position }),
        );
    }
    config.insert(
        "custom/tray".into(),
        json!({
            "icon-size": 0,
            "icon-spacing": 0,
            "exec": "octopus tray-status",
            "return-type": "json",
            "signal": 8,
            "interval": "once",
            "format": "{}",
            "on-click": "octopus toggle tray"
        }),
    );
    config.insert(
        "group/charge".into(),
        json!({ "orientation": "inherit", "modules": ["custom/battery", "battery"] }),
    );
    config.insert(
        "battery".into(),
        json!({ "format": "{capacity}", "interval": 60, "tooltip": false }),
    );
    config.insert(
        "custom/battery".into(),
        json!({
            "icon-size": 0,
            "icon-spacing": 0,
            "exec": "~/.config/octopus/scripts/battery.sh",
            "return-type": "json",
            "interval": 20,
            "signal": 9,
            "format": "{}"
        }),
    );
    config.insert(
        "group/controls".into(),
        json!({
            "orientation": "inherit",
            "drawer": {
                "transition-duration": 320,
                "children-class": "control-child",
                "transition-left-to-right": false
            },
            "modules": ["custom/control-center", "network", "bluetooth", "wireplumber", "custom/power"]
        }),
    );
    config.insert(
        "custom/control-center".into(),
        icon_module(
            "tune",
            "central de controle",
            "octopus toggle control-center",
        ),
    );
    config.insert(
        "network".into(),
        json!({
            "format-wifi": "{icon}",
            "format-ethernet": "lan",
            "format-disconnected": "wifi_off",
            "format-disabled": "wifi_off",
            "format-icons": ["network_wifi_1_bar", "network_wifi_2_bar", "network_wifi_3_bar", "signal_wifi_4_bar"],
            "tooltip-format-wifi": "{essid} · {signalStrength}%",
            "tooltip-format-disconnected": t("desconectado"),
            "on-click": "octopus toggle wifi"
        }),
    );
    config.insert(
        "bluetooth".into(),
        json!({
            "format": "bluetooth",
            "format-disabled": "bluetooth_disabled",
            "format-off": "bluetooth_disabled",
            "format-connected": "bluetooth_connected",
            "tooltip-format-connected": "{device_alias}",
            "on-click": "octopus toggle bluetooth"
        }),
    );
    config.insert(
        "wireplumber".into(),
        json!({
            "format": "{icon}",
            "format-muted": "volume_off",
            "format-icons": ["volume_mute", "volume_down", "volume_up"],
            "tooltip-format": "{volume}% · {node_name}",
            "on-click": "octopus toggle audio",
            "scroll-step": 5
        }),
    );
    config.insert(
        "custom/power".into(),
        icon_module("power_settings_new", t("energia"), "octopus toggle power"),
    );

    let mut text = String::from(
        "// Generated by octopus from octopus.toml [bar]; change it in `octopus settings`.\n",
    );
    text.push_str(&serde_json::to_string_pretty(&Value::Object(config)).unwrap_or_default());
    text.push('\n');
    text
}

/// Everything in the bar that depends on its width: each module fills the
/// column (margins + min-width = width) so its glyph sits centered.
pub fn layout_css(bar: &Bar) -> String {
    let w = bar.width as i32;
    let side = |inner: i32| ((w - inner) / 2).max(0);
    format!(
        "/* generated by octopus from octopus.toml [bar] */\n\
#custom-launcher, #custom-control-center, #custom-tray, #tray-arrow, #network, #bluetooth, #wireplumber, #custom-power {{ min-width: {icon}px; margin-left: 8px; margin-right: 8px; }}\n\
#workspaces button {{ margin: 3px {ws}px; }}\n\
#clock {{ margin-left: {clock}px; margin-right: {clock}px; }}\n\
#custom-battery {{ margin: 4px {bat}px 0; }}\n\
#battery {{ margin-left: {pct}px; margin-right: {pct}px; }}\n{flush}",
        flush = if bar.style == "flush" { flush_css(bar) } else { String::new() },
        icon = (w - 16).max(0),
        ws = side(10),
        clock = side(32),
        bat = side(14),
        pct = side(24),
    )
}

/// Writes Waybar's generated files and asks it to reload when they changed.
pub fn apply(bar: &Bar) {
    let dir = gtk::glib::user_config_dir().join("waybar");
    let _ = std::fs::create_dir_all(&dir);
    let mut changed = false;
    for (name, text) in [
        ("config.jsonc", waybar_config(bar)),
        ("layout.css", layout_css(bar)),
    ] {
        let path = dir.join(name);
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            changed |= std::fs::write(path, text).is_ok();
        }
    }
    if changed {
        exec::spawn("pkill", &["-SIGUSR2", "-x", "waybar"]);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn flush_css_mentions_both_corners() {
        let mut bar = crate::config::Bar::default();
        bar.style = "flush".into();
        let css = super::layout_css(&bar);
        println!("{css}");
        assert!(css.contains("radial-gradient(circle at 100% 100%"));
    }

    #[test]
    fn default_config_is_valid_json() {
        let bar = crate::config::Bar::default();
        let text = super::waybar_config(&bar);
        let body: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        let parsed: serde_json::Value = serde_json::from_str(&body).expect("valid json");
        assert_eq!(parsed["width"], 52);
        std::fs::write("/tmp/waybar-default.jsonc", &text).unwrap();
        std::fs::write("/tmp/waybar-layout.css", super::layout_css(&bar)).unwrap();
    }
}
