//! "adicionar módulo": the catalog beside the preview, like a widget
//! explorer. Search, a category, where it goes; then click a card, or drag
//! it onto the bar. A module dragged from the bar onto it leaves the bar.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};

use super::bar_preview::{place, where_is};
use super::rows::*;
use super::Ctx;
use crate::bar::modules::{Group, CATALOG};
use crate::i18n::t;
use crate::icons;
use crate::widgets::shape::{Kind, Shape};

const FILTERS: [(&str, &str); 6] = [
    ("all", "todos"),
    ("essentials", "essenciais"),
    ("system", "sistema"),
    ("media", "áudio e mídia"),
    ("shortcuts", "atalhos"),
    ("yours", "seus"),
];

fn group_key(group: Group) -> &'static str {
    match group {
        Group::Essentials => "essentials",
        Group::System => "system",
        Group::Media => "media",
        Group::Shortcuts => "shortcuts",
    }
}

pub struct Catalog {
    pub root: gtk::Box,
    ctx: Rc<Ctx>,
    grid: gtk::FlowBox,
    query: RefCell<String>,
    filter: Cell<&'static str>,
    zone: Cell<usize>,
    /// Hears the module just added.
    picked: Box<dyn Fn(&str)>,
}

impl Catalog {
    pub fn new(ctx: &Rc<Ctx>, back: impl Fn() + 'static, create: impl Fn(usize) + 'static, picked: impl Fn(&str) + 'static) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root.add_css_class("bar-catalog");

        let head = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let close = icons::button("arrow_back", t("voltar"));
        close.connect_clicked(move |_| back());
        head.append(&close);
        let title = gtk::Label::new(Some(t("adicionar módulo")));
        title.add_css_class("inspector-title");
        title.set_xalign(0.0);
        title.set_hexpand(true);
        head.append(&title);
        root.append(&head);

        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some(t("procurar módulos")));
        root.append(&search);

        let chips = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        chips.add_css_class("filter-chips");
        root.append(&chips);

        let grid = gtk::FlowBox::new();
        grid.add_css_class("module-catalog");
        grid.set_selection_mode(gtk::SelectionMode::None);
        grid.set_homogeneous(true);
        grid.set_max_children_per_line(3);
        grid.set_min_children_per_line(1);
        grid.set_row_spacing(8);
        grid.set_column_spacing(8);

        let catalog = Rc::new(Self {
            root: root.clone(),
            ctx: ctx.clone(),
            grid: grid.clone(),
            query: RefCell::default(),
            filter: Cell::new("all"),
            zone: Cell::new(2),
            picked: Box::new(picked),
        });

        let mut group: Option<gtk::ToggleButton> = None;
        for (key, label) in FILTERS {
            let chip = gtk::ToggleButton::with_label(t(label));
            chip.add_css_class("filter-chip");
            chip.set_group(group.as_ref());
            chip.set_active(key == "all");
            group.get_or_insert(chip.clone());
            let me = Rc::downgrade(&catalog);
            chip.connect_toggled(move |b| {
                if let (true, Some(c)) = (b.is_active(), me.upgrade()) {
                    c.filter.set(key);
                    c.fill();
                }
            });
            chips.append(&chip);
        }
        let me = Rc::downgrade(&catalog);
        search.connect_search_changed(move |s| {
            if let Some(c) = me.upgrade() {
                *c.query.borrow_mut() = s.text().trim().to_lowercase();
                c.fill();
            }
        });

        let me = Rc::downgrade(&catalog);
        let zone = choices(&[("0", "topo"), ("1", "meio"), ("2", "base")], "2", move |v| {
            if let Some(c) = me.upgrade() {
                c.zone.set(v.parse().unwrap_or(2));
            }
        });
        let into = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let label = gtk::Label::new(Some(t("pôr em")));
        label.add_css_class("settings-item-title");
        into.append(&label);
        into.append(&zone);
        root.append(&into);

        let grid_scroll = scroll(&grid);
        grid_scroll.set_vexpand(true);
        root.append(&grid_scroll);

        let make = action("construction", t("criar um módulo seu"), {
            let me = Rc::downgrade(&catalog);
            move || {
                if let Some(c) = me.upgrade() {
                    create(c.zone.get());
                }
            }
        });
        make.set_halign(gtk::Align::Start);
        root.append(&make);
        root.append(&hint(t("toque num cartão pra pôr na barra, ou arraste pra prévia no lugar exato")));

        // a module dragged from the bar onto the catalog leaves the bar
        let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
        let c = ctx.clone();
        target.connect_drop(move |_, value, _, _| {
            let Ok(payload) = value.get::<String>() else { return false };
            let Some(id) = payload.strip_prefix("move:") else { return false };
            let id = id.to_owned();
            c.edit(|cfg| place(&mut cfg.bar, &id, None));
            true
        });
        root.add_controller(target);

        let me = Rc::downgrade(&catalog);
        ctx.listen(move || {
            let me = me.clone();
            glib::idle_add_local_once(move || {
                if let Some(c) = me.upgrade() {
                    if c.root.is_mapped() {
                        c.fill();
                    }
                }
            });
        });
        catalog.fill();
        catalog
    }

    fn card(self: &Rc<Self>, id: &str, icon: &str, name: &str, about: &str, placed: bool) -> gtk::Button {
        let card = gtk::Button::new();
        card.add_css_class("module-card");
        let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let badge = Shape::new(Kind::Cookie6, 36, true, Some(&icons::icon(icon)));
        badge.add_css_class("module-badge");
        line.append(&badge);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
        texts.set_hexpand(true);
        let title = gtk::Label::new(Some(name));
        title.add_css_class("module-name");
        title.set_xalign(0.0);
        let sub = gtk::Label::new(Some(about));
        sub.add_css_class("settings-item-sub");
        sub.set_xalign(0.0);
        sub.set_wrap(true);
        sub.set_lines(2);
        sub.set_ellipsize(gtk::pango::EllipsizeMode::End);
        texts.append(&title);
        texts.append(&sub);
        line.append(&texts);
        let state = icons::icon(if placed { "check_circle" } else { "add_circle" });
        state.add_css_class("module-state");
        state.set_valign(gtk::Align::Center);
        line.append(&state);
        card.set_child(Some(&line));
        if placed {
            card.add_css_class("placed");
            card.set_tooltip_text(Some(t("já está na barra: toque pra editar")));
        }
        let (id_owned, me) = (id.to_owned(), Rc::downgrade(self));
        card.connect_clicked(move |_| {
            let Some(c) = me.upgrade() else { return };
            if where_is(&c.ctx.get().bar, &id_owned).is_none() {
                let zone = c.zone.get();
                c.ctx.edit(|cfg| place(&mut cfg.bar, &id_owned, Some((zone, if zone == 2 { 0 } else { usize::MAX }))));
            }
            (c.picked)(&id_owned);
        });
        if !placed {
            let source = gtk::DragSource::new();
            source.set_actions(gdk::DragAction::COPY);
            let payload = format!("add:{id}");
            source.connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&payload.to_value())));
            let picture = badge.clone();
            source.connect_drag_begin(move |s, _| s.set_icon(Some(&gtk::WidgetPaintable::new(Some(&picture))), 18, 18));
            card.add_controller(source);
        }
        card
    }

    pub fn fill(self: &Rc<Self>) {
        while let Some(child) = self.grid.first_child() {
            self.grid.remove(&child);
        }
        let bar = self.ctx.get().bar.clone();
        let query = self.query.borrow().clone();
        let filter = self.filter.get();
        let matches = |name: &str, about: &str| query.is_empty() || name.to_lowercase().contains(&query) || about.to_lowercase().contains(&query);
        let mut count = 0;
        for info in CATALOG.iter() {
            if filter != "all" && filter != group_key(info.group) {
                continue;
            }
            let (name, about) = (t(info.name), t(info.about));
            if !matches(name, about) {
                continue;
            }
            self.grid.insert(&self.card(info.id, info.icon, name, about, where_is(&bar, info.id).is_some()), -1);
            count += 1;
        }
        if filter == "all" || filter == "yours" {
            for custom in &bar.custom {
                if !matches(&custom.name, "") {
                    continue;
                }
                let placed = where_is(&bar, &custom.id).is_some();
                self.grid.insert(&self.card(&custom.id, &custom.icon, &custom.name, t("um módulo seu"), placed), -1);
                count += 1;
            }
        }
        if count == 0 {
            let empty = gtk::Label::new(Some(if filter == "yours" {
                t("você ainda não criou nenhum: use \"criar um módulo seu\" embaixo")
            } else {
                t("nenhum módulo com esse nome")
            }));
            empty.add_css_class("settings-hint");
            empty.set_wrap(true);
            self.grid.insert(&empty, -1);
        }
    }
}
