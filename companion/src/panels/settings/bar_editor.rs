//! barra › editor. The bar on the left, as it will look; beside it, the
//! inspector for the module you tapped, in three parts: where it sits, how
//! it looks, what it does. The "+" under the bar adds one from the catalog
//! or makes one of your own; dragging a module onto "remove" takes it out.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib};

use super::bar_catalog::Catalog;
use super::bar_preview::{place, where_is, Hooks, Preview};
use super::rows::*;
use super::{bar_maker, bar_pickers, Ctx};
use crate::bar::modules::{self, Opt, BACKGROUNDS};
use crate::config::Module;
use crate::i18n::t;
use crate::icons;
use crate::widgets::shape::{Kind, Shape};

struct Editor {
    ctx: Rc<Ctx>,
    side: gtk::Stack,
    inspector: gtk::Box,
    catalog: RefCell<Option<Rc<Catalog>>>,
    hooks: Rc<Hooks>,
}

pub fn build(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("editor da barra"), t("toque num módulo pra mudar como ele é; arraste pra trocar de lugar; o + adiciona"));
    column.add_css_class("editor-page");

    let inspector = gtk::Box::new(gtk::Orientation::Vertical, 6);
    inspector.add_css_class("bar-inspector");
    let side = gtk::Stack::new();
    side.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
    side.set_transition_duration(240);
    side.set_hexpand(true);
    side.set_vexpand(true);
    let inspector_scroll = scroll(&inspector);
    side.add_named(&inspector_scroll, Some("inspector"));

    let editor = Rc::new_cyclic(|weak: &std::rc::Weak<Editor>| {
        let weak = weak.clone();
        Editor {
            ctx: ctx.clone(),
            side: side.clone(),
            inspector: inspector.clone(),
            catalog: RefCell::default(),
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

    // under the bar: add, and a place to drop what should go
    let left = gtk::Box::new(gtk::Orientation::Vertical, 12);
    left.set_valign(gtk::Align::Start);
    left.append(&preview.root);
    let add = gtk::MenuButton::new();
    add.add_css_class("editor-add");
    let inside = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    inside.append(&icons::icon("add"));
    inside.append(&gtk::Label::new(Some(t("adicionar"))));
    add.set_child(Some(&inside));
    add.set_popover(Some(&editor.add_menu()));
    left.append(&add);
    left.append(&editor.trash());

    let split = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    split.set_vexpand(true);
    split.append(&left);
    split.append(&side);
    column.append(&split);

    editor.select("clock");
    let keep = (editor.clone(), preview.clone());
    column.connect_destroy(move |_| {
        let _ = &keep;
    });
    column.upcast()
}

impl Editor {
    fn add_menu(self: &Rc<Self>) -> gtk::Popover {
        let popover = gtk::Popover::new();
        popover.add_css_class("editor-menu");
        popover.set_has_arrow(false);
        let column = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let row = |icon: &str, title: &str, about: &str| {
            let button = gtk::Button::new();
            button.add_css_class("menu-row");
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            let badge = Shape::new(Kind::Cookie6, 36, true, Some(&icons::icon(icon)));
            badge.add_css_class("module-badge");
            line.append(&badge);
            let texts = gtk::Box::new(gtk::Orientation::Vertical, 0);
            let name = gtk::Label::new(Some(title));
            name.add_css_class("module-name");
            name.set_xalign(0.0);
            let sub = gtk::Label::new(Some(about));
            sub.add_css_class("settings-item-sub");
            sub.set_xalign(0.0);
            texts.append(&name);
            texts.append(&sub);
            line.append(&texts);
            button.set_child(Some(&line));
            button
        };
        let add = row("add_box", t("adicionar módulo"), t("um dos prontos, do catálogo"));
        let (me, pop) = (Rc::downgrade(self), popover.clone());
        add.connect_clicked(move |_| {
            pop.popdown();
            if let Some(e) = me.upgrade() {
                e.show_catalog();
            }
        });
        let create = row("construction", t("criar módulo"), t("um ícone, um comando, um clique"));
        let (me, pop) = (Rc::downgrade(self), popover.clone());
        create.connect_clicked(move |b| {
            pop.popdown();
            if let Some(e) = me.upgrade() {
                e.make(b, None, 2);
            }
        });
        column.append(&add);
        column.append(&create);
        popover.set_child(Some(&column));
        popover
    }

    fn trash(self: &Rc<Self>) -> gtk::Box {
        let zone = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        zone.add_css_class("editor-trash");
        zone.set_halign(gtk::Align::Center);
        zone.append(&icons::icon("delete"));
        zone.append(&gtk::Label::new(Some(t("solte aqui pra tirar"))));
        let target = gtk::DropTarget::new(String::static_type(), gdk::DragAction::MOVE);
        let z = zone.clone();
        target.connect_enter(move |_, _, _| {
            z.add_css_class("hover");
            gdk::DragAction::MOVE
        });
        let z = zone.clone();
        target.connect_leave(move |_| z.remove_css_class("hover"));
        let (me, z) = (Rc::downgrade(self), zone.clone());
        target.connect_drop(move |_, value, _, _| {
            z.remove_css_class("hover");
            let (Some(e), Ok(payload)) = (me.upgrade(), value.get::<String>()) else { return false };
            let Some(id) = payload.strip_prefix("move:") else { return false };
            let id = id.to_owned();
            e.ctx.edit(|cfg| place(&mut cfg.bar, &id, None));
            e.empty();
            true
        });
        zone.add_controller(target);
        zone
    }

    fn show_catalog(self: &Rc<Self>) {
        if self.catalog.borrow().is_none() {
            let (back, create, picked) = (Rc::downgrade(self), Rc::downgrade(self), Rc::downgrade(self));
            let catalog = Catalog::new(
                &self.ctx,
                move || {
                    if let Some(e) = back.upgrade() {
                        e.side.set_visible_child_name("inspector");
                    }
                },
                move |zone| {
                    if let Some(e) = create.upgrade() {
                        let anchor = e.side.clone();
                        e.make(&anchor, None, zone);
                    }
                },
                move |id| {
                    if let Some(e) = picked.upgrade() {
                        e.select(id);
                    }
                },
            );
            self.side.add_named(&catalog.root, Some("catalog"));
            *self.catalog.borrow_mut() = Some(catalog);
        }
        if let Some(c) = self.catalog.borrow().as_ref() {
            c.fill();
        }
        self.side.set_visible_child_name("catalog");
    }

    /// The module maker, for a new module (`id` None) or one of yours.
    fn make(self: &Rc<Self>, anchor: &impl IsA<gtk::Widget>, id: Option<&str>, zone: usize) {
        let Some(window) = anchor.as_ref().root().and_downcast::<gtk::Window>() else { return };
        let me = Rc::downgrade(self);
        bar_maker::open(&window, &self.ctx, id, zone, move |id| {
            if let Some(e) = me.upgrade() {
                e.select(id);
            }
        });
    }

    /// Changes a module's look; the inspector redraws so the pickers show it.
    fn edit_module(self: &Rc<Self>, id: &str, change: impl FnOnce(&mut Module)) {
        self.ctx.edit(|cfg| {
            let mut m = cfg.bar.module(id);
            change(&mut m);
            cfg.bar.modules.insert(id.to_owned(), m);
        });
        let (me, id) = (Rc::downgrade(self), id.to_owned());
        glib::idle_add_local_once(move || {
            if let Some(e) = me.upgrade() {
                e.select(&id);
            }
        });
    }

    fn empty(self: &Rc<Self>) {
        *self.hooks.selected.borrow_mut() = String::new();
        crate::widgets::clear(&self.inspector);
        let state = gtk::Box::new(gtk::Orientation::Vertical, 12);
        state.add_css_class("editor-empty");
        state.set_valign(gtk::Align::Center);
        state.set_vexpand(true);
        let badge = Shape::new(Kind::Clover4, 72, true, Some(&icons::icon("touch_app")));
        badge.add_css_class("tree-badge");
        badge.set_halign(gtk::Align::Center);
        state.append(&badge);
        let title = gtk::Label::new(Some(t("escolha um módulo na barra")));
        title.add_css_class("inspector-title");
        state.append(&title);
        let sub = gtk::Label::new(Some(t("ou use o + embaixo dela pra adicionar um")));
        sub.add_css_class("settings-item-sub");
        state.append(&sub);
        self.inspector.append(&state);
        self.side.set_visible_child_name("inspector");
        self.ctx.edit(|_| {});
    }

    fn head(&self, icon: &str, name: &str, about: &str) {
        let head = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        head.add_css_class("inspector-head");
        let badge = Shape::new(Kind::Cookie9, 52, true, Some(&icons::icon(icon)));
        badge.add_css_class("tree-badge");
        head.append(&badge);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
        texts.set_valign(gtk::Align::Center);
        texts.set_hexpand(true);
        let title = gtk::Label::new(Some(name));
        title.add_css_class("inspector-title");
        title.set_xalign(0.0);
        let sub = gtk::Label::new(Some(about));
        sub.add_css_class("settings-item-sub");
        sub.set_xalign(0.0);
        sub.set_wrap(true);
        texts.append(&title);
        texts.append(&sub);
        head.append(&texts);
        self.inspector.append(&head);
    }

    fn select(self: &Rc<Self>, id: &str) {
        let bar = self.ctx.get().bar.clone();
        let custom = id.starts_with("user-");
        let known = if custom { bar.custom(id).is_some() } else { modules::find(id).is_some() };
        if !known {
            return self.empty();
        }
        *self.hooks.selected.borrow_mut() = id.to_owned();
        crate::widgets::clear(&self.inspector);
        self.side.set_visible_child_name("inspector");
        if custom {
            let c = bar.custom(id).cloned().unwrap_or_default();
            self.head(&c.icon, &c.name, t("um módulo seu"));
        } else if let Some(info) = modules::find(id) {
            self.head(info.icon, t(info.name), t(info.about));
        }
        let m = bar.module(id);

        // where it sits
        self.inspector.append(&label(t("lugar")));
        let zone = choices(
            &[("0", "topo"), ("1", "meio"), ("2", "base")],
            &where_is(&bar, id).map(|(z, _)| z.to_string()).unwrap_or_default(),
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
        self.inspector.append(&group(&[&stacked("vertical_align_center", t("parte da barra"), t("topo, meio ou base; as setas mudam a ordem"), &position)]));

        let mut look: Vec<gtk::Widget> = Vec::new();
        let mut acts: Vec<gtk::Widget> = Vec::new();
        if modules::takes_looks(id) {
            let me = Rc::downgrade(self);
            let name = id.to_owned();
            let looks = bar_pickers::looks(id, &m, &bar, move |background, shape| {
                if let Some(e) = me.upgrade() {
                    e.edit_module(&name, |m| {
                        m.background = background.into();
                        m.shape = shape.into();
                    });
                }
            });
            look.push(stacked("auto_awesome", t("presets do módulo"), t("um visual inteiro num toque"), &looks));
        }
        for opt in modules::options(id) {
            let me = Rc::downgrade(self);
            let name = id.to_owned();
            let edit = move |change: Box<dyn FnOnce(&mut Module)>| {
                if let Some(e) = me.upgrade() {
                    e.edit_module(&name, change);
                }
            };
            match opt {
                Opt::Custom => {
                    let c = bar.custom(id).cloned().unwrap_or_default();
                    let summary = if c.exec.trim().is_empty() { t("só o ícone").to_owned() } else { format!("{} · {}s", c.exec, c.interval) };
                    let edit_button = action("edit", t("editar módulo"), {
                        let (me, id) = (Rc::downgrade(self), id.to_owned());
                        move || {
                            if let Some(e) = me.upgrade() {
                                let anchor = e.inspector.clone();
                                e.make(&anchor, Some(&id), 2);
                            }
                        }
                    });
                    acts.push(inline("terminal", t("o que ele faz"), &summary, &edit_button));
                }
                Opt::Background => look.push(stacked(
                    "format_color_fill",
                    t("fundo"),
                    t("a cor atrás do módulo"),
                    &choices(&BACKGROUNDS, &m.background, move |v| edit(Box::new(move |m| m.background = v.into()))),
                )),
                Opt::Shape { bun } => look.push(stacked(
                    "interests",
                    t("forma"),
                    t("o que fica atrás do módulo: cantos arredondados ou um shape m3"),
                    &bar_pickers::shapes(&m.shape, &m.background, bun, move |v| edit(Box::new(move |m| m.shape = v.into()))),
                )),
                Opt::Variant(title, _) if id == "logo" => look.push(stacked(
                    "star",
                    t(title),
                    t("o octopus, a sua distro ou um símbolo"),
                    &bar_pickers::glyphs(&m.variant, move |v| edit(Box::new(move |m| m.variant = v.into()))),
                )),
                Opt::Variant(title, _) if id == "workspaces" => look.push(stacked(
                    "more_vert",
                    t(title),
                    t("como cada workspace aparece; o atual se estica"),
                    &bar_pickers::workspaces(&m, &bar, move |v| edit(Box::new(move |m| m.variant = v.into()))),
                )),
                Opt::Variant(title, options) => acts.push(stacked(
                    "tune",
                    t(title),
                    "",
                    &choices(options, &m.variant, move |v| edit(Box::new(move |m| m.variant = v.into()))),
                )),
                Opt::Action(options) => acts.push(stacked(
                    "touch_app",
                    t("ao clicar"),
                    t("o que abre"),
                    &choices(options, &m.action, move |v| edit(Box::new(move |m| m.action = v.into()))),
                )),
                Opt::Text => acts.push(inline(
                    "pin",
                    t("mostrar o valor"),
                    t("o número embaixo do ícone"),
                    &switch(m.text, move |on| edit(Box::new(move |m| m.text = on))),
                )),
                Opt::Items(options) => {
                    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
                    let edit = Rc::new(edit);
                    for (value, label) in options.iter().copied() {
                        let edit = edit.clone();
                        let check = gtk::CheckButton::with_label(t(label));
                        check.set_active(m.items.iter().any(|i| i == value));
                        check.connect_toggled(move |c| {
                            let active = c.is_active();
                            edit(Box::new(move |m| {
                                m.items.retain(|i| i != value);
                                if active {
                                    let rank = |v: &str| modules::CONTROLS.iter().position(|(c, _)| *c == v).unwrap_or(0);
                                    let at = m.items.iter().filter(|i| rank(i) < rank(value)).count();
                                    m.items.insert(at, value.into());
                                }
                            }));
                        });
                        list.append(&check);
                    }
                    acts.push(stacked("checklist", t("botões"), t("quais aparecem, na ordem da barra"), &list));
                }
            }
        }
        if !look.is_empty() {
            self.inspector.append(&label(t("visual")));
            let refs: Vec<&gtk::Widget> = look.iter().collect();
            self.inspector.append(&group(&refs));
        }
        if !acts.is_empty() {
            self.inspector.append(&label(t("comportamento")));
            let refs: Vec<&gtk::Widget> = acts.iter().collect();
            self.inspector.append(&group(&refs));
        }

        let buttons = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        buttons.add_css_class("inspector-actions");
        let remove = action("remove_circle", t("tirar da barra"), {
            let (ctx, id, me) = (self.ctx.clone(), id.to_owned(), Rc::downgrade(self));
            move || {
                ctx.edit(|cfg| place(&mut cfg.bar, &id, None));
                if let Some(e) = me.upgrade() {
                    e.empty();
                }
            }
        });
        remove.add_css_class("danger");
        buttons.append(&remove);
        if custom {
            let delete = action("delete", t("apagar módulo"), {
                let (ctx, id, me) = (self.ctx.clone(), id.to_owned(), Rc::downgrade(self));
                move || {
                    ctx.edit(|cfg| {
                        place(&mut cfg.bar, &id, None);
                        cfg.bar.custom.retain(|c| c.id != id);
                        cfg.bar.modules.remove(&id);
                    });
                    if let Some(e) = me.upgrade() {
                        e.empty();
                    }
                }
            });
            delete.add_css_class("danger");
            buttons.append(&delete);
        }
        self.inspector.append(&buttons);
        // the preview marks the selection on its next rebuild
        self.ctx.edit(|_| {});
    }
}
