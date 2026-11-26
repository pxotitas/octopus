//! A picture on the desktop, cropped to fill the card. Its button picks
//! another; the choice is kept in ~/.local/state/octopus/photo.

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::i18n::t;
use crate::icons;

fn path() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| glib::home_dir().join(".local/state"))
        .join("octopus/photo")
}

/// The chosen picture, or the first one in ~/Pictures (or ~/Imagens).
fn chosen() -> Option<std::path::PathBuf> {
    if let Some(saved) = std::fs::read_to_string(path()).ok().map(|s| std::path::PathBuf::from(s.trim())) {
        if saved.exists() {
            return Some(saved);
        }
    }
    let pictures = glib::user_special_dir(glib::UserDirectory::Pictures)?;
    let image = |p: &std::path::Path| p.extension().is_some_and(|x| matches!(x.to_string_lossy().to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp"));
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&pictures).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Ok(inner) = std::fs::read_dir(&path) {
                found.extend(inner.flatten().map(|e| e.path()).filter(|p| image(p)));
            }
        } else if image(&path) {
            found.push(path);
        }
    }
    found.sort();
    found.into_iter().next()
}

/// Loads off the main thread: gtk's own loader leaves a placeholder for some
/// files (lossless webp with alpha, for one), a texture made directly doesn't.
fn show(picture: &gtk::Picture, file: std::path::PathBuf) {
    let picture = picture.clone();
    glib::spawn_future_local(async move {
        if let Ok(Ok(texture)) = gio::spawn_blocking(move || gtk::gdk::Texture::from_filename(&file)).await {
            picture.set_paintable(Some(&texture));
        }
    });
}

pub fn build() -> gtk::Overlay {
    let root = gtk::Overlay::new();
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-photo");
    root.set_overflow(gtk::Overflow::Hidden);

    let picture = gtk::Picture::new();
    picture.set_content_fit(gtk::ContentFit::Cover);
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    let empty = gtk::Label::new(Some(t("escolha uma foto")));
    empty.add_css_class("dim");
    let missing = chosen().is_none();
    match chosen() {
        Some(file) => show(&picture, file),
        None => root.add_overlay(&empty),
    }
    root.set_child(Some(&picture));

    let change = icons::button("add_photo_alternate", t("trocar a foto"));
    change.add_css_class("photo-change");
    if missing {
        change.add_css_class("shown");
    }
    change.set_halign(gtk::Align::End);
    change.set_valign(gtk::Align::End);
    change.set_margin_end(10);
    change.set_margin_bottom(10);
    let (pic, label) = (picture.clone(), empty.clone());
    change.connect_clicked(move |button| {
        let filter = gtk::FileFilter::new();
        filter.add_mime_type("image/*");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder().title(t("trocar a foto")).filters(&filters).modal(false).build();
        if let Some(dir) = glib::user_special_dir(glib::UserDirectory::Pictures) {
            dialog.set_initial_folder(Some(&gio::File::for_path(dir)));
        }
        let (pic, label) = (pic.clone(), label.clone());
        let window = button.root().and_downcast::<gtk::Window>();
        dialog.open(window.as_ref(), gio::Cancellable::NONE, move |result| {
            let Some(file) = result.ok().and_then(|f| f.path()) else { return };
            show(&pic, file.clone());
            label.set_visible(false);
            let path = path();
            if let Some(dir) = path.parent() {
                let _ = std::fs::create_dir_all(dir);
            }
            let _ = std::fs::write(path, file.to_string_lossy().as_bytes());
        });
    });
    root.add_overlay(&change);
    root
}
