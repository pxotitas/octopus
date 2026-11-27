//! What's on the network: computers announcing ssh/sftp or smb (through
//! avahi), and devices with LocalSend open. A click opens a computer's files.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::i18n::t;
use crate::icons;

struct Service {
    name: String,
    host: String,
    port: String,
    kind: &'static str,
}

fn browse() -> Vec<Service> {
    let Ok(out) = std::process::Command::new("avahi-browse").args(["-aprt", "--no-db-lookup"]).output() else { return Vec::new() };
    let mut found: Vec<Service> = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let f: Vec<&str> = line.split(';').collect();
        if f.len() < 9 || f[0] != "=" || f[2] != "IPv4" {
            continue;
        }
        let kind = match f[4] {
            "_sftp-ssh._tcp" | "_ssh._tcp" => "sftp",
            "_smb._tcp" => "smb",
            _ => continue,
        };
        let host = f[6].trim_end_matches('.').to_owned();
        if found.iter().any(|s| s.host == host && s.kind == kind) {
            continue;
        }
        found.push(Service { name: f[3].replace("\\032", " "), host, port: f[8].to_owned(), kind });
    }
    found
}

pub fn dialog(parent: &gtk::Window, open: Rc<dyn Fn(gio::File)>, error: Rc<dyn Fn(String)>) {
    let window = gtk::Window::builder().transient_for(parent).modal(true).title(t("na rede")).default_width(460).default_height(480).build();
    window.add_css_class("files-dialog");
    let column = gtk::Box::new(gtk::Orientation::Vertical, 10);
    column.add_css_class("files-dialog-root");
    let list = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let note = gtk::Label::new(Some(t("procurando…")));
    note.add_css_class("dim");
    note.set_wrap(true);
    list.append(&note);
    let scroller = gtk::ScrolledWindow::builder().child(&list).vexpand(true).hscrollbar_policy(gtk::PolicyType::Never).build();
    column.append(&scroller);
    window.set_child(Some(&column));
    window.present();

    let user = glib::user_name().to_string_lossy().into_owned();
    let add = {
        let (list, note) = (list.clone(), note.clone());
        move |symbol: &str, title: &str, detail: &str, act: Box<dyn Fn()>| {
            if note.parent().is_some() {
                list.remove(&note);
            }
            let b = gtk::Button::new();
            b.add_css_class("share-device");
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row.append(&icons::icon(symbol));
            let col = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let n = gtk::Label::new(Some(title));
            n.set_xalign(0.0);
            n.add_css_class("git-repo-name");
            let d = gtk::Label::new(Some(detail));
            d.set_xalign(0.0);
            d.add_css_class("dim");
            col.append(&n);
            col.append(&d);
            row.append(&col);
            b.set_child(Some(&row));
            b.connect_clicked(move |_| act());
            list.append(&b);
        }
    };
    let add = Rc::new(add);

    {
        let (add, w, open, error) = (add.clone(), window.clone(), open.clone(), error.clone());
        glib::spawn_future_local(async move {
            let services = gio::spawn_blocking(browse).await.unwrap_or_default();
            for s in services {
                let uri = if s.kind == "sftp" { format!("sftp://{user}@{}:{}/", s.host, s.port) } else { format!("smb://{}/", s.host) };
                let (w, open, error) = (w.clone(), open.clone(), error.clone());
                let detail = format!("{} · {}", s.kind, s.host);
                add(if s.kind == "smb" { "folder_shared" } else { "dns" }, &s.name, &detail, Box::new(move || mount(&w, &uri, open.clone(), error.clone())));
            }
        });
    }
    {
        let add = add.clone();
        super::share::discover(4, move |device| {
            let detail = format!("localsend · {}", device.ip);
            add(super::share::symbol(&device.kind), &device.alias, &detail, Box::new(|| {}));
        });
    }
    glib::timeout_add_local_once(std::time::Duration::from_secs(6), move || {
        if note.parent().is_some() {
            note.set_label(t("nada encontrado. computadores aparecem se anunciarem ssh ou smb na rede (avahi)"));
        }
    });
}

/// Mounts through gvfs (asking for a password if needed), then opens.
pub fn mount(parent: &gtk::Window, uri: &str, open: Rc<dyn Fn(gio::File)>, error: Rc<dyn Fn(String)>) {
    let file = gio::File::for_uri(uri);
    let op = gtk::MountOperation::new(Some(parent));
    let (f2, p) = (file.clone(), parent.clone());
    file.mount_enclosing_volume(gio::MountMountFlags::NONE, Some(&op), gio::Cancellable::NONE, move |r| match r {
        Ok(()) => {
            open(f2);
            p.close();
        }
        Err(e) if e.matches(gio::IOErrorEnum::AlreadyMounted) => {
            open(f2);
            p.close();
        }
        Err(e) if e.matches(gio::IOErrorEnum::NotSupported) && f2.uri_scheme().as_deref() == Some("smb") => error(t("pra abrir smb instale o gvfs-smb").to_owned()),
        Err(e) => error(e.to_string()),
    });
}
