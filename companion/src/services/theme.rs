use std::rc::Rc;

use gtk::glib;

use crate::exec;
use crate::store::Store;

/// Light or dark palette. The choice lives in a state file that the theme
/// script reads, so Matuwall's hook and this toggle agree.
pub struct Theme {
    pub dark: Rc<Store<bool>>,
}

fn state_file() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| glib::home_dir().join(".local/state"))
        .join("octopus")
        .join("mode")
}

impl Theme {
    pub fn new() -> Rc<Self> {
        let dark = std::fs::read_to_string(state_file())
            .map(|m| m.trim() != "light")
            .unwrap_or(true);
        Rc::new(Self {
            dark: Store::new(dark),
        })
    }

    pub fn set_dark(&self, dark: bool) {
        self.dark.set(dark);
        let mode = if dark { "dark" } else { "light" };
        let path = state_file();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&path, mode);
        exec::spawn(
            "gsettings",
            &[
                "set",
                "org.gnome.desktop.interface",
                "color-scheme",
                if dark { "prefer-dark" } else { "prefer-light" },
            ],
        );
        glib::spawn_future_local(async move {
            let query = exec::output("awww", &["query"]).await.unwrap_or_default();
            if let Some(image) = query.lines().find_map(|l| l.split("image: ").nth(1)) {
                exec::spawn(&exec::script("theme.sh"), &[image.trim()]);
            }
        });
    }
}
