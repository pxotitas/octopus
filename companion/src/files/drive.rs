//! Google Drive through rclone: rclone signs in with its own app, in the
//! browser, so there's nothing to create in a cloud console. The drive is
//! mounted in ~/Google Drive and opened like any folder.

use std::path::PathBuf;
use std::rc::Rc;

use gtk::{gio, glib};

use crate::i18n::t;

const REMOTE: &str = "gdrive";

fn mount_point() -> PathBuf {
    glib::home_dir().join("Google Drive")
}

fn mounted() -> bool {
    let point = mount_point();
    std::fs::read_to_string("/proc/mounts").unwrap_or_default().lines().any(|l| l.split(' ').nth(1).is_some_and(|m| m.replace("\\040", " ") == point.to_string_lossy()))
}

fn configured() -> bool {
    std::process::Command::new("rclone").arg("listremotes").output().is_ok_and(|o| String::from_utf8_lossy(&o.stdout).lines().any(|l| l == format!("{REMOTE}:")))
}

pub fn open(parent: &gtk::Window, open: Rc<dyn Fn(gio::File)>, error: Rc<dyn Fn(String)>) {
    if glib::find_program_in_path("rclone").is_none() {
        explain(parent, t("o google drive usa o rclone"), t("instale com: sudo pacman -S rclone. depois clique em google drive de novo."), None);
        return;
    }
    let (parent, open, error) = (parent.clone(), open.clone(), error.clone());
    glib::spawn_future_local(async move {
        if gio::spawn_blocking(mounted).await.unwrap_or(false) {
            open(gio::File::for_path(mount_point()));
            return;
        }
        if !gio::spawn_blocking(configured).await.unwrap_or(false) {
            let connect: Box<dyn Fn()> = Box::new(|| {
                // rclone opens the browser for the sign-in, then saves it
                super::run_in_terminal(&format!("rclone config create {REMOTE} drive && echo && echo 'conectado: clique em google drive de novo' ; read -r _"));
            });
            explain(&parent, t("conectar o google drive"), t("o login abre no navegador, com o rclone. nada pra criar no google cloud."), Some(connect));
            return;
        }
        let point = mount_point();
        let _ = std::fs::create_dir_all(&point);
        let p = point.clone();
        let started = gio::spawn_blocking(move || {
            std::process::Command::new("rclone")
                .args(["mount", &format!("{REMOTE}:"), &p.to_string_lossy(), "--vfs-cache-mode", "full", "--daemon"])
                .status()
                .is_ok_and(|s| s.success())
        })
        .await
        .unwrap_or(false);
        if started {
            glib::timeout_add_local_once(std::time::Duration::from_millis(1200), move || open(gio::File::for_path(point)));
        } else {
            error(t("o rclone não montou o drive (precisa do fuse3)").to_owned());
        }
    });
}

fn explain(parent: &gtk::Window, title: &str, body: &str, action: Option<Box<dyn Fn()>>) {
    let dialog = gtk::AlertDialog::builder().message(title).detail(body).build();
    match action {
        Some(act) => {
            dialog.set_buttons(&[t("agora não"), t("conectar")]);
            dialog.set_cancel_button(0);
            dialog.set_default_button(1);
            dialog.choose(Some(parent), gio::Cancellable::NONE, move |r| {
                if r == Ok(1) {
                    act();
                }
            });
        }
        None => dialog.show(Some(parent)),
    }
}
