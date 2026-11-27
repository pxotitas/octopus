//! Arquivos: the file manager. Its own process (`octopus files [path]`), so a
//! heavy folder never stalls the shell, drawn with the shell's colors,
//! symbols and shapes. Finder's ideas, Material's looks.

mod cells;
mod info;
mod menu;
mod ops;
mod pane;
mod quicklook;
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
    app.connect_startup(|app| {
        crate::theme::install();
        crate::i18n::init(&crate::config::load().language);
        menu::accels(app);
    });
    app.connect_activate(|app| {
        window::Window::open(app, None);
    });
    app.connect_open(|app, files, _| {
        for file in files {
            window::Window::open(app, Some(file.clone()));
        }
    });
    let argv: Vec<String> = std::iter::once("octopus-files".to_owned()).chain(args.iter().cloned()).collect();
    app.run_with_args(&argv)
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
