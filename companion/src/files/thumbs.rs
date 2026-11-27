//! Thumbnails, from and into the freedesktop cache (~/.cache/thumbnails), so
//! they are shared with every other file manager. Images are scaled here,
//! videos go through ffmpegthumbnailer; two at a time, off the main thread.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

type Done = Box<dyn FnOnce(Option<gdk::Texture>)>;

thread_local! {
    static QUEUE: RefCell<VecDeque<(gio::File, String, Done)>> = RefCell::default();
    static RUNNING: Cell<usize> = const { Cell::new(0) };
}

const WORKERS: usize = 2;

fn cache_path(uri: &str) -> PathBuf {
    let sum = glib::compute_checksum_for_string(glib::ChecksumType::Md5, uri).unwrap_or_default();
    glib::user_cache_dir().join("thumbnails/large").join(format!("{sum}.png"))
}

/// Whether a file of this type gets a thumbnail at all.
pub fn wanted(content_type: &str) -> bool {
    content_type.starts_with("image/") || content_type.starts_with("video/")
}

/// Calls `done` with a thumbnail, now if it's cached, later if it has to
/// be made; None when there is none.
pub fn request(file: &gio::File, info: &gio::FileInfo, done: impl FnOnce(Option<gdk::Texture>) + 'static) {
    let content_type = info.content_type().map(|c| c.to_string()).unwrap_or_default();
    if !wanted(&content_type) || file.path().is_none() {
        return done(None);
    }
    let cached = info
        .attribute_byte_string("thumbnail::path")
        .map(|p| PathBuf::from(p.as_str()))
        .filter(|p| p.exists())
        .unwrap_or_else(|| cache_path(&file.uri()));
    if cached.exists() {
        return load(cached, Box::new(done));
    }
    QUEUE.with(|q| q.borrow_mut().push_back((file.clone(), content_type, Box::new(done))));
    pump();
}

fn load(path: PathBuf, done: Done) {
    glib::spawn_future_local(async move {
        let texture = gio::spawn_blocking(move || gdk::Texture::from_filename(&path).ok()).await.ok().flatten();
        done(texture);
    });
}

fn pump() {
    while RUNNING.with(|r| r.get()) < WORKERS {
        let Some((file, content_type, done)) = QUEUE.with(|q| q.borrow_mut().pop_front()) else { return };
        RUNNING.with(|r| r.set(r.get() + 1));
        glib::spawn_future_local(async move {
            let uri = file.uri().to_string();
            let source = file.path();
            let target = cache_path(&uri);
            let made = gio::spawn_blocking(move || make(source?, &content_type, &target, &uri).then_some(target))
                .await
                .ok()
                .flatten();
            RUNNING.with(|r| r.set(r.get() - 1));
            match made {
                Some(path) => load(path, done),
                None => done(None),
            }
            pump();
        });
    }
}

fn make(source: PathBuf, content_type: &str, target: &std::path::Path, uri: &str) -> bool {
    if let Some(dir) = target.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if content_type.starts_with("video/") {
        return std::process::Command::new("ffmpegthumbnailer")
            .args(["-s", "256", "-i"])
            .arg(&source)
            .arg("-o")
            .arg(target)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
    }
    let Ok(pixbuf) = gdk::gdk_pixbuf::Pixbuf::from_file_at_scale(&source, 256, 256, true) else { return false };
    let mtime = std::fs::metadata(&source)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default();
    pixbuf
        .savev(target, "png", &[("tEXt::Thumb::URI", uri), ("tEXt::Thumb::MTime", &mtime)])
        .is_ok()
}
