//! Finder's colored tags. Each file keeps its own in the `user.xdg.tags`
//! extended attribute (what other freedesktop tools read too); a small index
//! lets the sidebar list every file with a tag.

use std::collections::BTreeMap;

use gtk::gio;
use gtk::prelude::*;

/// (id, name, color)
pub const COLORS: [(&str, &str, &str); 7] = [
    ("red", "vermelho", "#e5534b"),
    ("orange", "laranja", "#f0883e"),
    ("yellow", "amarelo", "#e3b341"),
    ("green", "verde", "#57ab5a"),
    ("blue", "azul", "#539bf5"),
    ("purple", "roxo", "#986ee2"),
    ("gray", "cinza", "#8b949e"),
];

pub fn color(id: &str) -> &'static str {
    COLORS.iter().find(|c| c.0 == id).map(|c| c.2).unwrap_or("#8b949e")
}

pub fn of(info: &gio::FileInfo) -> Vec<String> {
    info.attribute_string("xattr::xdg.tags")
        .map(|s| s.split(',').map(|t| t.trim().to_owned()).filter(|t| !t.is_empty()).collect())
        .unwrap_or_default()
}

fn index_path() -> std::path::PathBuf {
    super::state_dir().join("octopus/tags.json")
}

fn index() -> BTreeMap<String, Vec<String>> {
    std::fs::read_to_string(index_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// Turns a tag on or off on a file.
pub fn toggle(file: &gio::File, tag: &str) {
    let current: Vec<String> = file
        .query_info("xattr::xdg.tags", gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE)
        .map(|i| of(&i))
        .unwrap_or_default();
    let mut next = current.clone();
    if next.iter().any(|t| t == tag) {
        next.retain(|t| t != tag);
    } else {
        next.push(tag.to_owned());
    }
    let value = next.join(",");
    let _ = file.set_attribute_string("xattr::xdg.tags", &value, gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE);
    let mut idx = index();
    let uri = file.uri().to_string();
    if next.is_empty() {
        idx.remove(&uri);
    } else {
        idx.insert(uri, next);
    }
    if let Ok(json) = serde_json::to_string(&idx) {
        let _ = std::fs::create_dir_all(index_path().parent().unwrap_or(std::path::Path::new("/tmp")));
        let _ = std::fs::write(index_path(), json);
    }
}

/// Every file known to have a tag, that still exists.
pub fn tagged(tag: &str) -> Vec<gio::File> {
    index()
        .into_iter()
        .filter(|(_, tags)| tags.iter().any(|t| t == tag))
        .map(|(uri, _)| gio::File::for_uri(&uri))
        .filter(|f| f.query_exists(gio::Cancellable::NONE))
        .collect()
}
