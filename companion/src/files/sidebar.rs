//! The sidebar, like Finder's: favourites, bookmarks (the gtk ones, shared
//! with every file manager), disks and mounts, the network, and tags.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use super::{display_name, symbol_of, tags};
use crate::i18n::t;
use crate::icons;

#[derive(Clone)]
pub enum Place {
    Folder(gio::File),
    Tag(&'static str),
    Network,
    Connect,
    Drive,
}

pub struct Sidebar {
    pub root: gtk::ScrolledWindow,
    list: gtk::Box,
    rows: std::cell::RefCell<Vec<(gtk::Button, Option<gio::File>)>>,
    picked: Rc<dyn Fn(Place)>,
    dropped: super::cells::OnDrop,
    _monitors: std::cell::RefCell<Vec<glib::Object>>,
}

fn bookmarks_path() -> std::path::PathBuf {
    glib::user_config_dir().join("gtk-3.0/bookmarks")
}

pub fn bookmarks() -> Vec<(gio::File, String)> {
    std::fs::read_to_string(bookmarks_path())
        .unwrap_or_default()
        .lines()
        .filter_map(|line| {
            let (uri, name) = line.split_once(' ').unwrap_or((line, ""));
            (!uri.is_empty()).then(|| {
                let file = gio::File::for_uri(uri);
                let name = if name.is_empty() { display_name(&file) } else { name.to_owned() };
                (file, name)
            })
        })
        .collect()
}

pub fn toggle_bookmark(file: &gio::File) {
    let mut lines: Vec<String> = std::fs::read_to_string(bookmarks_path()).unwrap_or_default().lines().map(str::to_owned).collect();
    let uri = file.uri().to_string();
    let before = lines.len();
    lines.retain(|l| l.split(' ').next() != Some(uri.as_str()));
    if lines.len() == before {
        lines.push(uri);
    }
    let _ = std::fs::create_dir_all(bookmarks_path().parent().unwrap_or(std::path::Path::new("/tmp")));
    let _ = std::fs::write(bookmarks_path(), lines.join("\n") + "\n");
}

impl Sidebar {
    pub fn new(picked: Rc<dyn Fn(Place)>, dropped: super::cells::OnDrop) -> Rc<Self> {
        let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
        list.add_css_class("files-sidebar");
        let root = gtk::ScrolledWindow::builder().hscrollbar_policy(gtk::PolicyType::Never).child(&list).build();
        root.add_css_class("files-sidebar-scroll");
        let sidebar = Rc::new(Self { root, list, rows: Default::default(), picked, dropped, _monitors: Default::default() });
        sidebar.fill();

        // refill when bookmarks or mounts change
        let refill = {
            let weak = Rc::downgrade(&sidebar);
            move || {
                if let Some(s) = weak.upgrade() {
                    s.fill();
                }
            }
        };
        let refill = Rc::new(refill);
        if let Ok(monitor) = gio::File::for_path(bookmarks_path()).monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
            let r = refill.clone();
            monitor.connect_changed(move |_, _, _, _| r());
            sidebar._monitors.borrow_mut().push(monitor.upcast());
        }
        let volumes = gio::VolumeMonitor::get();
        let r = refill.clone();
        volumes.connect_mount_added(move |_, _| r());
        let r = refill.clone();
        volumes.connect_mount_removed(move |_, _| r());
        let r = refill.clone();
        volumes.connect_volume_added(move |_, _| r());
        volumes.connect_volume_removed(move |_, _| refill());
        sidebar._monitors.borrow_mut().push(volumes.upcast());
        sidebar
    }

    fn heading(&self, text: &str) {
        let label = gtk::Label::new(Some(text));
        label.add_css_class("files-sidebar-heading");
        label.set_xalign(0.0);
        self.list.append(&label);
    }

    fn row(&self, symbol: &str, name: &str, place: Place, trailing: Option<gtk::Widget>) {
        let button = gtk::Button::new();
        button.add_css_class("files-sidebar-row");
        let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let glyph = icons::icon(symbol);
        glyph.add_css_class("files-sidebar-icon");
        let label = gtk::Label::new(Some(name));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        line.append(&glyph);
        line.append(&label);
        if let Some(w) = trailing {
            line.append(&w);
        }
        button.set_child(Some(&line));
        let file = match &place {
            Place::Folder(f) => Some(f.clone()),
            _ => None,
        };
        if let Some(folder) = file.clone() {
            // files dropped on a place go there
            let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY | gdk::DragAction::MOVE);
            let (dropped, b) = (self.dropped.clone(), button.clone());
            target.connect_drop(move |t, value, _, _| {
                b.remove_css_class("drop-hover");
                let Ok(list) = value.get::<gdk::FileList>() else { return false };
                let copy = t.current_drop().is_some_and(|d| d.actions() == gdk::DragAction::COPY);
                dropped(list.files(), folder.clone(), copy);
                true
            });
            let b = button.clone();
            target.connect_enter(move |_, _, _| {
                b.add_css_class("drop-hover");
                gdk::DragAction::MOVE
            });
            let b = button.clone();
            target.connect_leave(move |_| b.remove_css_class("drop-hover"));
            button.add_controller(target);
        }
        let picked = self.picked.clone();
        button.connect_clicked(move |_| picked(place.clone()));
        self.list.append(&button);
        self.rows.borrow_mut().push((button, file));
    }

    fn fill(&self) {
        crate::widgets::clear(&self.list);
        self.rows.borrow_mut().clear();

        self.heading(t("favoritos"));
        let home = gio::File::for_path(glib::home_dir());
        self.row("schedule", t("recentes"), Place::Folder(gio::File::for_uri("recent:///")), None);
        self.row("home", t("pasta pessoal"), Place::Folder(home), None);
        for dir in [
            glib::UserDirectory::Desktop,
            glib::UserDirectory::Documents,
            glib::UserDirectory::Downloads,
            glib::UserDirectory::Pictures,
            glib::UserDirectory::Music,
            glib::UserDirectory::Videos,
        ] {
            if let Some(path) = glib::user_special_dir(dir).filter(|p| p.exists() && *p != glib::home_dir()) {
                let file = gio::File::for_path(&path);
                self.row(symbol_of(&file), &display_name(&file), Place::Folder(file.clone()), None);
            }
        }

        let marks = bookmarks();
        if !marks.is_empty() {
            self.heading(t("marcadores"));
            for (file, name) in marks {
                self.row("bookmark", &name, Place::Folder(file), None);
            }
        }

        self.heading(t("locais"));
        self.row("computer", t("computador"), Place::Folder(gio::File::for_path("/")), None);
        for mount in gio::VolumeMonitor::get().mounts() {
            if mount.is_shadowed() {
                continue;
            }
            let root = mount.root();
            let eject = icons::button(if mount.can_eject() { "eject" } else { "do_not_disturb_on" }, t("ejetar"));
            eject.add_css_class("files-eject");
            let m = mount.clone();
            eject.connect_clicked(move |_| {
                let op = gtk::MountOperation::new(None::<&gtk::Window>);
                if m.can_eject() {
                    m.eject_with_operation(gio::MountUnmountFlags::NONE, Some(&op), gio::Cancellable::NONE, |_| {});
                } else {
                    m.unmount_with_operation(gio::MountUnmountFlags::NONE, Some(&op), gio::Cancellable::NONE, |_| {});
                }
            });
            let symbol = match root.uri_scheme().as_deref() {
                Some("sftp" | "ftp" | "smb" | "dav" | "davs") => "lan",
                _ => "hard_drive",
            };
            self.row(symbol, &mount.name(), Place::Folder(root), Some(eject.upcast()));
        }
        // volumes that aren't mounted yet: a click mounts them
        for volume in gio::VolumeMonitor::get().volumes() {
            if volume.get_mount().is_some() || !volume.can_mount() {
                continue;
            }
            let button = gtk::Button::new();
            button.add_css_class("files-sidebar-row");
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            let glyph = icons::icon("hard_drive");
            glyph.add_css_class("files-sidebar-icon");
            line.append(&glyph);
            let label = gtk::Label::new(Some(&volume.name()));
            label.set_xalign(0.0);
            label.add_css_class("dim");
            line.append(&label);
            button.set_child(Some(&line));
            let (v, picked) = (volume.clone(), self.picked.clone());
            button.connect_clicked(move |_| {
                let picked = picked.clone();
                let v2 = v.clone();
                v.mount(gio::MountMountFlags::NONE, Some(&gtk::MountOperation::new(None::<&gtk::Window>)), gio::Cancellable::NONE, move |r| {
                    if r.is_ok() {
                        if let Some(m) = v2.get_mount() {
                            picked(Place::Folder(m.root()));
                        }
                    }
                });
            });
            self.list.append(&button);
        }
        self.row("delete", t("lixeira"), Place::Folder(gio::File::for_uri("trash:///")), None);

        self.heading(t("rede"));
        self.row("lan", t("na rede"), Place::Network, None);
        self.row("dns", t("conectar a servidor"), Place::Connect, None);
        self.row("add_to_drive", "google drive", Place::Drive, None);

        self.heading(t("tags"));
        for (id, name, color) in tags::COLORS {
            let dot = gtk::DrawingArea::new();
            dot.set_content_width(10);
            dot.set_content_height(10);
            let rgba = gdk::RGBA::parse(color).unwrap_or(gdk::RGBA::BLACK);
            dot.set_draw_func(move |_, cr, w, h| {
                cr.set_source_rgba(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64, 1.0);
                cr.arc(w as f64 / 2.0, h as f64 / 2.0, 5.0, 0.0, std::f64::consts::TAU);
                let _ = cr.fill();
            });
            dot.set_valign(gtk::Align::Center);
            self.row("label", t(name), Place::Tag(id), Some(dot.upcast()));
        }
    }

    /// Lights up the row of the folder shown.
    pub fn highlight(&self, location: &gio::File) {
        for (button, file) in self.rows.borrow().iter() {
            if file.as_ref().is_some_and(|f| f.equal(location)) {
                button.add_css_class("current");
            } else {
                button.remove_css_class("current");
            }
        }
    }
}
