//! A note on the desktop: type in it, and it is kept as you go in
//! ~/.local/state/octopus/note.txt.

use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;

fn path() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| gtk::glib::home_dir().join(".local/state"))
        .join("octopus/note.txt")
}

pub fn build() -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 8);
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-note");
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let glyph = icons::icon("sticky_note_2");
    glyph.add_css_class("note-icon");
    head.append(&glyph);
    let title = gtk::Label::new(Some(t("nota")));
    title.add_css_class("desktop-title");
    title.set_xalign(0.0);
    head.append(&title);
    root.append(&head);

    let text = gtk::TextView::new();
    text.add_css_class("note-text");
    text.set_wrap_mode(gtk::WrapMode::WordChar);
    text.set_vexpand(true);
    let buffer = text.buffer();
    buffer.set_text(&std::fs::read_to_string(path()).unwrap_or_default());
    let save = crate::exec::debounce(600, {
        let buffer = buffer.clone();
        move || {
            let (start, end) = buffer.bounds();
            let content = buffer.text(&start, &end, false);
            let path = path();
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(path, content.as_str());
        }
    });
    buffer.connect_changed(move |_| save());
    let scroll = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&text).build();
    scroll.set_vexpand(true);
    root.append(&scroll);
    root
}
