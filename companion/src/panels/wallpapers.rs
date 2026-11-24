use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::exec;
use crate::i18n::t;

const THUMB: i32 = 320;
const KINDS: [&str; 6] = ["jpg", "jpeg", "png", "webp", "gif", "bmp"];

/// The dash's wallpaper picker: Matuwall's folder as a grid of thumbnails
/// (made once and kept in the cache), applied with awww and then themed.
pub struct Wallpapers {
    pub root: gtk::Box,
    store: gtk::StringList,
    selection: gtk::SingleSelection,
}

/// Matuwall's folder, or the usual places.
fn folder() -> PathBuf {
    let home = glib::home_dir();
    let configured = std::fs::read_to_string(home.join(".config/matuwall/config.toml"))
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "directory").then(|| value.trim().trim_matches('"').to_owned())
            })
        });
    if let Some(dir) = configured {
        return match dir.strip_prefix("~/") {
            Some(rest) => home.join(rest),
            None => PathBuf::from(dir),
        };
    }
    [
        "Imagens/wallpapers",
        "Pictures/wallpapers",
        "Pictures",
        "Imagens",
    ]
    .iter()
    .map(|p| home.join(p))
    .find(|p| p.is_dir())
    .unwrap_or(home)
}

fn thumb_path(path: &Path) -> PathBuf {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    if let Ok(meta) = std::fs::metadata(path) {
        meta.modified().ok().hash(&mut hasher);
    }
    exec::cache_dir()
        .join("thumbs")
        .join(format!("{:x}.png", hasher.finish()))
}

/// Makes the thumbnail if it is not cached yet. Blocks; run it off the
/// main thread.
fn thumbnail(path: &Path) -> Option<PathBuf> {
    let thumb = thumb_path(path);
    if thumb.exists() {
        return Some(thumb);
    }
    let pixbuf = gtk::gdk_pixbuf::Pixbuf::from_file_at_scale(path, THUMB, -1, true).ok()?;
    std::fs::create_dir_all(thumb.parent()?).ok()?;
    pixbuf.savev(&thumb, "png", &[]).ok()?;
    Some(thumb)
}

fn current() -> String {
    std::process::Command::new("awww")
        .arg("query")
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find_map(|l| l.split("image: ").nth(1).map(|p| p.trim().to_owned()))
        })
        .unwrap_or_default()
}

fn apply(path: &str) {
    let script = "awww img --transition-type grow --transition-fps 60 --transition-duration 0.8 \"$1\" && exec \"$HOME/.local/bin/octopus-theme\" \"$1\"";
    exec::spawn("sh", &["-c", script, "sh", path]);
}

impl Wallpapers {
    pub fn new() -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
        root.add_css_class("wallpapers");

        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some(t("buscar wallpaper")));
        root.append(&search);

        let store = gtk::StringList::new(&[]);
        let query: Rc<RefCell<String>> = Rc::default();
        let filter = {
            let query = query.clone();
            gtk::CustomFilter::new(move |item| {
                let q = query.borrow();
                q.is_empty()
                    || item
                        .downcast_ref::<gtk::StringObject>()
                        .is_some_and(|s| s.string().to_lowercase().contains(q.as_str()))
            })
        };
        let filtered = gtk::FilterListModel::new(Some(store.clone()), Some(filter.clone()));
        let selection = gtk::SingleSelection::new(Some(filtered));
        selection.set_autoselect(false);

        let active = Rc::new(RefCell::new(current()));
        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let frame = gtk::Overlay::new();
            frame.add_css_class("wallpaper-thumb");
            frame.set_overflow(gtk::Overflow::Hidden);
            let picture = gtk::Picture::new();
            picture.set_content_fit(gtk::ContentFit::Cover);
            picture.set_size_request(240, 140);
            frame.set_child(Some(&picture));
            let check = crate::widgets::shape::Shape::new(
                crate::widgets::shape::Kind::Cookie9,
                30,
                true,
                Some(&crate::icons::icon("check")),
            );
            check.add_css_class("wallpaper-check");
            check.set_halign(gtk::Align::End);
            check.set_valign(gtk::Align::Start);
            check.set_margin_top(8);
            check.set_margin_end(8);
            frame.add_overlay(&check);
            item.downcast_ref::<gtk::ListItem>()
                .unwrap()
                .set_child(Some(&frame));
        });
        let shown = active.clone();
        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let Some(path) = item
                .item()
                .and_downcast::<gtk::StringObject>()
                .map(|s| s.string())
            else {
                return;
            };
            let Some(frame) = item.child().and_downcast::<gtk::Overlay>() else {
                return;
            };
            let Some(picture) = frame.child().and_downcast::<gtk::Picture>() else {
                return;
            };
            if let Some(check) = frame.last_child() {
                check.set_visible(*shown.borrow() == path.as_str());
            }
            frame.set_tooltip_text(
                Path::new(path.as_str())
                    .file_name()
                    .and_then(|n| n.to_str()),
            );
            picture.set_paintable(None::<&gdk::Paintable>);
            let picture = picture.downgrade();
            let wanted = path.to_string();
            glib::spawn_future_local(async move {
                let source = PathBuf::from(&wanted);
                let thumb = gio::spawn_blocking(move || thumbnail(&source))
                    .await
                    .ok()
                    .flatten();
                let (Some(picture), Some(thumb)) = (picture.upgrade(), thumb) else {
                    return;
                };
                let still = picture
                    .parent()
                    .and_then(|f| f.tooltip_text())
                    .is_some_and(|tip| wanted.ends_with(tip.as_str()));
                if still {
                    if let Ok(texture) = gdk::Texture::from_filename(&thumb) {
                        picture.set_paintable(Some(&texture));
                    }
                }
            });
        });

        let grid = gtk::GridView::new(Some(selection.clone()), Some(factory));
        grid.set_max_columns(3);
        grid.set_min_columns(3);
        grid.set_single_click_activate(true);
        grid.add_css_class("wallpaper-grid");
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_height(440)
            .max_content_height(440)
            .child(&grid)
            .build();
        root.append(&scroller);

        let panel = Rc::new(Self {
            root,
            store,
            selection: selection.clone(),
        });

        search.connect_search_changed(move |entry| {
            *query.borrow_mut() = entry.text().to_lowercase();
            filter.changed(gtk::FilterChange::Different);
        });
        let weak = Rc::downgrade(&panel);
        grid.connect_activate(move |grid, position| {
            let Some(p) = weak.upgrade() else { return };
            let Some(path) = p
                .selection
                .item(position)
                .and_downcast::<gtk::StringObject>()
                .map(|s| s.string().to_string())
            else {
                return;
            };
            apply(&path);
            *active.borrow_mut() = path;
            let model = grid.model();
            if let Some(model) = model {
                model.items_changed(0, model.n_items(), model.n_items());
            }
        });
        panel.load();
        panel
    }

    fn load(&self) {
        let dir = folder();
        let mut files: Vec<String> = std::fs::read_dir(&dir)
            .map(|entries| {
                entries
                    .flatten()
                    .map(|e| e.path())
                    .filter(|p| {
                        p.extension()
                            .and_then(|e| e.to_str())
                            .is_some_and(|e| KINDS.contains(&e.to_lowercase().as_str()))
                    })
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect()
            })
            .unwrap_or_default();
        files.sort();
        let refs: Vec<&str> = files.iter().map(String::as_str).collect();
        self.store.splice(0, self.store.n_items(), &refs);
    }
}
