//! The editor's pickers: chips that show what they pick, for the logo's
//! glyph, the shape behind a module, a whole look, and the workspaces.

use std::rc::Rc;

use gtk::prelude::*;

use super::bar_preview::{glyph_widget, module};
use crate::bar::modules::{CORNERS, GLYPHS, LOOKS, WORKSPACES};
use crate::config::{Bar, Module};
use crate::i18n::t;
use crate::widgets::shape::{Kind, Shape};

fn grid(per_line: u32) -> gtk::FlowBox {
    let flow = gtk::FlowBox::new();
    flow.add_css_class("picker");
    flow.set_homogeneous(true);
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_max_children_per_line(per_line);
    flow.set_min_children_per_line(3);
    flow.set_row_spacing(6);
    flow.set_column_spacing(6);
    flow
}

/// A chip that shows `face` and calls `on` when picked.
fn chip(face: &impl IsA<gtk::Widget>, tip: &str, on: bool, group: &mut Option<gtk::ToggleButton>, pick: impl Fn() + 'static) -> gtk::ToggleButton {
    let chip = gtk::ToggleButton::new();
    chip.add_css_class("picker-chip");
    chip.set_child(Some(face));
    chip.set_tooltip_text(Some(tip));
    chip.set_group(group.as_ref());
    chip.set_active(on);
    group.get_or_insert(chip.clone());
    chip.connect_toggled(move |b| {
        if b.is_active() {
            pick();
        }
    });
    chip
}

pub fn glyphs(current: &str, on: impl Fn(&'static str) + 'static) -> gtk::FlowBox {
    let flow = grid(7);
    let on = Rc::new(on);
    let mut group = None;
    let current = if current.is_empty() { "arch" } else { current };
    for (value, label, _) in GLYPHS.iter() {
        let on = on.clone();
        let value: &'static str = value;
        let face = glyph_widget(value, 26);
        flow.insert(&chip(&face, t(label), current == value, &mut group, move || on(value)), -1);
    }
    flow
}

/// The shapes behind a module: rounded rectangles, the clock's bun, and
/// every M3 shape, each in the module's color.
pub fn shapes(current: &str, background: &str, bun: bool, on: impl Fn(&'static str) + 'static) -> gtk::FlowBox {
    let flow = grid(8);
    let on = Rc::new(on);
    let mut group = None;
    let tone = format!("bg-{}", if background == "none" { "tonal" } else { background });
    if bun {
        let face = gtk::Box::new(gtk::Orientation::Vertical, 1);
        for _ in 0..2 {
            let half = gtk::Box::new(gtk::Orientation::Vertical, 0);
            half.add_css_class("picker-bun");
            half.add_css_class(&tone);
            face.append(&half);
        }
        face.set_valign(gtk::Align::Center);
        let on = on.clone();
        flow.insert(&chip(&face, "bun", current == "bun", &mut group, move || on("bun")), -1);
    }
    for (value, label) in CORNERS {
        let face = gtk::Box::new(gtk::Orientation::Vertical, 0);
        face.add_css_class("picker-corner");
        face.add_css_class(&format!("corner-{value}"));
        face.add_css_class(&tone);
        face.set_valign(gtk::Align::Center);
        face.set_halign(gtk::Align::Center);
        let on = on.clone();
        flow.insert(&chip(&face, t(label), current == value, &mut group, move || on(value)), -1);
    }
    for (name, kind) in Kind::NAMED {
        let face = Shape::new(kind, 28, true, None::<&gtk::Widget>);
        face.add_css_class("pm-shape");
        face.add_css_class(&tone);
        let on = on.clone();
        flow.insert(&chip(&face, name, current == name, &mut group, move || on(name)), -1);
    }
    flow
}

/// Whole looks for the module, each shown on the module itself.
pub fn looks(id: &str, m: &Module, bar: &Bar, on: impl Fn(&'static str, &'static str) + 'static) -> gtk::FlowBox {
    let flow = grid(8);
    let on = Rc::new(on);
    let mut group = None;
    for (name, background, shape) in LOOKS {
        let look = Module { background: background.into(), shape: shape.into(), ..m.clone() };
        let face = gtk::Box::new(gtk::Orientation::Vertical, 4);
        face.append(&module(id, &look, bar));
        let label = gtk::Label::new(Some(t(name)));
        label.add_css_class("picker-label");
        face.append(&label);
        let on = on.clone();
        let current = m.background == background && m.shape == shape;
        flow.insert(&chip(&face, t(name), current, &mut group, move || on(background, shape)), -1);
    }
    flow
}

pub fn workspaces(m: &Module, bar: &Bar, on: impl Fn(&'static str) + 'static) -> gtk::FlowBox {
    let flow = grid(5);
    let on = Rc::new(on);
    let mut group = None;
    let current = if m.variant.is_empty() { "dots" } else { m.variant.as_str() };
    for (value, label) in WORKSPACES {
        let style = Module { variant: value.into(), ..m.clone() };
        let face = gtk::Box::new(gtk::Orientation::Vertical, 4);
        face.append(&module("workspaces", &style, bar));
        let name = gtk::Label::new(Some(t(label)));
        name.add_css_class("picker-label");
        face.append(&name);
        let on = on.clone();
        flow.insert(&chip(&face, t(label), current == value, &mut group, move || on(value)), -1);
    }
    flow
}
