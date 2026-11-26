//! The bar as the draft would make it, drawn with the bar's own shapes and
//! colors. In the editor its modules can be picked and dragged between the
//! top, middle and bottom, and new ones dropped in from the catalog.

use std::cell::RefCell;
use std::rc::Rc;

use chrono::Timelike;
use gtk::prelude::*;
use gtk::{gdk, glib};

use super::Ctx;
use crate::config::Module;
use crate::icons;

/// What the editor hears from the preview.
pub struct Hooks {
    pub select: Box<dyn Fn(&str)>,
    pub selected: RefCell<String>,
}

pub struct Preview {
    pub root: gtk::Box,
    column: gtk::Box,
    ctx: Rc<Ctx>,
    hooks: Option<Rc<Hooks>>,
}

fn icon(name: &str, class: &str) -> gtk::Label {
    let glyph = icons::icon(name);
    glyph.add_css_class(class);
    glyph
}

fn value(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("pm-value");
    label
}

/// One module as the bar draws it.
fn module(id: &str, m: &Module) -> gtk::Box {
    let item = gtk::Box::new(gtk::Orientation::Vertical, 2);
    item.add_css_class("preview-module");
    item.add_css_class(&format!("pm-{id}"));
    item.set_halign(gtk::Align::Center);
    if m.background != "none" {
        item.add_css_class(&format!("bg-{}", m.background));
    }
    item.add_css_class(&format!("shape-{}", m.shape));
    match id {
        "logo" => {
            if m.variant == "arch" || m.variant.is_empty() {
                let glyph = gtk::Label::new(Some("\u{f303}"));
                glyph.add_css_class("pm-arch");
                item.append(&glyph);
            } else {
                item.append(&icon(&m.variant, "pm-logo"));
            }
        }
        "workspaces" => {
            for (i, active) in [false, true, false].into_iter().enumerate() {
                let dot = if m.variant == "numbers" {
                    let n = gtk::Label::new(Some(&(i + 1).to_string()));
                    n.add_css_class("pm-number");
                    n.upcast::<gtk::Widget>()
                } else {
                    gtk::Box::new(gtk::Orientation::Vertical, 0).upcast()
                };
                dot.add_css_class("pm-dot");
                if active {
                    dot.add_css_class("active");
                }
                dot.set_halign(gtk::Align::Center);
                item.append(&dot);
            }
        }
        "clock" => {
            let now = chrono::Local::now();
            let hour = if m.variant == "12h" { (now.hour() + 11) % 12 + 1 } else { now.hour() };
            item.set_spacing(0);
            if m.shape == "bun" {
                item.add_css_class("bun");
            }
            for text in [format!("{hour:02}"), format!("{:02}", now.minute())] {
                let half = gtk::Label::new(Some(&text));
                half.add_css_class("pm-clock");
                item.append(&half);
            }
        }
        "tray" => item.append(&icon("keyboard_arrow_down", "pm-icon")),
        "battery" => {
            let pill = gtk::Box::new(gtk::Orientation::Vertical, 0);
            pill.add_css_class("pm-battery");
            pill.set_halign(gtk::Align::Center);
            let fill = gtk::Box::new(gtk::Orientation::Vertical, 0);
            fill.add_css_class("pm-battery-fill");
            fill.set_vexpand(true);
            pill.append(&fill);
            item.append(&pill);
            if m.text {
                item.append(&value("99"));
            }
        }
        "controls" => {
            let shown: &[String] = if m.variant == "open" { &m.items } else { &m.items[..m.items.len().min(1)] };
            for name in shown {
                let (glyph, class) = match name.as_str() {
                    "control-center" => ("tune", "pm-chip"),
                    "network" => ("signal_wifi_4_bar", "pm-icon"),
                    "bluetooth" => ("bluetooth", "pm-icon"),
                    "volume" => ("volume_up", "pm-icon"),
                    _ => ("power_settings_new", "pm-danger"),
                };
                item.append(&icon(glyph, class));
            }
            if shown.is_empty() {
                item.append(&icon("tune", "pm-icon"));
            }
        }
        _ => {
            let (glyph, sample) = match id {
                "cpu" => ("memory", "12"),
                "memory" => ("memory_alt", "41"),
                "temperature" => ("thermostat", "48"),
                "brightness" => ("brightness_medium", "70"),
                "volume" => ("volume_up", "55"),
                "idle" => ("bedtime", ""),
                "media" => ("play_arrow", ""),
                "notifications" => ("notifications", ""),
                "power" => ("power_settings_new", ""),
                _ => ("help", ""),
            };
            item.append(&icon(glyph, if id == "power" { "pm-danger" } else { "pm-icon" }));
            if m.text && !sample.is_empty() {
                item.append(&value(sample));
            }
        }
    }
    item
}

/// A bar that only shows itself, for the presets.
pub fn still(bar: &crate::config::Bar, height: i32) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
    column.add_css_class("bar-preview");
    column.add_css_class("floating");
    column.set_size_request(bar.width as i32, height);
    column.set_halign(gtk::Align::Center);
    for (index, list) in [&bar.top, &bar.center, &bar.bottom].into_iter().enumerate() {
        let zone = gtk::Box::new(gtk::Orientation::Vertical, bar.spacing as i32);
        match index {
            0 => zone.set_valign(gtk::Align::Start),
            1 => {
                zone.set_valign(gtk::Align::Center);
                zone.set_vexpand(true);
            }
            _ => zone.set_valign(gtk::Align::End),
        }
        for id in list {
            zone.append(&module(id, &bar.module(id)));
        }
        column.append(&zone);
    }
    column
}

impl Preview {
    pub fn new(ctx: &Rc<Ctx>, hooks: Option<Rc<Hooks>>) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.add_css_class("bar-stage");
        root.set_valign(gtk::Align::Start);
        let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
        column.add_css_class("bar-preview");
        column.set_size_request(-1, 520);
        root.append(&column);
        let preview = Rc::new(Self { root, column, ctx: ctx.clone(), hooks });
        preview.rebuild();
        let weak = Rc::downgrade(&preview);
        ctx.listen(move || {
            let weak = weak.clone();
            glib::idle_add_local_once(move || {
                if let Some(p) = weak.upgrade() {
                    p.rebuild();
                }
            });
        });
        preview
    }

    pub fn rebuild(self: &Rc<Self>) {
        crate::widgets::clear(&self.column);
        let bar = self.ctx.get().bar.clone();
        self.column.set_size_request(bar.width as i32, 520);
        self.root.set_halign(if bar.right() { gtk::Align::End } else { gtk::Align::Start });
        for class in ["flush", "floating"] {
            self.column.remove_css_class(class);
        }
        self.column.add_css_class(if bar.style == "flush" { "flush" } else { "floating" });

        for (index, list) in [&bar.top, &bar.center, &bar.bottom].into_iter().enumerate() {
            let zone = gtk::Box::new(gtk::Orientation::Vertical, bar.spacing as i32);
            zone.add_css_class("preview-zone");
            match index {
                0 => zone.set_valign(gtk::Align::Start),
                1 => {
                    zone.set_valign(gtk::Align::Center);
                    zone.set_vexpand(true);
                }
                _ => zone.set_valign(gtk::Align::End),
            }
            for id in list {
                let item = module(id, &bar.module(id));
                if let Some(hooks) = &self.hooks {
                    self.interactive(&item, id, hooks);
                }
                zone.append(&item);
            }
            if self.hooks.is_some() {
                self.accept(&zone, index);
            }
            self.column.append(&zone);
        }
    }

    fn interactive(self: &Rc<Self>, item: &gtk::Box, id: &str, hooks: &Rc<Hooks>) {
        if *hooks.selected.borrow() == id {
            item.add_css_class("selected");
        }
        let id = id.to_owned();
        let click = gtk::GestureClick::new();
        let (h, name) = (hooks.clone(), id.clone());
        click.connect_released(move |_, _, _, _| (h.select)(&name));
        item.add_controller(click);

        let source = gtk::DragSource::new();
        source.set_actions(gdk::DragAction::MOVE);
        let payload = format!("move:{id}");
        source.connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&payload.to_value())));
        let picture = item.clone();
        source.connect_drag_begin(move |source, _| {
            source.set_icon(Some(&gtk::WidgetPaintable::new(Some(&picture))), 16, 16);
            picture.add_css_class("dragging");
        });
        let picture = item.clone();
        source.connect_drag_end(move |_, _, _| picture.remove_css_class("dragging"));
        item.add_controller(source);
    }

    /// A zone takes modules dragged in, at the gap nearest the pointer.
    fn accept(self: &Rc<Self>, zone: &gtk::Box, index: usize) {
        let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE | gdk::DragAction::COPY);
        let z = zone.clone();
        target.connect_enter(move |_, _, _| {
            z.add_css_class("drop-hover");
            gdk::DragAction::MOVE
        });
        let z = zone.clone();
        target.connect_leave(move |_| z.remove_css_class("drop-hover"));
        let (weak, z) = (Rc::downgrade(self), zone.clone());
        target.connect_drop(move |_, value, _, y| {
            z.remove_css_class("drop-hover");
            let (Some(p), Ok(payload)) = (weak.upgrade(), value.get::<String>()) else { return false };
            let Some((_, id)) = payload.split_once(':') else { return false };
            let mut at = 0;
            let mut child = z.first_child();
            while let Some(c) = child {
                let middle = c.compute_point(&z, &gtk::graphene::Point::new(0.0, c.height() as f32 / 2.0)).map(|p| p.y() as f64);
                if middle.is_some_and(|m| y > m) {
                    at += 1;
                }
                child = c.next_sibling();
            }
            let id = id.to_owned();
            p.ctx.edit(|cfg| place(&mut cfg.bar, &id, Some((index, at))));
            if let Some(hooks) = &p.hooks {
                (hooks.select)(&id);
            }
            true
        });
        zone.add_controller(target);
    }
}

/// Moves a module to (zone, position), or out of the bar with None.
pub fn place(bar: &mut crate::config::Bar, id: &str, to: Option<(usize, usize)>) {
    let mut from: Option<(usize, usize)> = None;
    for (z, list) in [&mut bar.top, &mut bar.center, &mut bar.bottom].into_iter().enumerate() {
        if let Some(pos) = list.iter().position(|m| m == id) {
            list.remove(pos);
            from = Some((z, pos));
        }
    }
    let Some((zone, mut at)) = to else { return };
    if let Some((z, pos)) = from {
        if z == zone && pos < at {
            at -= 1;
        }
    }
    let list = match zone {
        0 => &mut bar.top,
        1 => &mut bar.center,
        _ => &mut bar.bottom,
    };
    let at = at.min(list.len());
    list.insert(at, id.to_owned());
}

/// Which zone a module sits in and where, if it is in the bar.
pub fn where_is(bar: &crate::config::Bar, id: &str) -> Option<(usize, usize)> {
    [&bar.top, &bar.center, &bar.bottom]
        .into_iter()
        .enumerate()
        .find_map(|(z, list)| list.iter().position(|m| m == id).map(|p| (z, p)))
}
