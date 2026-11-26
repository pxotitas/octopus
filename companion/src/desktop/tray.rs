//! The case: a card at the bottom of the screen while arranging, holding
//! every widget. Drag one out onto the desktop, or click it to place it in
//! the first free spot; drop a widget back on the case to put it away.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::gdk;

use super::catalog::{self, ENTRIES};
use crate::config;
use crate::i18n::t;
use crate::icons;
use crate::widgets::shape::Shape;

struct Tile {
    id: &'static str,
    button: gtk::Button,
    state: gtk::Label,
    about: gtk::Label,
}

pub struct Tray {
    pub root: gtk::Box,
    tiles: Vec<Tile>,
    hint: gtk::Label,
}

impl Tray {
    pub fn new(desktop: &config::Desktop, on_add: Rc<dyn Fn(&str)>, on_done: Rc<dyn Fn()>) -> Self {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root.add_css_class("desktop-case");
        root.set_halign(gtk::Align::Center);
        root.set_valign(gtk::Align::End);
        root.set_margin_bottom(24);

        let head = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let badge = Shape::new(crate::widgets::shape::Kind::Cookie6, 40, true, Some(&icons::icon("business_center")));
        badge.add_css_class("case-badge");
        head.append(&badge);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
        texts.set_hexpand(true);
        texts.set_valign(gtk::Align::Center);
        let title = gtk::Label::new(Some(t("widgets")));
        title.add_css_class("case-title");
        title.set_xalign(0.0);
        let hint = gtk::Label::new(Some(t("arraste um widget pra área de trabalho, ou solte um aqui pra guardar")));
        hint.add_css_class("dim");
        hint.set_xalign(0.0);
        texts.append(&title);
        texts.append(&hint);
        head.append(&texts);
        let done = gtk::Button::new();
        done.add_css_class("case-done");
        let label = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        label.append(&icons::icon("check"));
        label.append(&gtk::Label::new(Some(t("pronto"))));
        done.set_child(Some(&label));
        done.set_valign(gtk::Align::Center);
        done.connect_clicked(move |_| on_done());
        head.append(&done);
        root.append(&head);

        let flow = gtk::FlowBox::new();
        flow.add_css_class("case-grid");
        flow.set_selection_mode(gtk::SelectionMode::None);
        flow.set_homogeneous(true);
        flow.set_min_children_per_line(3);
        flow.set_max_children_per_line(3);
        flow.set_column_spacing(10);
        flow.set_row_spacing(10);

        let mut tiles = Vec::new();
        for entry in ENTRIES.iter() {
            let button = gtk::Button::new();
            button.add_css_class("case-tile");
            let column = gtk::Box::new(gtk::Orientation::Vertical, 6);
            let top = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            let shape = Shape::new(entry.shape, 48, true, Some(&icons::icon(entry.icon)));
            shape.add_css_class("case-tile-badge");
            shape.set_hexpand(true);
            shape.set_halign(gtk::Align::Start);
            top.append(&shape);
            let state = icons::icon("add");
            state.add_css_class("case-tile-state");
            state.set_valign(gtk::Align::Start);
            top.append(&state);
            column.append(&top);
            let name = gtk::Label::new(Some(t(entry.title)));
            name.add_css_class("case-tile-title");
            name.set_xalign(0.0);
            let about = gtk::Label::new(Some(t(entry.about)));
            about.add_css_class("dim");
            about.set_xalign(0.0);
            about.set_wrap(true);
            about.set_lines(2);
            about.set_ellipsize(gtk::pango::EllipsizeMode::End);
            column.append(&name);
            column.append(&about);
            button.set_child(Some(&column));

            let id = entry.id;
            let add = on_add.clone();
            button.connect_clicked(move |_| add(id));

            let source = gtk::DragSource::new();
            source.set_actions(gdk::DragAction::COPY);
            source.connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&id.to_value())));
            let picture = shape.clone();
            source.connect_drag_begin(move |source, _| {
                let paintable = gtk::WidgetPaintable::new(Some(&picture));
                source.set_icon(Some(&paintable), 24, 24);
            });
            button.add_controller(source);

            flow.insert(&button, -1);
            tiles.push(Tile { id, button, state, about });
        }

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_height(176)
            .max_content_height(176)
            .propagate_natural_height(true)
            .child(&flow)
            .build();
        scroller.set_size_request(600, -1);
        root.append(&scroller);

        let tray = Self { root, tiles, hint };
        tray.refresh(desktop, &[]);
        tray
    }

    /// Marks what is already on the desktop and what can't be placed yet.
    pub fn refresh(&self, desktop: &config::Desktop, placed: &[String]) {
        for tile in &self.tiles {
            let entry = catalog::entry(tile.id).expect("in the catalog");
            let on = placed.iter().any(|p| catalog::entry(p).is_some_and(|e| e.id == tile.id));
            let usable = catalog::available(tile.id, desktop);
            tile.button.set_sensitive(usable && !on);
            if on {
                tile.button.add_css_class("placed");
            } else {
                tile.button.remove_css_class("placed");
            }
            tile.state.set_label(if on { "check_circle" } else if usable { "add" } else { "block" });
            tile.about.set_label(if usable {
                t(entry.about)
            } else {
                t("precisa de um usuário do github nas configurações")
            });
        }
    }

    /// Lights up while a widget is held over the case.
    pub fn set_receiving(&self, on: bool) {
        if on {
            self.root.add_css_class("receiving");
            self.hint.set_label(t("solte pra guardar"));
        } else {
            self.root.remove_css_class("receiving");
            self.hint.set_label(t("arraste um widget pra área de trabalho, ou solte um aqui pra guardar"));
        }
    }

    pub fn contains(&self, fixed: &impl IsA<gtk::Widget>, x: f64, y: f64) -> bool {
        self.root
            .compute_bounds(fixed)
            .is_some_and(|b| b.contains_point(&gtk::graphene::Point::new(x as f32, y as f32)))
    }
}
