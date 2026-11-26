//! Pictures for Waybar in the theme's colors: the octopus logo, and the M3
//! shape behind each module that asked for one. GTK 3 can't tint an SVG,
//! so they are written again whenever the colors change.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::Bar;
use crate::widgets::shape::{self, Kind};

/// The logo as drawn: a cookie, and the octopus on it.
const LOGO_CONTAINER: &str = "#e9e78f";
const LOGO_INK: &str = "#1d1d00";

fn dir() -> PathBuf {
    gtk::glib::user_config_dir().join("waybar").join("art")
}

/// `@define-color name #hex;` lines from Waybar's colors.css.
pub fn colors() -> HashMap<String, String> {
    let path = gtk::glib::user_config_dir().join("waybar/colors.css");
    std::fs::read_to_string(path)
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("@define-color ")?;
            let (name, value) = rest.split_once(' ')?;
            Some((name.to_owned(), value.trim().trim_end_matches(';').to_owned()))
        })
        .collect()
}

fn write(name: &str, text: &str) -> bool {
    let dir = dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(name);
    if std::fs::read_to_string(&path).ok().as_deref() == Some(text) {
        return false;
    }
    std::fs::write(path, text).is_ok()
}

/// logo.svg (the whole logo) and octopus.svg (only the octopus, in the
/// primary color), from the repo's logo.
pub fn logos() -> bool {
    let source = crate::exec::config_dir().join("assets/logo.svg");
    let Ok(svg) = std::fs::read_to_string(source) else { return false };
    let c = colors();
    let get = |name: &str, fallback: &str| c.get(name).cloned().unwrap_or_else(|| fallback.to_owned());
    let whole = svg
        .replace(LOGO_CONTAINER, &get("primary_container", LOGO_CONTAINER))
        .replace(LOGO_INK, &get("on_primary_container", LOGO_INK));
    // the octopus alone: the cookie goes, the eyes take the bar's color
    let alone: String = svg
        .lines()
        .filter(|l| !(l.contains("<path fill=\"") && l.contains(LOGO_CONTAINER)))
        .collect::<Vec<_>>()
        .join("\n")
        .replace(LOGO_INK, &get("primary", LOGO_INK))
        .replace(LOGO_CONTAINER, &get("surface_container", LOGO_CONTAINER));
    let a = write("logo.svg", &whole);
    let b = write("octopus.svg", &alone);
    a || b
}

pub fn logo_path() -> PathBuf {
    dir().join("logo.svg")
}

/// The background color of a module's look.
pub fn fill_of(background: &str) -> &'static str {
    match background {
        "primary" => "primary_container",
        "surface" => "surface_container_highest",
        _ => "secondary_container",
    }
}

/// One shape for each module whose look is an M3 shape.
pub fn shapes(bar: &Bar) -> bool {
    let c = colors();
    let mut changed = false;
    for id in bar.placed() {
        let m = bar.module(id);
        let Some(kind) = Kind::from_name(&m.shape) else { continue };
        let fill = c.get(fill_of(&m.background)).cloned().unwrap_or_else(|| "#888888".into());
        let svg = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 100 100\"><path fill=\"{fill}\" d=\"{}\"/></svg>\n",
            shape::svg_path(kind, 100.0)
        );
        changed |= write(&format!("{id}.svg"), &svg);
    }
    changed
}

/// Everything again, for new colors.
pub fn refresh(bar: &Bar) {
    let a = logos();
    let b = shapes(bar);
    if a || b {
        crate::exec::spawn("pkill", &["-SIGUSR2", "-x", "waybar"]);
    }
}
