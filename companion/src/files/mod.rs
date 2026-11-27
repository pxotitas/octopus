//! Arquivos: the file manager. Its own process (`octopus files [path]`), so a
//! heavy folder never stalls the shell, drawn with the shell's colors,
//! symbols and shapes. Finder's ideas, Material's looks.

mod cells;
mod dbus;
mod drive;
mod git;
mod info;
mod menu;
mod network;
mod ops;
mod pane;
mod quicklook;
mod share;
mod sidebar;
mod state;
mod tags;
mod thumbs;
mod window;

use gtk::prelude::*;
use gtk::{gio, glib};

pub const APP_ID: &str = "dev.pxota.Octopus.Files";

/// Attributes every listing asks for.
pub const ATTRIBUTES: &str = "standard::name,standard::display-name,standard::type,standard::size,standard::content-type,\
standard::icon,standard::is-hidden,standard::is-backup,standard::is-symlink,standard::target-uri,time::modified,\
thumbnail::path,thumbnail::failed,xattr::xdg.tags,trash::orig-path,trash::deletion-date,id::filesystem,access::can-write";

/// ~/.local/state (or $XDG_STATE_HOME).
pub fn state_dir() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME").map(std::path::PathBuf::from).unwrap_or_else(|| glib::home_dir().join(".local/state"))
}

pub fn run(args: &[String]) -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::HANDLES_OPEN)
        .build();
    // started by d-bus to answer "show in folder": no window of its own
    let service = args.iter().any(|a| a == "--service");
    let args: Vec<String> = args.iter().filter(|a| *a != "--service").cloned().collect();
    app.connect_startup(move |app| {
        crate::theme::install();
        crate::i18n::init(&crate::config::load().language);
        menu::accels(app);
        let app2 = app.clone();
        dbus::serve(move |ask| match ask {
            dbus::Ask::Folders(uris) => {
                for f in dbus::files(&uris) {
                    window::Window::open(&app2, Some(f));
                }
            }
            dbus::Ask::Items(uris) => {
                for f in dbus::files(&uris) {
                    if let Some(parent) = f.parent() {
                        let w = window::Window::open(&app2, Some(parent));
                        w.select_soon(f);
                    }
                }
            }
            dbus::Ask::Properties(uris) => {
                for f in dbus::files(&uris) {
                    let w = window::Window::open(&app2, f.parent());
                    info::show(w.window.upcast_ref(), &f);
                }
            }
        });
        if service {
            let guard = app.hold();
            glib::timeout_add_local_once(std::time::Duration::from_secs(20), move || drop(guard));
        }
    });
    app.connect_activate(move |app| {
        if !service || app.windows().is_empty() && !service {
            window::Window::open(app, None);
        }
    });
    app.connect_open(|app, files, _| {
        for file in files {
            window::Window::open(app, Some(file.clone()));
        }
    });
    let argv: Vec<String> = std::iter::once("octopus-files".to_owned()).chain(args.iter().cloned()).collect();
    app.run_with_args(&argv)
}

/// The terminal to open folders and logins in.
pub fn terminal() -> Option<String> {
    std::env::var("TERMINAL").ok().filter(|t| !t.is_empty()).or_else(|| {
        ["kitty", "alacritty", "foot", "ghostty", "wezterm", "gnome-terminal", "konsole", "xterm"]
            .into_iter()
            .find(|t| glib::find_program_in_path(t).is_some())
            .map(str::to_owned)
    })
}

/// Runs a shell line in a terminal window, each terminal its own way.
pub fn run_in_terminal(script: &str) {
    let Some(term) = terminal() else { return };
    let name = std::path::Path::new(&term).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let mut command = std::process::Command::new(&term);
    match name.as_str() {
        "kitty" | "foot" | "ghostty" => command.args(["sh", "-c", script]),
        "wezterm" => command.args(["start", "--", "sh", "-c", script]),
        "gnome-terminal" | "konsole" => command.args(["--", "sh", "-c", script]),
        _ => command.args(["-e", "sh", "-c", script]),
    };
    let _ = command.spawn();
}

/// The file a listing item stands for.
pub fn file_of(info: &gio::FileInfo) -> Option<gio::File> {
    info.attribute_object("standard::file").and_downcast::<gio::File>()
}

/// "Downloads", "~", "/": a folder's name the way it's shown.
pub fn display_name(file: &gio::File) -> String {
    if file.path().is_some_and(|p| p == glib::home_dir()) {
        return crate::i18n::t("pasta pessoal").to_owned();
    }
    match file.uri().as_str() {
        "trash:///" => return crate::i18n::t("lixeira").to_owned(),
        "recent:///" => return crate::i18n::t("recentes").to_owned(),
        "file:///" => return crate::i18n::t("computador").to_owned(),
        _ => {}
    }
    file.basename()
        .map(|b| b.to_string_lossy().into_owned())
        .filter(|b| b != "/")
        .unwrap_or_else(|| file.parse_name().to_string())
}

/// A material symbol for a place.
pub fn symbol_of(file: &gio::File) -> &'static str {
    let home = glib::home_dir();
    let special = |dir: glib::UserDirectory| glib::user_special_dir(dir).is_some_and(|d| file.path().as_deref() == Some(d.as_path()));
    if file.path().is_some_and(|p| p == home) {
        "home"
    } else if special(glib::UserDirectory::Desktop) {
        "desktop_windows"
    } else if special(glib::UserDirectory::Documents) {
        "description"
    } else if special(glib::UserDirectory::Downloads) {
        "download"
    } else if special(glib::UserDirectory::Pictures) {
        "image"
    } else if special(glib::UserDirectory::Music) {
        "music_note"
    } else if special(glib::UserDirectory::Videos) {
        "movie"
    } else if file.uri_scheme().as_deref() == Some("trash") {
        "delete"
    } else if file.uri_scheme().as_deref() == Some("recent") {
        "schedule"
    } else if file.uri() == "file:///" {
        "computer"
    } else {
        "folder"
    }
}
