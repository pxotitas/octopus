//! How one file looks: a tile in the icon view, a row's cells in the list.
//! Thumbnails come in when ready; folders take drops.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use super::{file_of, tags, thumbs};
use crate::i18n::t;

/// What a drop onto a folder does: (files, the folder, copy instead of move).
pub type OnDrop = Rc<dyn Fn(Vec<gio::File>, gio::File, bool)>;

fn info_of(item: &gtk::ListItem) -> Option<gio::FileInfo> {
    item.item().and_downcast::<gio::FileInfo>()
}

fn is_dir(info: &gio::FileInfo) -> bool {
    info.file_type() == gio::FileType::Directory
}

/// The icon, or the thumbnail once it's there; only if the row still shows
/// the same file by then.
fn set_icon(image: &gtk::Image, item: &gtk::ListItem, info: &gio::FileInfo, size: i32) {
    image.set_pixel_size(size);
    match info.icon() {
        Some(icon) => image.set_from_gicon(&icon),
        None => image.set_icon_name(Some("text-x-generic")),
    }
    image.remove_css_class("thumb");
    let Some(file) = file_of(info) else { return };
    let ct = info.content_type().map(|c| c.to_string()).unwrap_or_default();
    if !thumbs::wanted(&ct) {
        return;
    }
    let (weak_item, weak_image, wanted) = (item.downgrade(), image.downgrade(), info.clone());
    thumbs::request(&file, info, move |texture| {
        let (Some(item), Some(image), Some(texture)) = (weak_item.upgrade(), weak_image.upgrade(), texture) else { return };
        if info_of(&item).is_some_and(|i| i == wanted) {
            image.set_paintable(Some(&texture));
            image.add_css_class("thumb");
        }
    });
}

fn dots(info: &gio::FileInfo) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    row.add_css_class("tag-dots");
    for tag in tags::of(info) {
        let dot = gtk::DrawingArea::new();
        dot.set_content_width(8);
        dot.set_content_height(8);
        dot.set_valign(gtk::Align::Center);
        let color = gdk::RGBA::parse(tags::color(&tag)).unwrap_or(gdk::RGBA::BLACK);
        dot.set_draw_func(move |_, cr, w, h| {
            cr.set_source_rgba(color.red() as f64, color.green() as f64, color.blue() as f64, 1.0);
            cr.arc(w as f64 / 2.0, h as f64 / 2.0, w.min(h) as f64 / 2.0, 0.0, std::f64::consts::TAU);
            let _ = cr.fill();
        });
        row.append(&dot);
    }
    row.set_visible(row.first_child().is_some());
    row
}

/// A folder under the pointer takes dropped files.
pub fn drop_into(widget: &impl IsA<gtk::Widget>, item: &gtk::ListItem, on_drop: &OnDrop) {
    let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY | gdk::DragAction::MOVE);
    let (weak, on_drop) = (item.downgrade(), on_drop.clone());
    target.connect_accept(move |_, _| weak.upgrade().and_then(|i| info_of(&i)).is_some_and(|i| is_dir(&i)));
    let weak = item.downgrade();
    target.connect_drop(move |t, value, _, _| {
        let Some(info) = weak.upgrade().and_then(|i| info_of(&i)) else { return false };
        let (Some(folder), Ok(list)) = (file_of(&info), value.get::<gdk::FileList>()) else { return false };
        let copy = t.current_drop().is_some_and(|d| d.actions() == gdk::DragAction::COPY);
        on_drop(list.files(), folder, copy);
        true
    });
    let w = widget.as_ref().clone();
    target.connect_enter(move |_, _, _| {
        w.add_css_class("drop-hover");
        gdk::DragAction::MOVE
    });
    let w = widget.as_ref().clone();
    target.connect_leave(move |_| w.remove_css_class("drop-hover"));
    widget.as_ref().add_controller(target);
}

/// The icon view's tile.
pub fn grid_factory(size: Rc<std::cell::Cell<i32>>, on_drop: OnDrop) -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_setup(move |_, obj| {
        let Some(item) = obj.downcast_ref::<gtk::ListItem>() else { return };
        let tile = gtk::Box::new(gtk::Orientation::Vertical, 6);
        tile.add_css_class("file-tile");
        let image = gtk::Image::new();
        image.add_css_class("file-icon");
        image.set_halign(gtk::Align::Center);
        let name = gtk::Label::new(None);
        name.add_css_class("file-name");
        name.set_wrap(true);
        name.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        name.set_lines(2);
        name.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        name.set_justify(gtk::Justification::Center);
        name.set_max_width_chars(16);
        let attrs = gtk::pango::AttrList::new();
        attrs.insert(gtk::pango::AttrInt::new_insert_hyphens(false));
        name.set_attributes(Some(&attrs));
        tile.set_size_request(104, -1);
        let line = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        line.set_halign(gtk::Align::Center);
        line.append(&name);
        tile.append(&image);
        tile.append(&line);
        drop_into(&tile, item, &on_drop);
        item.set_child(Some(&tile));
    });
    factory.connect_bind(move |_, obj| {
        let Some(item) = obj.downcast_ref::<gtk::ListItem>() else { return };
        let Some(info) = info_of(item) else { return };
        let Some(tile) = item.child().and_downcast::<gtk::Box>() else { return };
        let Some(image) = tile.first_child().and_downcast::<gtk::Image>() else { return };
        let Some(line) = tile.last_child().and_downcast::<gtk::Box>() else { return };
        set_icon(&image, item, &info, size.get());
        if let Some(name) = line.first_child().and_downcast::<gtk::Label>() {
            name.set_label(&info.display_name());
        }
        while let Some(extra) = line.first_child().and_then(|c| c.next_sibling()) {
            line.remove(&extra);
        }
        line.append(&dots(&info));
        tile.set_opacity(if info.is_hidden() { 0.6 } else { 1.0 });
        tile.set_tooltip_text(Some(&info.display_name()));
    });
    factory
}

/// The list view's columns: name (with its icon), size, modified, kind.
pub fn list_columns(view: &gtk::ColumnView, on_drop: OnDrop) {
    let name = gtk::SignalListItemFactory::new();
    name.connect_setup(move |_, obj| {
        let Some(item) = obj.downcast_ref::<gtk::ListItem>() else { return };
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        row.add_css_class("file-row");
        let image = gtk::Image::new();
        let label = gtk::Label::new(None);
        label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        label.set_xalign(0.0);
        row.append(&image);
        row.append(&label);
        drop_into(&row, item, &on_drop);
        item.set_child(Some(&row));
    });
    name.connect_bind(|_, obj| {
        let Some(item) = obj.downcast_ref::<gtk::ListItem>() else { return };
        let Some(info) = info_of(item) else { return };
        let Some(row) = item.child().and_downcast::<gtk::Box>() else { return };
        let Some(image) = row.first_child().and_downcast::<gtk::Image>() else { return };
        set_icon(&image, item, &info, 22);
        if let Some(label) = image.next_sibling().and_downcast::<gtk::Label>() {
            label.set_label(&info.display_name());
        }
        while let Some(extra) = row.last_child().filter(|c| !c.is::<gtk::Label>() && !c.is::<gtk::Image>()) {
            row.remove(&extra);
        }
        row.append(&dots(&info));
        row.set_opacity(if info.is_hidden() { 0.6 } else { 1.0 });
    });

    let text_column = |getter: fn(&gio::FileInfo) -> String, end: bool| {
        let f = gtk::SignalListItemFactory::new();
        f.connect_setup(move |_, obj| {
            if let Some(item) = obj.downcast_ref::<gtk::ListItem>() {
                let label = gtk::Label::new(None);
                label.add_css_class("file-meta");
                label.set_xalign(if end { 1.0 } else { 0.0 });
                label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                item.set_child(Some(&label));
            }
        });
        f.connect_bind(move |_, obj| {
            let Some(item) = obj.downcast_ref::<gtk::ListItem>() else { return };
            if let (Some(info), Some(label)) = (info_of(item), item.child().and_downcast::<gtk::Label>()) {
                label.set_label(&getter(&info));
            }
        });
        f
    };

    let columns: [(&str, gtk::SignalListItemFactory, bool, i32); 4] = [
        ("nome", name, true, -1),
        ("modificado", text_column(modified, false), false, 170),
        ("tamanho", text_column(size, true), false, 90),
        ("tipo", text_column(kind, false), false, 150),
    ];
    for (title, factory, expand, width) in columns {
        let column = gtk::ColumnViewColumn::new(Some(t(title)), Some(factory));
        column.set_expand(expand);
        column.set_resizable(true);
        if width > 0 {
            column.set_fixed_width(width);
        }
        view.append_column(&column);
    }
}

pub fn size(info: &gio::FileInfo) -> String {
    if is_dir(info) {
        "—".to_owned()
    } else {
        glib::format_size(info.size().max(0) as u64).to_string()
    }
}

pub fn modified(info: &gio::FileInfo) -> String {
    info.modification_date_time()
        .and_then(|d| d.to_local().ok())
        .and_then(|d| {
            let today = glib::DateTime::now_local().ok()?;
            let same_day = d.year() == today.year() && d.day_of_year() == today.day_of_year();
            d.format(if same_day { "%H:%M" } else { "%d %b %Y, %H:%M" }).ok()
        })
        .map(|s| if s.len() <= 5 { format!("{} {s}", t("hoje,")) } else { s.to_string() })
        .unwrap_or_default()
}

pub fn kind(info: &gio::FileInfo) -> String {
    if is_dir(info) {
        return t("pasta").to_owned();
    }
    info.content_type().map(|c| gio::content_type_get_description(&c).to_string()).unwrap_or_default()
}
