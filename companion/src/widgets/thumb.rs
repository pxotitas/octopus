//! Images at the size they are shown, not the size they were saved: decoded
//! off the main thread, scaled down once, shared between every picture that
//! shows the same file, and for the ones that come back (the desktop photo)
//! kept as a small png in ~/.cache/octopus/images.

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use gtk::gdk_pixbuf::Pixbuf;
use gtk::prelude::*;
use gtk::{gdk, gio, glib};

const KEPT: usize = 24;

thread_local! {
    static LOADED: RefCell<HashMap<(PathBuf, i32), glib::WeakRef<gdk::Texture>>> = RefCell::default();
}

fn cached(path: &Path, px: i32) -> Option<gdk::Texture> {
    LOADED.with(|m| m.borrow().get(&(path.to_owned(), px)).and_then(|w| w.upgrade()))
}

fn remember(path: &Path, px: i32, texture: &gdk::Texture) {
    LOADED.with(|m| {
        let mut m = m.borrow_mut();
        m.retain(|_, w| w.upgrade().is_some());
        m.insert((path.to_owned(), px), texture.downgrade());
    });
}

fn dir() -> PathBuf {
    crate::exec::cache_dir().join("images")
}

/// Named after the file, its size and when it changed, so an edited file
/// gets a new one.
fn kept_at(path: &Path, px: i32) -> Option<PathBuf> {
    let meta = std::fs::metadata(path).ok()?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut h);
    meta.len().hash(&mut h);
    meta.modified().ok()?.hash(&mut h);
    px.hash(&mut h);
    Some(dir().join(format!("{:016x}.png", h.finish())))
}

fn prune() {
    let Ok(entries) = std::fs::read_dir(dir()) else { return };
    let mut files: Vec<_> = entries
        .flatten()
        .filter_map(|e| Some((e.metadata().ok()?.modified().ok()?, e.path())))
        .collect();
    if files.len() <= KEPT {
        return;
    }
    files.sort();
    for (_, old) in &files[..files.len() - KEPT] {
        let _ = std::fs::remove_file(old);
    }
}

/// Its shorter side at `px`, enough to cover a square that big.
fn decode(path: &Path, px: i32) -> Option<Pixbuf> {
    let (_, w, h) = Pixbuf::file_info(path)?;
    if w <= 0 || h <= 0 {
        return None;
    }
    let short = w.min(h);
    if short <= px {
        return Pixbuf::from_file(path).ok();
    }
    let scale = px as f64 / short as f64;
    let (tw, th) = ((w as f64 * scale).round() as i32, (h as f64 * scale).round() as i32);
    Pixbuf::from_file_at_scale(path, tw.max(1), th.max(1), false).ok()
}

/// Runs off the main thread. Falls back to the whole image when the pixbuf
/// loaders can't read it.
fn load(path: &Path, px: i32, keep: bool) -> Option<gdk::Texture> {
    let png = keep.then(|| kept_at(path, px)).flatten();
    if let Some(png) = &png {
        if let Ok(texture) = gdk::Texture::from_filename(png) {
            return Some(texture);
        }
    }
    match decode(path, px) {
        Some(pixbuf) => {
            if let Some(png) = &png {
                let _ = std::fs::create_dir_all(dir());
                if pixbuf.savev(png, "png", &[]).is_ok() {
                    prune();
                }
            }
            #[allow(deprecated)]
            Some(gdk::Texture::for_pixbuf(&pixbuf))
        }
        None => gdk::Texture::from_filename(path).ok(),
    }
}

/// Shows `path` in `picture` at about `px` device pixels on its shorter
/// side; `keep` saves the small copy for next time. Asking again for the
/// file already shown does nothing.
pub fn set(picture: &gtk::Picture, path: Option<&Path>, px: i32, keep: bool) {
    let px = px * picture.scale_factor().max(1);
    let wanted = path.map(|p| (p.to_owned(), px));
    let shown: Option<(PathBuf, i32)> = unsafe { picture.data::<(PathBuf, i32)>("thumb").map(|p| p.as_ref().clone()) };
    if wanted.is_some() && shown == wanted {
        return;
    }
    match wanted {
        None => {
            unsafe { picture.steal_data::<(PathBuf, i32)>("thumb") };
            picture.set_paintable(None::<&gdk::Paintable>);
        }
        Some((path, px)) => {
            unsafe { picture.set_data("thumb", (path.clone(), px)) };
            if let Some(texture) = cached(&path, px) {
                picture.set_paintable(Some(&texture));
                return;
            }
            let picture = picture.clone();
            glib::spawn_future_local(async move {
                let file = path.clone();
                let Ok(Some(texture)) = gio::spawn_blocking(move || load(&file, px, keep)).await else { return };
                let still: Option<(PathBuf, i32)> =
                    unsafe { picture.data::<(PathBuf, i32)>("thumb").map(|p| p.as_ref().clone()) };
                if still.as_ref().is_some_and(|(p, _)| *p == path) {
                    remember(&path, px, &texture);
                    picture.set_paintable(Some(&texture));
                }
            });
        }
    }
}
