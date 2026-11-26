//! A picture on the desktop, cropped to fill the card or shown whole. Its
//! button picks another; the choice is kept in ~/.local/state/octopus/photo,
//! watched, so the settings can change it too. It can also go through the
//! chosen picture's folder, one every few minutes.

use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::config::DeskPhoto;
use crate::i18n::t;
use crate::icons;

pub fn path() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| glib::home_dir().join(".local/state"))
        .join("octopus/photo")
}

/// Saves the choice; a desktop showing a photo picks it up on its own.
pub fn choose(file: &std::path::Path) {
    let path = path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, file.to_string_lossy().as_bytes());
}

fn is_image(p: &std::path::Path) -> bool {
    p.extension().is_some_and(|x| matches!(x.to_string_lossy().to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp" | "avif" | "jxl"))
}

fn images_in(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut found: Vec<_> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| is_image(p))
        .collect();
    found.sort();
    found
}

/// The chosen picture, or the first one in Pictures or a folder inside it.
pub fn chosen() -> Option<std::path::PathBuf> {
    if let Some(saved) = std::fs::read_to_string(path()).ok().map(|s| std::path::PathBuf::from(s.trim())) {
        if saved.exists() {
            return Some(saved);
        }
    }
    let pictures = glib::user_special_dir(glib::UserDirectory::Pictures)?;
    let mut found = images_in(&pictures);
    for entry in std::fs::read_dir(&pictures).ok()?.flatten() {
        if entry.path().is_dir() {
            found.extend(images_in(&entry.path()));
        }
    }
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

pub fn build(options: &DeskPhoto) -> gtk::Overlay {
    let root = gtk::Overlay::new();
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-photo");
    root.set_overflow(gtk::Overflow::Hidden);

    let picture = gtk::Picture::new();
    let contain = options.fit == "contain";
    picture.set_content_fit(if contain { gtk::ContentFit::Contain } else { gtk::ContentFit::Cover });
    if contain {
        root.add_css_class("contain");
    }
    picture.set_can_shrink(true);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    root.set_child(Some(&picture));

    let empty = gtk::Label::new(Some(t("escolha uma foto")));
    empty.add_css_class("dim");
    root.add_overlay(&empty);

    let change = icons::button("add_photo_alternate", t("trocar a foto"));
    change.add_css_class("photo-change");
    change.set_halign(gtk::Align::End);
    change.set_valign(gtk::Align::End);
    change.set_margin_end(10);
    change.set_margin_bottom(10);

    let refresh: Rc<dyn Fn()> = {
        let (picture, empty, change) = (picture.clone(), empty.clone(), change.clone());
        Rc::new(move || match chosen() {
            Some(file) => {
                show(&picture, file);
                empty.set_visible(false);
                change.remove_css_class("shown");
            }
            None => {
                empty.set_visible(true);
                change.add_css_class("shown");
            }
        })
    };
    refresh();

    // the dialog has no parent: it can't be a child of a layer surface, and
    // asking for that makes the compositor drop the whole companion
    change.connect_clicked(move |_| {
        let filter = gtk::FileFilter::new();
        filter.add_mime_type("image/*");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder().title(t("trocar a foto")).filters(&filters).modal(false).build();
        let start = chosen().and_then(|f| f.parent().map(|d| d.to_path_buf())).or_else(|| glib::user_special_dir(glib::UserDirectory::Pictures));
        if let Some(dir) = start {
            dialog.set_initial_folder(Some(&gio::File::for_path(dir)));
        }
        dialog.open(None::<&gtk::Window>, gio::Cancellable::NONE, move |result| {
            if let Some(file) = result.ok().and_then(|f| f.path()) {
                choose(&file);
            }
        });
    });
    root.add_overlay(&change);

    let state = gio::File::for_path(path());
    if let Ok(monitor) = state.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
        let refresh = refresh.clone();
        monitor.connect_changed(move |_, _, _, event| {
            if matches!(event, gio::FileMonitorEvent::ChangesDoneHint | gio::FileMonitorEvent::Created) {
                refresh();
            }
        });
        // kept alive with the widget
        unsafe { root.set_data("monitor", monitor) };
    }

    if options.shuffle > 0 {
        let every = Duration::from_secs(options.shuffle as u64 * 60);
        let weak = root.downgrade();
        glib::timeout_add_local(every, move || {
            let Some(root) = weak.upgrade() else { return glib::ControlFlow::Break };
            if root.is_mapped() {
                let current = chosen();
                if let Some(dir) = current.as_ref().and_then(|f| f.parent()) {
                    let all = images_in(dir);
                    if all.len() > 1 {
                        let start = current.as_ref().and_then(|c| all.iter().position(|p| p == c)).unwrap_or(0);
                        let mut next = (glib::random_int_range(0, all.len() as i32 - 1) as usize + start + 1) % all.len();
                        if next == start {
                            next = (start + 1) % all.len();
                        }
                        choose(&all[next]);
                    }
                }
            }
            glib::ControlFlow::Continue
        });
    }
    root
}
