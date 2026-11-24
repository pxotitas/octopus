use std::collections::HashMap;

use gio_unix::prelude::*;
use gio_unix::DesktopAppInfo;
use gtk::gio;
use gtk::prelude::*;

/// Desktop entries by id, and which entry each window class belongs to.
///
/// `StartupWMClass` is the intended field but many entries lack it, so the
/// file id, its last dotted segment, the `Exec` binary and the name are also
/// claimed; the highest claim on a class wins (after Velatura's dock).
pub struct Apps {
    entries: HashMap<String, DesktopAppInfo>,
    classes: HashMap<String, (u8, String)>,
}

impl Apps {
    pub fn load() -> Self {
        let mut entries = HashMap::new();
        let mut classes: HashMap<String, (u8, String)> = HashMap::new();
        let mut claim = |key: &str, id: &str, score: u8| {
            let key = key.trim().to_lowercase();
            if key.is_empty() {
                return;
            }
            let held = classes.get(&key).map(|(s, _)| *s).unwrap_or(0);
            if score > held {
                classes.insert(key, (score, id.to_owned()));
            }
        };
        for info in gio::AppInfo::all() {
            let Ok(app) = info.downcast::<DesktopAppInfo>() else {
                continue;
            };
            let Some(id) = app.id().map(|i| i.to_string()) else {
                continue;
            };
            let base = id.trim_end_matches(".desktop").to_owned();
            claim(&app.name(), &id, 1);
            if let Some(binary) = app
                .executable()
                .and_then(|e| e.file_name().map(|f| f.to_string_lossy().into_owned()))
            {
                claim(&binary, &id, 2);
            }
            claim(base.rsplit('.').next().unwrap_or(&base), &id, 3);
            claim(&base, &id, 4);
            if let Some(class) = app.startup_wm_class() {
                let bonus = if class.eq_ignore_ascii_case(&base) {
                    6
                } else {
                    5
                };
                claim(&class, &id, bonus);
            }
            entries.insert(id, app);
        }
        Self { entries, classes }
    }

    pub fn id_for(&self, class: &str) -> Option<&str> {
        self.classes
            .get(&class.trim().to_lowercase())
            .map(|(_, id)| id.as_str())
    }

    pub fn get(&self, id: &str) -> Option<&DesktopAppInfo> {
        self.entries.get(id)
    }

    pub fn launch(&self, id: &str) {
        let Some(app) = self.entries.get(id) else {
            return;
        };
        let context = gtk::gdk::Display::default().map(|d| d.app_launch_context());
        if let Err(err) = app.launch(&[], context.as_ref()) {
            eprintln!("octopus: cannot launch {id}: {err}");
        }
    }
}
