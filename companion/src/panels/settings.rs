use crate::i18n::t;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::bar::MODULES;
use crate::config::Config;
use crate::icons;
use crate::services::Services;
use crate::widgets;
use crate::widgets::shape::{Kind, Shape};

const SECTIONS: [&str; 4] = ["topo", "meio", "base", "oculto"];

/// The settings window: an M3 navigation rail on the left, one page per
/// part of the shell, and an "apply" button that writes octopus.toml.
/// Changes stay in a draft until applied.
pub struct Settings {
    pub window: gtk::Window,
    draft: RefCell<Config>,
    modules: gtk::Box,
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
            .default_width(820)
            .default_height(620)
            .build();
        window.add_css_class("octopus-settings");

        let modules = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let settings = Rc::new(Self {
            window: window.clone(),
            draft: RefCell::new(config.clone()),
            modules: modules.clone(),
        });

        let pages = gtk::Stack::new();
        pages.set_transition_type(gtk::StackTransitionType::SlideUpDown);
        pages.set_transition_duration(260);
        pages.set_hexpand(true);

        pages.add_named(&settings.look_page(s, &config), Some("look"));
        pages.add_named(&settings.bar_page(&config), Some("bar"));
        let widgets_page = page(t("widgets da barra"), t("onde cada coisa fica na barra"));
        widgets_page.append(&group(&[modules.upcast_ref()]));
        pages.add_named(&scroll(&widgets_page), Some("widgets"));
        settings.rebuild_modules();
        pages.add_named(&settings.dock_page(&config), Some("dock"));
        pages.add_named(&settings.other_page(&config), Some("other"));

        let rail = gtk::Box::new(gtk::Orientation::Vertical, 6);
        rail.add_css_class("settings-rail");
        rail.set_hexpand(false);
        let badge = Shape::new(Kind::Cookie9, 52, true, Some(&icons::icon("settings")));
        badge.add_css_class("settings-badge");
        badge.set_margin_bottom(18);
        rail.append(&badge);
        let mut first: Option<gtk::ToggleButton> = None;
        for (name, icon, label) in [
            ("look", "palette", t("aparência")),
            ("bar", "side_navigation", t("barra")),
            ("widgets", "widgets", t("widgets")),
            ("dock", "dock_to_bottom", t("dock")),
            ("other", "tune", t("outros")),
        ] {
            let button = gtk::ToggleButton::new();
            button.add_css_class("rail-item");
            let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
            let pill = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            pill.add_css_class("rail-pill");
            pill.set_halign(gtk::Align::Center);
            let glyph = icons::icon(icon);
            glyph.set_hexpand(true);
            pill.append(&glyph);
            column.append(&pill);
            column.append(&gtk::Label::new(Some(label)));
            button.set_child(Some(&column));
            button.set_group(first.as_ref());
            let (pages, badge) = (pages.clone(), badge.clone());
            button.connect_toggled(move |b| {
                if b.is_active() {
                    pages.set_visible_child_name(name);
                    badge.adapt(&[]);
                }
            });
            if first.is_none() {
                button.set_active(true);
            }
            first.get_or_insert(button.clone());
            rail.append(&button);
        }

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
        let weak = Rc::downgrade(&settings);
        save.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                apply(s.draft.borrow().clone());
            }
        });

        let body = gtk::Overlay::new();
        body.set_hexpand(true);
        body.set_child(Some(&pages));
        body.add_overlay(&save);

        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.append(&rail);
        root.append(&body);
        window.set_child(Some(&root));
        settings
    }

    fn look_page(self: &Rc<Self>, s: &Rc<Services>, config: &Config) -> gtk::ScrolledWindow {
        use crate::services::theme::Mode;
        let column = page(t("aparência"), t("cores e idioma do shell"));
        let current = match *s.theme.mode.get() {
            Mode::Light => 0,
            Mode::Dark => 1,
            Mode::Auto => 2,
        };
        let palette = s.theme.clone();
        let mode = segmented(
            &[t("claro"), t("escuro"), t("automático")],
            current,
            move |i| palette.set_mode([Mode::Light, Mode::Dark, Mode::Auto][i]),
        );
        let languages = ["auto", "pt", "en"];
        let current = languages
            .iter()
            .position(|l| *l == config.language)
            .unwrap_or(0);
        let weak = Rc::downgrade(self);
        let language = segmented(
            &[t("sistema"), "português", "english"],
            current,
            move |i| {
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().language = languages[i].into();
                }
            },
        );
        column.append(&group(&[
            stacked(
                "contrast",
                t("modo de cores"),
                t("o automático segue o wallpaper"),
                &mode,
            )
            .upcast_ref(),
            stacked(
                "translate",
                t("idioma"),
                t("reinicia o octopus ao aplicar"),
                &language,
            )
            .upcast_ref(),
        ]));
        scroll(&column)
    }

    fn bar_page(self: &Rc<Self>, config: &Config) -> gtk::ScrolledWindow {
        let column = page(t("barra"), t("a coluna da waybar"));
        let weak = Rc::downgrade(self);
        let style = segmented(
            &[t("flutuante"), t("encostada")],
            usize::from(config.bar.style == "flush"),
            move |i| {
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().bar.style =
                        if i == 1 { "flush" } else { "floating" }.into();
                }
            },
        );
        let weak = Rc::downgrade(self);
        let position = segmented(
            &[t("esquerda"), t("direita")],
            usize::from(config.bar.right()),
            move |i| {
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().bar.position =
                        if i == 1 { "right" } else { "left" }.into();
                }
            },
        );
        let width = gtk::Scale::with_range(gtk::Orientation::Horizontal, 40.0, 72.0, 2.0);
        width.set_value(config.bar.width as f64);
        width.set_hexpand(true);
        width.set_draw_value(false);
        let value = gtk::Label::new(Some(&format!("{}px", config.bar.width)));
        value.add_css_class("percent");
        let holder = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        holder.append(&width);
        holder.append(&value);
        let weak = Rc::downgrade(self);
        width.connect_value_changed(move |scale| {
            let w = (scale.value() / 2.0).round() as u32 * 2;
            value.set_label(&format!("{w}px"));
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().bar.width = w;
            }
        });
        column.append(&group(&[
            stacked(
                "rounded_corner",
                t("estilo"),
                t("encostada fica colada na borda, com cantos côncavos"),
                &style,
            )
            .upcast_ref(),
            stacked("swap_horiz", t("posição"), t("lado da tela"), &position).upcast_ref(),
            stacked("width", t("largura"), t("em pixels"), &holder).upcast_ref(),
        ]));
        scroll(&column)
    }

    fn dock_page(self: &Rc<Self>, config: &Config) -> gtk::ScrolledWindow {
        let column = page(t("dock"), t("apps fixados e abertos, embaixo da tela"));
        let enabled = gtk::Switch::new();
        enabled.set_active(config.dock.enabled);
        enabled.set_valign(gtk::Align::Center);
        let weak = Rc::downgrade(self);
        enabled.connect_active_notify(move |sw| {
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().dock.enabled = sw.is_active();
            }
        });
        let modes = ["fixed", "autohide", "smart"];
        let current = modes
            .iter()
            .position(|m| *m == config.dock.mode)
            .unwrap_or(2);
        let weak = Rc::downgrade(self);
        let mode = segmented(
            &[t("fixa"), t("esconde sozinha"), t("inteligente")],
            current,
            move |i| {
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().dock.mode = modes[i].into();
                }
            },
        );
        let size = gtk::Scale::with_range(gtk::Orientation::Horizontal, 28.0, 64.0, 2.0);
        size.set_value(config.dock.size as f64);
        size.set_hexpand(true);
        size.set_draw_value(false);
        let value = gtk::Label::new(Some(&format!("{}px", config.dock.size)));
        value.add_css_class("percent");
        let holder = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        holder.append(&size);
        holder.append(&value);
        let weak = Rc::downgrade(self);
        size.connect_value_changed(move |scale| {
            let px = (scale.value() / 2.0).round() as u32 * 2;
            value.set_label(&format!("{px}px"));
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().dock.size = px;
            }
        });
        let magnify = gtk::Switch::new();
        magnify.set_active(config.dock.magnify);
        magnify.set_valign(gtk::Align::Center);
        let weak = Rc::downgrade(self);
        magnify.connect_active_notify(move |sw| {
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().dock.magnify = sw.is_active();
            }
        });
        column.append(&group(&[
            inline("dock_to_bottom", t("mostrar a dock"), t("reinicia o octopus ao aplicar"), &enabled)
                .upcast_ref(),
            stacked("photo_size_select_small", t("tamanho dos ícones"), t("em pixels"), &holder)
                .upcast_ref(),
            inline("zoom_in", t("magnificação"), t("os ícones crescem embaixo do ponteiro"), &magnify)
                .upcast_ref(),
            stacked(
                "visibility",
                t("comportamento"),
                t("fixa ocupa espaço; esconde sozinha aparece na borda; inteligente sai da frente das janelas"),
                &mode,
            )
            .upcast_ref(),
        ]));
        let hint = gtk::Label::new(Some(t(
            "clique direito num ícone da dock pra fixar ou desafixar",
        )));
        hint.add_css_class("settings-hint");
        hint.set_xalign(0.0);
        hint.set_wrap(true);
        column.append(&hint);
        scroll(&column)
    }

    fn other_page(self: &Rc<Self>, config: &Config) -> gtk::ScrolledWindow {
        let column = page(t("outros"), t("clima, notificações e música"));
        let place = gtk::Entry::new();
        place.set_text(&config.weather.place);
        place.set_placeholder_text(Some(t("pelo ip")));
        let weak = Rc::downgrade(self);
        place.connect_changed(move |e| {
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().weather.place = e.text().to_string();
            }
        });
        let seconds = gtk::SpinButton::with_range(2.0, 30.0, 1.0);
        seconds.set_value((config.notifications.timeout_ms / 1000) as f64);
        seconds.set_valign(gtk::Align::Center);
        let weak = Rc::downgrade(self);
        seconds.connect_value_changed(move |spin| {
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().notifications.timeout_ms = spin.value() as u32 * 1000;
            }
        });
        let lyrics = gtk::Switch::new();
        lyrics.set_active(config.media.lyrics);
        lyrics.set_valign(gtk::Align::Center);
        let weak = Rc::downgrade(self);
        lyrics.connect_active_notify(move |sw| {
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().media.lyrics = sw.is_active();
            }
        });
        let offset = gtk::Scale::with_range(gtk::Orientation::Horizontal, -1.5, 1.5, 0.05);
        offset.set_value(config.media.lyrics_offset_ms as f64 / 1000.0);
        offset.set_hexpand(true);
        offset.set_draw_value(false);
        offset.add_mark(0.0, gtk::PositionType::Bottom, None);
        let shown = gtk::Label::new(Some(&format!(
            "{:+.2}s",
            config.media.lyrics_offset_ms as f64 / 1000.0
        )));
        shown.add_css_class("percent");
        let offset_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        offset_row.append(&offset);
        offset_row.append(&shown);
        let weak = Rc::downgrade(self);
        offset.connect_value_changed(move |scale| {
            let ms = (scale.value() * 20.0).round() as i64 * 50;
            shown.set_label(&format!("{:+.2}s", ms as f64 / 1000.0));
            crate::services::lyrics::set_offset(ms);
            if let Some(s) = weak.upgrade() {
                s.draft.borrow_mut().media.lyrics_offset_ms = ms;
            }
        });
        column.append(&group(&[
            stacked(
                "location_on",
                t("cidade do clima"),
                t("vazio usa o ip"),
                &place,
            )
            .upcast_ref(),
            inline(
                "notifications",
                t("notificações"),
                t("segundos na tela"),
                &seconds,
            )
            .upcast_ref(),
            inline(
                "lyrics",
                t("letras"),
                t("no dash e na tela de bloqueio"),
                &lyrics,
            )
            .upcast_ref(),
            stacked(
                "sync",
                t("tempo da letra"),
                t("adiantar (+) ou atrasar (-) as letras, se não baterem com a música"),
                &offset_row,
            )
            .upcast_ref(),
        ]));
        let hint = gtk::Label::new(Some(t(
            "clima, notificações e letras valem depois de reiniciar o octopus",
        )));
        hint.add_css_class("settings-hint");
        hint.set_xalign(0.0);
        hint.set_wrap(true);
        column.append(&hint);
        scroll(&column)
    }

    fn section_of(&self, module: &str) -> (usize, usize) {
        let draft = self.draft.borrow();
        for (i, list) in [&draft.bar.top, &draft.bar.center, &draft.bar.bottom]
            .into_iter()
            .enumerate()
        {
            if let Some(pos) = list.iter().position(|m| m == module) {
                return (i, pos);
            }
        }
        (3, 0)
    }

    fn move_to(&self, module: &str, section: usize) {
        let mut draft = self.draft.borrow_mut();
        let bar = &mut draft.bar;
        for list in [&mut bar.top, &mut bar.center, &mut bar.bottom] {
            list.retain(|m| m != module);
        }
        match section {
            0 => bar.top.push(module.into()),
            1 => bar.center.push(module.into()),
            2 => bar.bottom.push(module.into()),
            _ => {}
        }
    }

    fn shift(&self, module: &str, by: i32) {
        let mut draft = self.draft.borrow_mut();
        let bar = &mut draft.bar;
        for list in [&mut bar.top, &mut bar.center, &mut bar.bottom] {
            if let Some(pos) = list.iter().position(|m| m == module) {
                let to = pos as i32 + by;
                if to >= 0 && (to as usize) < list.len() {
                    list.swap(pos, to as usize);
                }
            }
        }
    }

    fn rebuild_modules(self: &Rc<Self>) {
        widgets::clear(&self.modules);
        let mut ordered: Vec<(usize, usize, &str, &str)> = MODULES
            .iter()
            .map(|(id, name)| {
                let (section, pos) = self.section_of(id);
                (section, pos, *id, *name)
            })
            .collect();
        ordered.sort();
        for (section, _, id, name) in ordered {
            let line = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            line.add_css_class("settings-item");
            let label = gtk::Label::new(Some(t(name)));
            label.add_css_class("settings-item-title");
            label.set_xalign(0.0);
            label.set_hexpand(true);
            line.append(&label);

            let up = icons::button("arrow_upward", t("subir"));
            let down = icons::button("arrow_downward", t("descer"));
            up.set_sensitive(section < 3);
            down.set_sensitive(section < 3);
            let names: Vec<&str> = SECTIONS.iter().map(|s| t(s)).collect();
            let where_ = gtk::DropDown::from_strings(&names);
            where_.set_selected(section as u32);
            where_.set_valign(gtk::Align::Center);
            line.append(&up);
            line.append(&down);
            line.append(&where_);

            let weak = Rc::downgrade(self);
            up.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.shift(id, -1);
                    s.rebuild_modules();
                }
            });
            let weak = Rc::downgrade(self);
            down.connect_clicked(move |_| {
                if let Some(s) = weak.upgrade() {
                    s.shift(id, 1);
                    s.rebuild_modules();
                }
            });
            let weak = Rc::downgrade(self);
            where_.connect_selected_notify(move |dd| {
                let Some(s) = weak.upgrade() else { return };
                let to = dd.selected() as usize;
                gtk::glib::idle_add_local_once(move || {
                    s.move_to(id, to);
                    s.rebuild_modules();
                });
            });
            self.modules.append(&line);
        }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn page(title: &str, subtitle: &str) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 6);
    column.add_css_class("settings-page");
    let heading = gtk::Label::new(Some(title));
    heading.add_css_class("settings-title");
    heading.set_xalign(0.0);
    let sub = gtk::Label::new(Some(subtitle));
    sub.add_css_class("settings-subtitle");
    sub.set_xalign(0.0);
    column.append(&heading);
    column.append(&sub);
    column
}

fn scroll(child: &gtk::Box) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(child)
        .build()
}

/// Items in one rounded group, M3 list style: the first and last round off.
fn group(items: &[&gtk::Widget]) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 2);
    column.add_css_class("settings-group");
    for item in items {
        column.append(*item);
    }
    column
}

fn lead(icon: &str, title: &str, subtitle: &str) -> gtk::Box {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    let glyph = icons::icon(icon);
    glyph.add_css_class("settings-icon");
    glyph.set_valign(gtk::Align::Center);
    line.append(&glyph);
    let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
    texts.set_valign(gtk::Align::Center);
    texts.set_hexpand(true);
    let name = gtk::Label::new(Some(title));
    name.add_css_class("settings-item-title");
    name.set_xalign(0.0);
    let sub = gtk::Label::new(Some(subtitle));
    sub.add_css_class("settings-item-sub");
    sub.set_xalign(0.0);
    sub.set_wrap(true);
    texts.append(&name);
    texts.append(&sub);
    line.append(&texts);
    line
}

/// Icon and text with the control at the end of the line.
fn inline(icon: &str, title: &str, subtitle: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
    let line = lead(icon, title, subtitle);
    line.add_css_class("settings-item");
    line.append(control);
    line
}

/// Icon and text with a wide control under them.
fn stacked(icon: &str, title: &str, subtitle: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
    let item = gtk::Box::new(gtk::Orientation::Vertical, 12);
    item.add_css_class("settings-item");
    item.append(&lead(icon, title, subtitle));
    item.append(control);
    item
}

/// M3 segmented buttons.
fn segmented(options: &[&str], selected: usize, on_change: impl Fn(usize) + 'static) -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    group.add_css_class("segmented");
    group.set_hexpand(true);
    group.set_homogeneous(true);
    let on_change = Rc::new(on_change);
    let mut first: Option<gtk::ToggleButton> = None;
    for (i, option) in options.iter().enumerate() {
        let button = gtk::ToggleButton::with_label(option);
        button.set_group(first.as_ref());
        button.set_active(i == selected);
        let on_change = on_change.clone();
        button.connect_toggled(move |b| {
            if b.is_active() {
                on_change(i);
            }
        });
        first.get_or_insert(button.clone());
        group.append(&button);
    }
    group
}
