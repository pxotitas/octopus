//! What the file manager remembers between windows: how folders are shown.

use std::cell::RefCell;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    /// "icons", "list" or "columns".
    pub view: String,
    /// "name", "size", "modified" or "kind".
    pub sort: String,
    pub descending: bool,
    pub hidden: bool,
    /// The icons' size in the icon view.
    pub zoom: i32,
    pub sidebar: i32,
    pub width: i32,
    pub height: i32,
}

impl Default for State {
    fn default() -> Self {
        Self { view: "icons".into(), sort: "name".into(), descending: false, hidden: false, zoom: 64, sidebar: 220, width: 1040, height: 680 }
    }
}

fn path() -> std::path::PathBuf {
    super::state_dir().join("octopus/files.json")
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(
        std::fs::read_to_string(path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    );
}

pub fn get() -> State {
    STATE.with(|s| s.borrow().clone())
}

pub fn edit(change: impl FnOnce(&mut State)) {
    STATE.with(|s| {
        change(&mut s.borrow_mut());
        let path = path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(json) = serde_json::to_string_pretty(&*s.borrow()) {
            let _ = std::fs::write(path, json);
        }
    });
}
