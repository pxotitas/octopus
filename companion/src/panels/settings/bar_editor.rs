//! barra › editor: the preview on the left; tap a module to change how it
//! looks and acts in the inspector beside it, drag it to move it, and drag
//! new ones in from the catalog under them (or back there to take them out).

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};

use super::bar_preview::{place, where_is, Hooks, Preview};
use super::rows::*;
use super::Ctx;
use crate::bar::modules::{self, Opt, BACKGROUNDS, CATALOG, GROUPS};
use crate::config::Module;
use crate::i18n::t;
use crate::icons;
use crate::widgets::shape::{Kind, Shape};

struct Editor {
    ctx: Rc<Ctx>,
    inspector: gtk::Box,
    catalog: gtk::Box,
    hooks: Rc<Hooks>,
}

pub fn build(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("editor da barra"), t("toque num módulo pra mudar como ele é; arraste pra trocar de lugar"));

    let inspector = gtk::Box::new(gtk::Orientation::Vertical, 10);
    inspector.add_css_class("bar-inspector");
    inspector.set_hexpand(true);
    let catalog = gtk::Box::new(gtk::Orientation::Vertical, 8);

    let editor = Rc::new_cyclic(|weak: &std::rc::Weak<Editor>| {
        let weak = weak.clone();
        Editor {
            ctx: ctx.clone(),
            inspector: inspector.clone(),
            catalog: catalog.clone(),
            hooks: Rc::new(Hooks {
                select: Box::new(move |id: &str| {
                    if let Some(e) = weak.upgrade() {
                        e.select(id);
                    }
                }),
                selected: RefCell::new(String::new()),
            }),
        }
    });
    let preview = Preview::new(ctx, Some(editor.hooks.clone()));

    // the preview stays put while the side scrolls, so a module can be
    // carried from the catalog to the bar
    let side = gtk::Box::new(gtk::Orientation::Vertical, 6);
    side.add_css_class("editor-side");
    side.append(&inspector);
    side.append(&label(t("adicionar módulos")));
    side.append(&hint(t("arraste um pra prévia, ou toque pra pôr na base; arraste um da prévia pra cá pra tirar")));
    side.append(&catalog);
    side.append(&hint(t("a barra muda ao aplicar, sem reiniciar")));
    let side_scroll = scroll(&side);
    side_scroll.set_hexpand(true);
    side_scroll.set_vexpand(true);
    let split = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    split.set_vexpand(true);
    split.append(&preview.root);
    split.append(&side_scroll);
    column.append(&split);

    editor.select("clock");
    editor.fill_catalog();
    editor.accept_removal();
    let weak = Rc::downgrade(&editor);
    ctx.listen(move || {
        let weak = weak.clone();
        glib::idle_add_local_once(move || {
            if let Some(e) = weak.upgrade() {
                e.fill_catalog();
            }
        });
    });
    // the editor and its preview live as long as their page
    let keep = (editor.clone(), preview.clone());
    column.connect_destroy(move |_| {
        let _ = &keep;
    });
    column.add_css_class("editor-page");
    column.upcast()
}

impl Editor {
    fn edit_module(&self, id: &str, change: impl FnOnce(&mut Module)) {
        self.ctx.edit(|cfg| {
            let mut m = cfg.bar.module(id);
            change(&mut m);
            cfg.bar.modules.insert(id.to_owned(), m);
        });
    }

    fn select(self: &Rc<Self>, id: &str) {
        *self.hooks.selected.borrow_mut() = id.to_owned();
        crate::widgets::clear(&self.inspector);
        let Some(info) = modules::find(id) else { return };
        let bar = self.ctx.get().bar.clone();
        let m = bar.module(id);

        let head = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        let badge = Shape::new(Kind::Cookie9, 48, true, Some(&icons::icon(info.icon)));
        badge.add_css_class("tree-badge");
        head.append(&badge);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
        texts.set_valign(gtk::Align::Center);
        texts.set_hexpand(true);
        let name = gtk::Label::new(Some(t(info.name)));
        name.add_css_class("inspector-title");
        name.set_xalign(0.0);
        let about = gtk::Label::new(Some(t(info.about)));
        about.add_css_class("settings-item-sub");
        about.set_xalign(0.0);
        about.set_wrap(true);
        texts.append(&name);
        texts.append(&about);
        head.append(&texts);
        self.inspector.append(&head);

        let mut items: Vec<gtk::Widget> = Vec::new();
        let place_now = where_is(&bar, id);
        let zone = choices(
            &[("0", "topo"), ("1", "meio"), ("2", "base")],
            &place_now.map(|(z, _)| z.to_string()).unwrap_or_default(),
            {
                let (ctx, id) = (self.ctx.clone(), id.to_owned());
                move |v| {
                    let zone: usize = v.parse().unwrap_or(2);
                    ctx.edit(|cfg| place(&mut cfg.bar, &id, Some((zone, usize::MAX))));
                }
            },
        );
        let order = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        for (glyph, by) in [("arrow_upward", -1i32), ("arrow_downward", 1)] {
            let button = icons::button(glyph, if by < 0 { t("subir") } else { t("descer") });
            let (ctx, id) = (self.ctx.clone(), id.to_owned());
            button.connect_clicked(move |_| {
                ctx.edit(|cfg| {
                    if let Some((z, p)) = where_is(&cfg.bar, &id) {
                        let to = (p as i32 + by).max(0) as usize;
                        place(&mut cfg.bar, &id, Some((z, if by > 0 { to + 1 } else { to })));
                    }
                })
            });
            order.append(&button);
        }
        let position = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        position.append(&zone);
        position.append(&order);
        items.push(stacked("vertical_align_center", t("lugar"), t("topo, meio ou base da coluna"), &position));

        let is_logo = id == "logo";
        for opt in modules::options(id) {
            let id = id.to_owned();
            let me = Rc::downgrade(self);
            let edit = move |change: Box<dyn FnOnce(&mut Module)>| {
                if let Some(e) = me.upgrade() {
                    e.edit_module(&id, change);
                }
            };
            items.push(match opt {
                Opt::Background => stacked(
                    "format_color_fill",
                    t("fundo"),
                    t("a cor atrás do módulo"),
                    &choices(&BACKGROUNDS, &m.background, move |v| edit(Box::new(move |m| m.background = v.into()))),
                ),
                Opt::Shape(options) => stacked(
                    "rounded_corner",
                    t("forma"),
                    t("os cantos do fundo"),
                    &choices(options, &m.shape, move |v| edit(Box::new(move |m| m.shape = v.into()))),
                ),
                Opt::Variant(title, options) if is_logo => stacked("star", t(title), t("o glifo no topo da barra"), &glyphs(options, &m.variant, edit)),
                Opt::Variant(title, options) => stacked(
                    "tune",
                    t(title),
                    "",
                    &choices(options, &m.variant, move |v| edit(Box::new(move |m| m.variant = v.into()))),
                ),
                Opt::Action(options) => stacked(
                    "touch_app",
                    t("ao clicar"),
                    t("o que abre"),
                    &choices(options, &m.action, move |v| edit(Box::new(move |m| m.action = v.into()))),
                ),
                Opt::Text => inline(
                    "pin",
                    t("mostrar o valor"),
                    t("o número embaixo do ícone"),
                    &switch(m.text, move |on| edit(Box::new(move |m| m.text = on))),
                ),
                Opt::Items(options) => {
                    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
                    let edit = Rc::new(edit);
                    for (value, label) in options.iter().copied() {
                        let edit = edit.clone();
                        let on = m.items.iter().any(|i| i == value);
                        let check = gtk::CheckButton::with_label(t(label));
                        check.set_active(on);
                        check.connect_toggled(move |c| {
                            let active = c.is_active();
                            edit(Box::new(move |m| {
                                m.items.retain(|i| i != value);
                                if active {
                                    let at = modules::CONTROLS.iter().position(|(v, _)| *v == value).unwrap_or(0);
                                    let before = m.items.iter().filter(|i| modules::CONTROLS.iter().position(|(v, _)| v == *i).unwrap_or(0) < at).count();
                                    m.items.insert(before, value.into());
                                }
                            }));
                        });
                        list.append(&check);
                    }
                    stacked("checklist", t("botões"), t("quais aparecem, na ordem da barra"), &list)
                }
            });
        }
        let refs: Vec<&gtk::Widget> = items.iter().collect();
        self.inspector.append(&group(&refs));

        let remove = action("remove_circle", t("tirar da barra"), {
            let (ctx, id, me) = (self.ctx.clone(), id.to_owned(), Rc::downgrade(self));
            move || {
                ctx.edit(|cfg| place(&mut cfg.bar, &id, None));
                if let Some(e) = me.upgrade() {
                    crate::widgets::clear(&e.inspector);
                    e.inspector.append(&empty());
                }
            }
        });
        remove.add_css_class("danger");
        remove.set_halign(gtk::Align::Start);
        self.inspector.append(&remove);
        // the preview marks the selection on its next rebuild
        self.ctx.edit(|_| {});
    }

    /// The catalog, by group: placed modules are marked, the rest can be
    /// dragged in or clicked to land at the bottom.
    fn fill_catalog(self: &Rc<Self>) {
        crate::widgets::clear(&self.catalog);
        let bar = self.ctx.get().bar.clone();
        for (group, name) in GROUPS {
            let flow = gtk::FlowBox::new();
            flow.add_css_class("module-catalog");
            flow.set_selection_mode(gtk::SelectionMode::None);
            flow.set_homogeneous(true);
            flow.set_max_children_per_line(3);
            flow.set_min_children_per_line(2);
            flow.set_row_spacing(8);
            flow.set_column_spacing(8);
            for info in CATALOG.iter().filter(|i| i.group == group) {
                let placed = where_is(&bar, info.id).is_some();
                let card = gtk::Button::new();
                card.add_css_class("module-card");
                let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
                let badge = Shape::new(Kind::Cookie6, 34, true, Some(&icons::icon(info.icon)));
                badge.add_css_class("module-badge");
                line.append(&badge);
                let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
                texts.set_hexpand(true);
                let title = gtk::Label::new(Some(t(info.name)));
                title.add_css_class("module-name");
                title.set_xalign(0.0);
                let about = gtk::Label::new(Some(if placed { t("na barra") } else { t(info.about) }));
                about.add_css_class("settings-item-sub");
                about.set_xalign(0.0);
                about.set_ellipsize(gtk::pango::EllipsizeMode::End);
                about.set_max_width_chars(24);
                texts.append(&title);
                texts.append(&about);
                line.append(&texts);
                line.append(&icons::icon(if placed { "check" } else { "add" }));
                card.set_child(Some(&line));
                if placed {
                    card.add_css_class("placed");
                }
                let (id, me) = (info.id, Rc::downgrade(self));
                card.connect_clicked(move |_| {
                    let Some(e) = me.upgrade() else { return };
                    if where_is(&e.ctx.get().bar, id).is_none() {
                        e.ctx.edit(|cfg| place(&mut cfg.bar, id, Some((2, 0))));
                    }
                    e.select(id);
                });
                if !placed {
                    let source = gtk::DragSource::new();
                    source.set_actions(gdk::DragAction::COPY);
                    let payload = format!("add:{}", info.id);
                    source.connect_prepare(move |_, _, _| Some(gdk::ContentProvider::for_value(&payload.to_value())));
                    let picture = badge.clone();
                    source.connect_drag_begin(move |s, _| s.set_icon(Some(&gtk::WidgetPaintable::new(Some(&picture))), 17, 17));
                    card.add_controller(source);
                }
                flow.insert(&card, -1);
            }
            self.catalog.append(&label(t(name)));
            self.catalog.append(&flow);
        }
    }

    fn accept_removal(self: &Rc<Self>) {
        // dropping a module from the preview here takes it out of the bar
        let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE | gdk::DragAction::COPY);
        let me = Rc::downgrade(self);
        target.connect_drop(move |_, value, _, _| {
            let (Some(e), Ok(payload)) = (me.upgrade(), value.get::<String>()) else { return false };
            let Some(id) = payload.strip_prefix("move:") else { return false };
            let id = id.to_owned();
            e.ctx.edit(|cfg| place(&mut cfg.bar, &id, None));
            true
        });
        self.catalog.add_controller(target);
    }
}

fn empty() -> gtk::Label {
    let label = gtk::Label::new(Some(t("toque num módulo da prévia pra mudar como ele é")));
    label.add_css_class("settings-hint");
    label.set_wrap(true);
    label
}

/// The logo's glyphs as chips that show themselves.
fn glyphs(options: &'static [(&'static str, &'static str)], current: &str, edit: impl Fn(Box<dyn FnOnce(&mut Module)>) + 'static) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let edit = Rc::new(edit);
    let mut first: Option<gtk::ToggleButton> = None;
    for (value, label) in options.iter().copied() {
        let chip = gtk::ToggleButton::new();
        chip.add_css_class("glyph-chip");
        let glyph = if value == "arch" {
            let l = gtk::Label::new(Some("\u{f303}"));
            l.add_css_class("pm-arch");
            l
        } else {
            icons::icon(value)
        };
        chip.set_child(Some(&glyph));
        chip.set_tooltip_text(Some(t(label)));
        chip.set_group(first.as_ref());
        chip.set_active(value == current || (current.is_empty() && value == "arch"));
        first.get_or_insert(chip.clone());
        let edit = edit.clone();
        chip.connect_toggled(move |b| {
            if b.is_active() {
                edit(Box::new(move |m| m.variant = value.into()));
            }
        });
        row.append(&chip);
    }
    row
}
