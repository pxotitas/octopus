//! Quick look: space shows the file big, without opening anything; the
//! arrows go to the neighbours, space or esc close it.

use std::cell::Cell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::i18n::t;

thread_local! {
    static OPEN: std::cell::RefCell<Option<gtk::glib::WeakRef<gtk::Window>>> = const { std::cell::RefCell::new(None) };
}

/// Closes the preview if one is open; true if it was.
pub fn close() -> bool {
    let open = OPEN.with(|o| o.borrow_mut().take()).and_then(|w| w.upgrade());
    if let Some(w) = &open {
        w.close();
    }
    open.is_some()
}

pub fn show(parent: &gtk::Window, items: Vec<(gio::File, gio::FileInfo)>, start: usize) {
    if items.is_empty() {
        return;
    }
    let window = gtk::Window::builder()
        .transient_for(parent)
        .modal(true)
        .default_width(860)
        .default_height(600)
        .decorated(false)
        .build();
    window.add_css_class("quicklook");

    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.set_focusable(true);
    root.add_css_class("quicklook-root");
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let title = gtk::Label::new(None);
    title.add_css_class("quicklook-title");
    title.set_hexpand(true);
    title.set_xalign(0.0);
    title.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
    let detail = gtk::Label::new(None);
    detail.add_css_class("quicklook-detail");
    let open = crate::icons::button("open_in_new", t("abrir"));
    let close = crate::icons::button("close", t("fechar"));
    head.append(&title);
    head.append(&detail);
    head.append(&open);
    head.append(&close);
    root.append(&head);
    let body = gtk::Box::new(gtk::Orientation::Vertical, 0);
    body.set_vexpand(true);
    body.add_css_class("quicklook-body");
    root.append(&body);
    window.set_child(Some(&root));

    let items = Rc::new(items);
    let index = Rc::new(Cell::new(start.min(items.len() - 1)));
    let render = {
        let (items, index, body, title, detail) = (items.clone(), index.clone(), body.clone(), title.clone(), detail.clone());
        Rc::new(move || {
            crate::widgets::clear(&body);
            let (file, info) = &items[index.get()];
            title.set_label(&info.display_name());
            detail.set_label(&format!("{} · {}", super::cells::kind(info), super::cells::size(info)));
            body.append(&preview(file, info));
        })
    };
    render();

    let w = window.clone();
    close.connect_clicked(move |_| w.close());
    let (items_o, index_o, w) = (items.clone(), index.clone(), window.clone());
    open.connect_clicked(move |_| {
        let uri = items_o[index_o.get()].0.uri();
        let _ = gio::AppInfo::launch_default_for_uri(&uri, gdk::Display::default().map(|d| d.app_launch_context()).as_ref());
        w.close();
    });

    let keys = gtk::EventControllerKey::new();
    let w = window.clone();
    keys.connect_key_pressed(move |_, key, _, _| {
        let step: i64 = match key {
            gdk::Key::space | gdk::Key::Escape => {
                w.close();
                return glib::Propagation::Stop;
            }
            gdk::Key::Right | gdk::Key::Down => 1,
            gdk::Key::Left | gdk::Key::Up => -1,
            _ => return glib::Propagation::Proceed,
        };
        let next = (index.get() as i64 + step).rem_euclid(items.len() as i64) as usize;
        index.set(next);
        render();
        glib::Propagation::Stop
    });
    window.add_controller(keys);
    OPEN.with(|o| *o.borrow_mut() = Some(window.downgrade()));
    window.present();
    root.grab_focus();
}

fn preview(file: &gio::File, info: &gio::FileInfo) -> gtk::Widget {
    let ct = info.content_type().map(|c| c.to_string()).unwrap_or_default();
    if info.file_type() == gio::FileType::Directory {
        return big_icon(info);
    }
    if ct.starts_with("image/") {
        let picture = gtk::Picture::new();
        picture.set_content_fit(gtk::ContentFit::Contain);
        picture.set_vexpand(true);
        if let Some(path) = file.path() {
            let p = picture.clone();
            glib::spawn_future_local(async move {
                if let Ok(Ok(texture)) = gio::spawn_blocking(move || gdk::Texture::from_filename(&path)).await {
                    p.set_paintable(Some(&texture));
                }
            });
        }
        return picture.upcast();
    }
    if ct.starts_with("video/") || ct.starts_with("audio/") {
        let video = gtk::Video::for_file(Some(file));
        video.set_autoplay(true);
        video.set_vexpand(true);
        return video.upcast();
    }
    let textual = ct.starts_with("text/") || gio::content_type_is_a(&ct, "text/plain") || ct.contains("json") || ct.contains("xml") || ct.contains("x-shellscript");
    if textual {
        if let Some(path) = file.path() {
            let mut text = String::new();
            if let Ok(f) = std::fs::File::open(&path) {
                use std::io::Read;
                let mut buf = Vec::new();
                let _ = f.take(200_000).read_to_end(&mut buf);
                text = String::from_utf8_lossy(&buf).into_owned();
            }
            let view = gtk::TextView::new();
            view.set_editable(false);
            view.set_monospace(true);
            view.set_wrap_mode(gtk::WrapMode::WordChar);
            view.buffer().set_text(&text);
            view.add_css_class("quicklook-text");
            let scroller = gtk::ScrolledWindow::builder().child(&view).vexpand(true).build();
            return scroller.upcast();
        }
    }
    big_icon(info)
}

fn big_icon(info: &gio::FileInfo) -> gtk::Widget {
    let image = gtk::Image::new();
    if let Some(icon) = info.icon() {
        image.set_from_gicon(&icon);
    }
    image.set_pixel_size(196);
    image.set_vexpand(true);
    image.set_valign(gtk::Align::Center);
    image.upcast()
}
