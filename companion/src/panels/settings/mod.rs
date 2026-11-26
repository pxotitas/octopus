//! The settings window: a tree of major categories and their categories on
//! the left, the page on the right, and "apply" writing octopus.toml.
//! Pages are built the first time they are opened; changes stay in a draft
//! until applied.

mod bar_anchor;
mod bar_custom;
mod bar_editor;
mod bar_pickers;
mod bar_presets;
mod bar_preview;
mod bar_shape;
mod desktop;
mod look;
mod panels;
mod rows;
mod system;
mod tree;

use std::cell::{Ref, RefCell};
use std::collections::HashSet;
use std::rc::Rc;

use gtk::prelude::*;

use crate::config::Config;
use crate::i18n::t;
use crate::icons;
use crate::services::Services;
use crate::widgets::shape::Kind;
use tree::{Category, Major};

const fn category(id: &'static str, name: &'static str, about: &'static str) -> Category {
    Category { id, name, about }
}

pub const MAJORS: [Major; 5] = [
    Major {
        id: "look",
        name: "aparência",
        icon: "palette",
        shape: Kind::Flower,
        categories: &[
            category("colors", "cores", "o modo, o estilo e o contraste da paleta do wallpaper"),
            category("language", "idioma", "a língua do shell"),
        ],
    },
    Major {
        id: "bar",
        name: "barra",
        icon: "side_navigation",
        shape: Kind::Rounded,
        categories: &[
            category("editor", "editor", "arraste os módulos, toque num pra mudar como ele é"),
            category("presets", "presets", "barras inteiras pra começar"),
            category("shape", "forma", "largura, estilo e espaçamento"),
            category("anchor", "ancoragem", "lado, camada e distância das bordas"),
        ],
    },
    Major {
        id: "panels",
        name: "painéis",
        icon: "dashboard",
        shape: Kind::Cookie9,
        categories: &[
            category("dash", "dash", "o que abre no relógio"),
            category("notifications", "notificações", "quanto tempo e onde aparecem"),
            category("weather", "clima", "onde e em que unidade"),
            category("music", "música", "letras e o tempo delas"),
        ],
    },
    Major {
        id: "desktop",
        name: "área de trabalho",
        icon: "desktop_windows",
        shape: Kind::Clover4,
        categories: &[
            category("widgets", "widgets", "o que fica embaixo das janelas"),
            category("contributions", "contribuições", "o calendário do github"),
            category("dock", "dock", "apps fixados e abertos"),
        ],
    },
    Major {
        id: "system",
        name: "sistema",
        icon: "settings_suggest",
        shape: Kind::Cookie6,
        categories: &[
            category("lock", "tela de bloqueio", "o que aparece quando trava"),
            category("about", "sobre", "versão, links e reiniciar"),
        ],
    },
];

/// What every page shares: the draft and the services.
pub struct Ctx {
    draft: RefCell<Config>,
    applied: RefCell<Config>,
    pub services: Rc<Services>,
    listeners: RefCell<Vec<Box<dyn Fn()>>>,
}

impl Ctx {
    pub fn get(&self) -> Ref<'_, Config> {
        self.draft.borrow()
    }

    pub fn edit(&self, change: impl FnOnce(&mut Config)) {
        change(&mut self.draft.borrow_mut());
        for listener in self.listeners.borrow().iter() {
            listener();
        }
    }

    /// Runs after every change to the draft.
    pub fn listen(&self, f: impl Fn() + 'static) {
        self.listeners.borrow_mut().push(Box::new(f));
    }

    fn dirty(&self) -> bool {
        *self.draft.borrow() != *self.applied.borrow()
    }
}

pub struct Settings {
    pub window: gtk::Window,
    ctx: Rc<Ctx>,
    pages: gtk::Stack,
    built: RefCell<HashSet<String>>,
    tree: Rc<tree::Tree>,
    crumb: gtk::Label,
}

fn build_page(ctx: &Rc<Ctx>, id: &str) -> gtk::Widget {
    match id {
        "look/colors" => look::colors(ctx),
        "look/language" => look::language(ctx),
        "bar/editor" => bar_editor::build(ctx),
        "bar/presets" => bar_presets::build(ctx),
        "bar/shape" => bar_shape::build(ctx),
        "bar/anchor" => bar_anchor::build(ctx),
        "panels/dash" => panels::dash(ctx),
        "panels/notifications" => panels::notifications(ctx),
        "panels/weather" => panels::weather(ctx),
        "panels/music" => panels::music(ctx),
        "desktop/widgets" => desktop::widgets(ctx),
        "desktop/contributions" => desktop::contributions(ctx),
        "desktop/dock" => desktop::dock(ctx),
        "system/lock" => system::lock(ctx),
        _ => system::about(ctx),
    }
}

fn find(id: &str) -> Option<(&'static Major, &'static Category)> {
    let (major, category) = id.split_once('/')?;
    let major = MAJORS.iter().find(|m| m.id == major)?;
    Some((major, major.categories.iter().find(|c| c.id == category)?))
}

impl Settings {
    pub fn new(
        app: &gtk::Application,
        s: &Rc<Services>,
        config: Config,
        apply: Rc<dyn Fn(Config)>,
    ) -> Rc<Self> {
        let window = gtk::Window::builder()
            .application(app)
            .title("octopus")
            .default_width(1040)
            .default_height(700)
            .build();
        window.add_css_class("octopus-settings");

        let ctx = Rc::new(Ctx {
            draft: RefCell::new(config.clone()),
            applied: RefCell::new(config),
            services: s.clone(),
            listeners: RefCell::default(),
        });

        let pages = gtk::Stack::new();
        pages.set_transition_type(gtk::StackTransitionType::Crossfade);
        pages.set_transition_duration(180);
        pages.set_hexpand(true);
        pages.set_vexpand(true);

        let crumb = gtk::Label::new(None);
        crumb.add_css_class("settings-crumb");
        crumb.set_xalign(0.0);

        let settings = Rc::new_cyclic(|weak: &std::rc::Weak<Settings>| {
            let weak = weak.clone();
            let tree = tree::Tree::new(
                &MAJORS,
                Rc::new(move |page: &str| {
                    if let Some(s) = weak.upgrade() {
                        s.open(page);
                    }
                }),
            );
            Self {
                window: window.clone(),
                ctx: ctx.clone(),
                pages: pages.clone(),
                built: RefCell::default(),
                tree,
                crumb: crumb.clone(),
            }
        });

        let side = gtk::Box::new(gtk::Orientation::Vertical, 8);
        side.add_css_class("settings-side");
        side.set_hexpand(false);
        let title = gtk::Label::new(Some(t("configurações")));
        title.add_css_class("settings-app-title");
        title.set_xalign(0.0);
        side.append(&title);
        let tree_scroll = rows::scroll(&settings.tree.root);
        tree_scroll.set_vexpand(true);
        side.append(&tree_scroll);

        let save = gtk::Button::new();
        save.add_css_class("settings-fab");
        let fab = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        fab.append(&icons::icon("check"));
        fab.append(&gtk::Label::new(Some(t("aplicar"))));
        save.set_child(Some(&fab));
        save.set_halign(gtk::Align::End);
        save.set_valign(gtk::Align::End);
        save.set_margin_end(24);
        save.set_margin_bottom(24);
        save.set_sensitive(false);
        let weak = Rc::downgrade(&ctx);
        let button = save.clone();
        ctx.listen(move || {
            if let Some(ctx) = weak.upgrade() {
                button.set_sensitive(ctx.dirty());
            }
        });
        let weak = Rc::downgrade(&ctx);
        save.connect_clicked(move |button| {
            let Some(ctx) = weak.upgrade() else { return };
            let next = ctx.draft.borrow().clone();
            let look_changed = next.look != ctx.applied.borrow().look;
            *ctx.applied.borrow_mut() = next.clone();
            button.set_sensitive(false);
            if look_changed {
                crate::config::write_look(&next.look);
                let mode = *ctx.services.theme.mode.get();
                ctx.services.theme.set_mode(mode);
            }
            apply(next);
        });

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.append(&crumb);
        content.append(&pages);
        let body = gtk::Overlay::new();
        body.set_hexpand(true);
        body.set_child(Some(&content));
        body.add_overlay(&save);

        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.append(&side);
        root.append(&body);
        window.set_child(Some(&root));

        settings.tree.select("look/colors");
        settings
    }

    fn open(&self, page: &str) {
        let Some((major, category)) = find(page) else { return };
        if self.built.borrow_mut().insert(page.to_owned()) {
            self.pages.add_named(&build_page(&self.ctx, page), Some(page));
        }
        self.pages.set_visible_child_name(page);
        self.crumb.set_label(&format!("{}  ›  {}", t(major.name), t(category.name)));
        let _ = category.about;
        tree::follow(&self.tree)(page);
    }

    /// Opens a page by "major/category" or a major's name.
    pub fn show_page(&self, page: &str) {
        self.tree.select(page);
    }

    pub fn present(&self) {
        self.window.present();
    }
}
