//! Where each desktop widget sits and how big it is, kept across restarts.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

fn path() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| gtk::glib::home_dir().join(".local/state"))
        .join("octopus")
        .join("desktop.json")
}

pub fn load() -> HashMap<String, Rect> {
    std::fs::read_to_string(path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

pub fn save(layout: &HashMap<String, Rect>) {
    let path = path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(text) = serde_json::to_string_pretty(layout) {
        if let Err(err) = std::fs::write(&path, text) {
            eprintln!("octopus: cannot save {}: {err}", path.display());
        }
    }
}
