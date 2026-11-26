//! The settings' navigation: major categories as the trunks, their
//! categories as branches drawn off a line, like
//!
//!     bar
//!      ├ editor
//!      └ presets

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;
use crate::widgets::shape::{Kind, Shape};

pub struct Category {
    pub id: &'static str,
    pub name: &'static str,
    pub about: &'static str,
}

pub struct Major {
    pub id: &'static str,
    pub name: &'static str,
    pub icon: &'static str,
    pub shape: Kind,
    pub categories: &'static [Category],
}

/// One branch: the line coming down from the trunk, curving into the leaf.
fn branch(last: bool) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.add_css_class("tree-branch");
    area.set_content_width(26);
    area.set_draw_func(move |area, cr, w, h| {
        let ink = area.color();
        cr.set_source_rgba(ink.red() as f64, ink.green() as f64, ink.blue() as f64, ink.alpha() as f64);
        cr.set_line_width(2.0);
        cr.set_line_cap(gtk::cairo::LineCap::Round);
        let (x, mid, r) = (12.0, h as f64 / 2.0, 7.0);
        cr.move_to(x, 0.0);
        cr.line_to(x, mid - r);
        cr.arc_negative(x + r, mid - r, r, std::f64::consts::PI, std::f64::consts::FRAC_PI_2);
        cr.line_to(w as f64 - 2.0, mid);
        if !last {
            cr.move_to(x, mid - r);
            cr.line_to(x, h as f64);
        }
        let _ = cr.stroke();
    });
    area
}

pub struct Tree {
    pub root: gtk::Box,
    leaves: Vec<(String, gtk::ToggleButton)>,
    reveal: Vec<(&'static str, gtk::Revealer, gtk::Button)>,
}

/// Opens one major and closes the rest, like an accordion.
fn unfold(reveal: &[(&'static str, gtk::Revealer, gtk::Button)], major: &str, open: bool) {
    for (name, revealer, trunk) in reveal {
        let this = *name == major && open;
        revealer.set_reveal_child(this);
        if let Some(chevron) = trunk.child().and_then(|line| line.last_child()) {
            if this {
                chevron.add_css_class("open");
            } else {
                chevron.remove_css_class("open");
            }
        }
    }
}

impl Tree {
    pub fn new(majors: &'static [Major], on_select: Rc<dyn Fn(&str)>) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 2);
        root.add_css_class("settings-tree");
        let mut leaves = Vec::new();
        let mut reveal = Vec::new();
        let mut group: Option<gtk::ToggleButton> = None;

        for major in majors {
            let trunk = gtk::Button::new();
            trunk.add_css_class("tree-major");
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            let badge = Shape::new(major.shape, 32, true, Some(&icons::icon(major.icon)));
            badge.add_css_class("tree-badge");
            line.append(&badge);
            let name = gtk::Label::new(Some(t(major.name)));
            name.set_xalign(0.0);
            name.set_hexpand(true);
            line.append(&name);
            let chevron = icons::icon("expand_more");
            chevron.add_css_class("tree-chevron");
            line.append(&chevron);
            trunk.set_child(Some(&line));
            root.append(&trunk);

            let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
            column.add_css_class("tree-branches");
            for (i, category) in major.categories.iter().enumerate() {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
                row.append(&branch(i + 1 == major.categories.len()));
                let leaf = gtk::ToggleButton::with_label(t(category.name));
                leaf.add_css_class("tree-leaf");
                leaf.set_hexpand(true);
                if let Some(label) = leaf.child().and_downcast::<gtk::Label>() {
                    label.set_xalign(0.0);
                }
                leaf.set_group(group.as_ref());
                group.get_or_insert(leaf.clone());
                let id = format!("{}/{}", major.id, category.id);
                let (select, page) = (on_select.clone(), id.clone());
                leaf.connect_toggled(move |b| {
                    if b.is_active() {
                        select(&page);
                    }
                });
                row.append(&leaf);
                column.append(&row);
                leaves.push((id, leaf));
            }
            let revealer = gtk::Revealer::builder()
                .transition_type(gtk::RevealerTransitionType::SlideDown)
                .transition_duration(220)
                .child(&column)
                .build();
            root.append(&revealer);

            reveal.push((major.id, revealer, trunk));
        }

        let tree = Rc::new(Self { root, leaves, reveal });
        for (name, revealer, trunk) in &tree.reveal {
            let (weak, name, revealer) = (Rc::downgrade(&tree), *name, revealer.clone());
            trunk.connect_clicked(move |_| {
                if let Some(tree) = weak.upgrade() {
                    unfold(&tree.reveal, name, !revealer.reveals_child());
                }
            });
        }
        tree
    }

    /// Selects a page by "major/category", or a major's first category.
    pub fn select(&self, page: &str) {
        let found = self
            .leaves
            .iter()
            .find(|(id, _)| id == page)
            .or_else(|| self.leaves.iter().find(|(id, _)| id.split('/').next() == Some(page)));
        let Some((id, leaf)) = found else { return };
        let major = id.split('/').next().unwrap_or_default();
        unfold(&self.reveal, major, true);
        leaf.set_active(true);
    }
}

/// Keeps the trunk of the page on screen highlighted as pages change.
pub fn follow(tree: &Rc<Tree>) -> impl Fn(&str) {
    let tree = Rc::downgrade(tree);
    let last: RefCell<String> = RefCell::default();
    move |page: &str| {
        let major = page.split('/').next().unwrap_or_default().to_owned();
        if *last.borrow() == major {
            return;
        }
        if let Some(tree) = tree.upgrade() {
            for (name, _, trunk) in &tree.reveal {
                if *name == major {
                    trunk.add_css_class("current");
                } else {
                    trunk.remove_css_class("current");
                }
            }
        }
        *last.borrow_mut() = major;
    }
}
