//! Get info: a file's name, kind, size (folders counted in the background),
//! where it is, its dates, who can touch it, what opens it, and its tags.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use super::{cells, tags};
use crate::i18n::t;

fn line(grid: &gtk::Grid, row: i32, label: &str, value: &impl IsA<gtk::Widget>) {
    let key = gtk::Label::new(Some(label));
    key.add_css_class("info-key");
    key.set_xalign(1.0);
    key.set_valign(gtk::Align::Start);
    grid.attach(&key, 0, row, 1, 1);
    grid.attach(value, 1, row, 1, 1);
}

fn text(value: &str) -> gtk::Label {
    let l = gtk::Label::new(Some(value));
    l.set_xalign(0.0);
    l.set_wrap(true);
    l.set_wrap_mode(gtk::pango::WrapMode::WordChar);
    l.set_selectable(true);
    l
}

pub fn show(parent: &gtk::Window, file: &gio::File) {
    let Ok(info) = file.query_info(&format!("{},unix::mode,unix::uid,owner::user,time::created,time::access,standard::target-uri", super::ATTRIBUTES), gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE) else { return };
    let window = gtk::Window::builder().transient_for(parent).title(info.display_name()).default_width(420).resizable(false).build();
    window.add_css_class("files-info");
    let root = gtk::Box::new(gtk::Orientation::Vertical, 16);
    root.add_css_class("files-info-root");

    let head = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    let image = gtk::Image::new();
    if let Some(icon) = info.icon() {
        image.set_from_gicon(&icon);
    }
    image.set_pixel_size(64);
    head.append(&image);
    let names = gtk::Box::new(gtk::Orientation::Vertical, 2);
    names.set_valign(gtk::Align::Center);
    let name = gtk::Entry::new();
    name.set_text(&info.display_name());
    name.add_css_class("info-name");
    let kind = gtk::Label::new(Some(&cells::kind(&info)));
    kind.add_css_class("dim");
    kind.set_xalign(0.0);
    names.append(&name);
    names.append(&kind);
    names.set_hexpand(true);
    head.append(&names);
    root.append(&head);
    {
        let file = file.clone();
        let w = window.clone();
        name.connect_activate(move |e| {
            let new = e.text().to_string();
            let (file, w) = (file.clone(), w.clone());
            glib::spawn_future_local(async move {
                if let Some(renamed) = super::ops::get().rename(file, new).await {
                    w.set_title(Some(&super::display_name(&renamed)));
                }
            });
        });
    }

    let grid = gtk::Grid::new();
    grid.set_row_spacing(8);
    grid.set_column_spacing(14);
    let size = text(&cells::size(&info));
    line(&grid, 0, t("tamanho"), &size);
    if info.file_type() == gio::FileType::Directory {
        size.set_label(t("contando…"));
        if let Some(path) = file.path() {
            let s = size.clone();
            glib::spawn_future_local(async move {
                let (bytes, count) = gio::spawn_blocking(move || {
                    fn walk(p: &std::path::Path, acc: &mut (u64, u64)) {
                        if let Ok(rd) = std::fs::read_dir(p) {
                            for e in rd.flatten() {
                                if let Ok(m) = e.metadata() {
                                    acc.1 += 1;
                                    if m.is_dir() && !m.is_symlink() {
                                        walk(&e.path(), acc);
                                    } else {
                                        acc.0 += m.len();
                                    }
                                }
                            }
                        }
                    }
                    let mut acc = (0, 0);
                    walk(&path, &mut acc);
                    acc
                })
                .await
                .unwrap_or((0, 0));
                s.set_label(&format!("{} · {}", glib::format_size(bytes), crate::i18n::tf("{} itens", count)));
            });
        }
    }
    let place = file.parent().map(|p| p.parse_name().to_string()).unwrap_or_default();
    line(&grid, 1, t("onde"), &text(&place));
    let date = |attr: &str| {
        info.attribute_uint64(attr)
            .checked_sub(0)
            .filter(|s| *s > 0)
            .and_then(|s| glib::DateTime::from_unix_local(s as i64).ok())
            .and_then(|d| d.format("%d %b %Y, %H:%M").ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".to_owned())
    };
    line(&grid, 2, t("criado"), &text(&date("time::created")));
    line(&grid, 3, t("modificado"), &text(&date("time::modified")));
    line(&grid, 4, t("aberto"), &text(&date("time::access")));
    let mode = info.attribute_uint32("unix::mode");
    let rwx = |shift: u32| {
        let bits = (mode >> shift) & 7;
        format!("{}{}{}", if bits & 4 != 0 { "r" } else { "-" }, if bits & 2 != 0 { "w" } else { "-" }, if bits & 1 != 0 { "x" } else { "-" })
    };
    let owner = info.attribute_string("owner::user").map(|s| s.to_string()).unwrap_or_default();
    line(&grid, 5, t("permissões"), &text(&format!("{owner} · {} {} {}", rwx(6), rwx(3), rwx(0))));

    // what opens it
    if let Some(ct) = info.content_type().filter(|_| info.file_type() != gio::FileType::Directory) {
        let apps = gio::AppInfo::recommended_for_type(&ct);
        if !apps.is_empty() {
            let names: Vec<String> = apps.iter().map(|a| a.display_name().to_string()).collect();
            let refs: Vec<&str> = names.iter().map(String::as_str).collect();
            let chooser = gtk::DropDown::from_strings(&refs);
            let default = gio::AppInfo::default_for_type(&ct, false);
            if let Some(d) = default {
                if let Some(i) = apps.iter().position(|a| a.id() == d.id()) {
                    chooser.set_selected(i as u32);
                }
            }
            let apps = Rc::new(apps);
            chooser.connect_selected_notify(move |c| {
                if let Some(app) = apps.get(c.selected() as usize) {
                    let _ = app.set_as_default_for_type(&ct);
                }
            });
            line(&grid, 6, t("abrir com"), &chooser);
        }
    }

    let dots = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let current = tags::of(&info);
    for (id, name, color) in tags::COLORS {
        let toggle = gtk::ToggleButton::new();
        toggle.add_css_class("info-tag");
        toggle.add_css_class(id);
        toggle.set_tooltip_text(Some(t(name)));
        toggle.set_active(current.iter().any(|c| c == id));
        let css = gtk::CssProvider::new();
        css.load_from_string(&format!("togglebutton.info-tag.{id} {{ background: {color}; }}"));
        gtk::style_context_add_provider_for_display(&toggle.display(), &css, gtk::STYLE_PROVIDER_PRIORITY_USER + 2);
        let f = file.clone();
        toggle.connect_toggled(move |_| tags::toggle(&f, id));
        dots.append(&toggle);
    }
    line(&grid, 7, t("tags"), &dots);
    root.append(&grid);
    window.set_child(Some(&root));
    window.present();
}
