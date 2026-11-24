//! The list wofi shows on Mod+D: the spotlight's apps, ranked by how often
//! they are opened from either, so both learn from each other.

use gtk::gio;
use gtk::prelude::*;

use super::apps::Apps;

/// One line per app for `wofi --dmenu`: `img:<icon file>:text:<name>`.
pub fn list() -> String {
    let apps = Apps::load();
    let theme = gtk::gdk::Display::default().map(|d| gtk::IconTheme::for_display(&d));
    let mut out = String::new();
    for index in apps.ranked() {
        let entry = apps.get(index);
        let name = entry.name.replace('\n', " ");
        let icon = entry
            .icon
            .as_ref()
            .and_then(|icon| icon_file(icon, theme.as_ref()))
            .unwrap_or_default();
        if icon.is_empty() {
            out.push_str(&format!("{name}\n"));
        } else {
            out.push_str(&format!("img:{icon}:text:{name}\n"));
        }
    }
    out
}

/// Opens the app on line `line` of the last list.
pub fn launch(line: usize) {
    let mut apps = Apps::load();
    if let Some(index) = apps.ranked().get(line).copied() {
        apps.launch(index);
    }
}

fn icon_file(icon: &gio::Icon, theme: Option<&gtk::IconTheme>) -> Option<String> {
    if let Some(file) = icon.downcast_ref::<gio::FileIcon>() {
        return file.file().path().map(|p| p.to_string_lossy().into_owned());
    }
    let paintable = theme?.lookup_by_gicon(
        icon,
        32,
        1,
        gtk::TextDirection::None,
        gtk::IconLookupFlags::empty(),
    );
    paintable
        .file()
        .and_then(|f| f.path())
        .map(|p| p.to_string_lossy().into_owned())
}
