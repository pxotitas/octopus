//! Sending files with LocalSend: the devices nearby that have it open show
//! up, a click sends. The protocol lives in scripts/localsend.py.

use std::io::BufRead;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::i18n::{t, tf};
use crate::icons;

#[derive(Clone, serde::Deserialize)]
pub struct Device {
    pub ip: String,
    pub alias: String,
    pub model: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub port: u16,
    pub protocol: String,
}

fn script() -> std::path::PathBuf {
    crate::exec::config_dir().join("scripts/localsend.py")
}

/// Streams devices as they answer, for a few seconds.
pub fn discover(seconds: u32, found: impl Fn(Device) + 'static) {
    let (tx, rx) = async_channel::unbounded::<Device>();
    let script = script();
    std::thread::spawn(move || {
        let Ok(mut child) = std::process::Command::new("python3")
            .arg(script)
            .args(["discover", &seconds.to_string()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
        else {
            return;
        };
        if let Some(out) = child.stdout.take() {
            for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                if let Ok(device) = serde_json::from_str::<Device>(&line) {
                    if tx.send_blocking(device).is_err() {
                        break;
                    }
                }
            }
        }
        let _ = child.wait();
    });
    glib::spawn_future_local(async move {
        while let Ok(device) = rx.recv().await {
            found(device);
        }
    });
}

pub fn symbol(kind: &str) -> &'static str {
    match kind {
        "mobile" => "smartphone",
        "web" => "language",
        "server" | "headless" => "dns",
        _ => "computer",
    }
}

pub fn dialog(parent: &gtk::Window, files: Vec<gio::File>) {
    let paths: Vec<String> = files.iter().filter_map(|f| f.path()).map(|p| p.to_string_lossy().into_owned()).collect();
    if paths.is_empty() {
        return;
    }
    let window = gtk::Window::builder().transient_for(parent).modal(true).title("localsend").default_width(420).default_height(460).build();
    window.add_css_class("files-dialog");
    let column = gtk::Box::new(gtk::Orientation::Vertical, 12);
    column.add_css_class("files-dialog-root");
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let badge = crate::widgets::shape::Shape::new(crate::widgets::shape::Kind::Sunny, 48, true, Some(&icons::icon("send")));
    badge.add_css_class("git-badge");
    head.append(&badge);
    let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let title = gtk::Label::new(Some(&tf("enviar {} itens", paths.len())));
    title.add_css_class("reminders-title");
    title.set_xalign(0.0);
    let sub = gtk::Label::new(Some(t("aparelhos com o localsend aberto, na mesma rede")));
    sub.add_css_class("dim");
    sub.set_xalign(0.0);
    sub.set_wrap(true);
    texts.append(&title);
    texts.append(&sub);
    head.append(&texts);
    column.append(&head);

    let list = gtk::Box::new(gtk::Orientation::Vertical, 6);
    let searching = gtk::Label::new(Some(t("procurando…")));
    searching.add_css_class("dim");
    list.append(&searching);
    let scroller = gtk::ScrolledWindow::builder().child(&list).vexpand(true).hscrollbar_policy(gtk::PolicyType::Never).build();
    column.append(&scroller);
    let progress = gtk::ProgressBar::new();
    progress.add_css_class("files-progress");
    progress.set_visible(false);
    let state = gtk::Label::new(None);
    state.add_css_class("dim");
    column.append(&state);
    column.append(&progress);
    let again = gtk::Button::with_label(t("procurar de novo"));
    column.append(&again);
    window.set_child(Some(&column));
    window.present();

    let paths = Rc::new(paths);
    let search = {
        let (list, searching, paths, state, progress) = (list.clone(), searching.clone(), paths.clone(), state.clone(), progress.clone());
        Rc::new(move || {
            crate::widgets::clear(&list);
            searching.set_label(t("procurando…"));
            list.append(&searching);
            let (list, paths, state, progress, searching) = (list.clone(), paths.clone(), state.clone(), progress.clone(), searching.clone());
            let s2 = searching.clone();
            glib::timeout_add_local_once(std::time::Duration::from_secs(6), move || {
                if s2.parent().is_some() {
                    s2.set_label(t("ninguém apareceu: abra o localsend no outro aparelho"));
                }
            });
            discover(5, move |device| {
                if searching.parent().is_some() {
                    list.remove(&searching);
                }
                let b = gtk::Button::new();
                b.add_css_class("share-device");
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                row.append(&icons::icon(symbol(&device.kind)));
                let col = gtk::Box::new(gtk::Orientation::Vertical, 0);
                let n = gtk::Label::new(Some(&device.alias));
                n.set_xalign(0.0);
                n.add_css_class("git-repo-name");
                let m = gtk::Label::new(Some(&format!("{} · {}", device.model, device.ip)));
                m.set_xalign(0.0);
                m.add_css_class("dim");
                col.append(&n);
                col.append(&m);
                row.append(&col);
                b.set_child(Some(&row));
                let (paths, state, progress) = (paths.clone(), state.clone(), progress.clone());
                b.connect_clicked(move |b| {
                    b.set_sensitive(false);
                    send(&device, &paths, state.clone(), progress.clone());
                });
                list.append(&b);
            });
        })
    };
    search();
    again.connect_clicked(move |_| search());
}

fn send(device: &Device, paths: &[String], state: gtk::Label, progress: gtk::ProgressBar) {
    let (tx, rx) = async_channel::unbounded::<String>();
    let mut command = std::process::Command::new("python3");
    command.arg(script()).args(["send", &device.ip, &device.port.to_string(), &device.protocol]).args(paths);
    std::thread::spawn(move || {
        let Ok(mut child) = command.stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::null()).spawn() else {
            let _ = tx.send_blocking("error python3".into());
            return;
        };
        if let Some(out) = child.stdout.take() {
            for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                let _ = tx.send_blocking(line);
            }
        }
        let _ = child.wait();
    });
    let alias = device.alias.clone();
    glib::spawn_future_local(async move {
        while let Ok(line) = rx.recv().await {
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("waiting") => state.set_label(&tf("esperando {} aceitar…", &alias)),
                Some("progress") => {
                    let done: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                    let total: f64 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
                    progress.set_visible(true);
                    progress.set_fraction(done / total.max(1.0));
                    state.set_label(&tf("enviando pra {}", &alias));
                }
                Some("done") => {
                    progress.set_fraction(1.0);
                    state.set_label(&tf("enviado pra {}", &alias));
                }
                Some("error") => state.set_label(&format!("{}: {}", t("não foi"), parts.collect::<Vec<_>>().join(" "))),
                _ => {}
            }
        }
    });
}
