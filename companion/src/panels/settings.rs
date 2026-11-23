use crate::i18n::t;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::bar::MODULES;
use crate::config::Config;
use crate::icons;
use crate::services::Services;
use crate::widgets;

const SECTIONS: [&str; 4] = ["topo", "meio", "base", "oculto"];

/// A small window for the few things worth changing without an editor.
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
            .default_width(560)
            .default_height(760)
            .build();
        window.add_css_class("octopus-settings");
        window.set_hide_on_close(true);

        let column = gtk::Box::new(gtk::Orientation::Vertical, 14);
        column.add_css_class("settings-column");
        let title = gtk::Label::new(Some(t("configurações")));
        title.add_css_class("settings-title");
        title.set_xalign(0.0);
        column.append(&title);

        let modules = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let settings = Rc::new(Self {
            window: window.clone(),
            draft: RefCell::new(config.clone()),
            modules: modules.clone(),
        });

        let theme = card(t("aparência"));
        {
            use crate::services::theme::Mode;
            let current = match *s.theme.mode.get() {
                Mode::Light => 0,
                Mode::Dark => 1,
                Mode::Auto => 2,
            };
            let palette = s.theme.clone();
            theme.append(&row(
                t("modo de cores"),
                &segmented(
                    &[t("claro"), t("escuro"), t("automático")],
                    current,
                    move |i| palette.set_mode([Mode::Light, Mode::Dark, Mode::Auto][i]),
                ),
            ));
            let hint = gtk::Label::new(Some(t("segue o wallpaper")));
            hint.add_css_class("dim");
            hint.set_xalign(1.0);
            theme.append(&hint);

            let languages = ["auto", "pt", "en"];
            let current = languages
                .iter()
                .position(|l| *l == config.language)
                .unwrap_or(0);
            let weak = Rc::downgrade(&settings);
            theme.append(&row(
                t("idioma"),
                &segmented(
                    &[t("sistema"), "português", "english"],
                    current,
                    move |i| {
                        if let Some(s) = weak.upgrade() {
                            s.draft.borrow_mut().language = languages[i].into();
                        }
                    },
                ),
            ));
        }
        column.append(&theme);

        let bar = card(t("barra"));
        {
            let weak = Rc::downgrade(&settings);
            let current = if config.bar.right() { 1 } else { 0 };
            bar.append(&row(
                t("posição"),
                &segmented(&[t("esquerda"), t("direita")], current, move |i| {
                    if let Some(s) = weak.upgrade() {
                        s.draft.borrow_mut().bar.position =
                            if i == 1 { "right" } else { "left" }.into();
                    }
                }),
            ));
            let width = gtk::Scale::with_range(gtk::Orientation::Horizontal, 40.0, 72.0, 2.0);
            width.set_value(config.bar.width as f64);
            width.set_hexpand(true);
            width.set_draw_value(false);
            let value = gtk::Label::new(Some(&format!("{}px", config.bar.width)));
            value.add_css_class("percent");
            let holder = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            holder.set_hexpand(true);
            holder.append(&width);
            holder.append(&value);
            let weak = Rc::downgrade(&settings);
            width.connect_value_changed(move |scale| {
                let w = (scale.value() / 2.0).round() as u32 * 2;
                value.set_label(&format!("{w}px"));
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().bar.width = w;
                }
            });
            bar.append(&row(t("largura"), &holder));
        }
        column.append(&bar);

        let widgets_card = card(t("widgets da barra"));
        widgets_card.append(&modules);
        column.append(&widgets_card);
        settings.rebuild_modules();

        let other = card(t("outros"));
        {
            let place = gtk::Entry::new();
            place.set_text(&config.weather.place);
            place.set_placeholder_text(Some(t("pelo ip")));
            place.set_hexpand(true);
            let weak = Rc::downgrade(&settings);
            place.connect_changed(move |e| {
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().weather.place = e.text().to_string();
                }
            });
            other.append(&row(t("cidade do clima"), &place));

            let seconds = gtk::SpinButton::with_range(2.0, 30.0, 1.0);
            seconds.set_value((config.notifications.timeout_ms / 1000) as f64);
            let weak = Rc::downgrade(&settings);
            seconds.connect_value_changed(move |spin| {
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().notifications.timeout_ms = spin.value() as u32 * 1000;
                }
            });
            other.append(&row(t("notificações (s)"), &seconds));

            let lyrics = gtk::Switch::new();
            lyrics.set_active(config.media.lyrics);
            lyrics.set_halign(gtk::Align::End);
            let weak = Rc::downgrade(&settings);
            lyrics.connect_active_notify(move |sw| {
                if let Some(s) = weak.upgrade() {
                    s.draft.borrow_mut().media.lyrics = sw.is_active();
                }
            });
            other.append(&row(t("letras"), &lyrics));
        }
        column.append(&other);

        let footer = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let note = gtk::Label::new(Some(t(
            "clima, notificações e letras valem depois de reiniciar o octopus",
        )));
        note.add_css_class("dim");
        note.set_wrap(true);
        note.set_xalign(0.0);
        note.set_hexpand(true);
        let save = gtk::Button::with_label(t("aplicar"));
        save.add_css_class("pill");
        save.add_css_class("filled");
        save.set_valign(gtk::Align::Center);
        footer.append(&note);
        footer.append(&save);
        column.append(&footer);

        let weak = Rc::downgrade(&settings);
        save.connect_clicked(move |_| {
            if let Some(s) = weak.upgrade() {
                apply(s.draft.borrow().clone());
            }
        });

        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&column)
            .build();
        window.set_child(Some(&scroller));
        settings
    }

    pub fn present(&self) {
        self.window.present();
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
        {
            let mut draft = self.draft.borrow_mut();
            let bar = &mut draft.bar;
            for list in [&mut bar.top, &mut bar.center, &mut bar.bottom] {
                list.retain(|m| m != module);
            }
            match section {
                0 => draft.bar.top.push(module.into()),
                1 => draft.bar.center.push(module.into()),
                2 => draft.bar.bottom.push(module.into()),
                _ => {}
            }
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
            line.add_css_class("settings-module");
            let label = gtk::Label::new(Some(t(name)));
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
}

fn card(title: &str) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);
    card.add_css_class("card");
    card.append(&widgets::heading(title, None));
    card
}

fn row(label: &str, control: &impl IsA<gtk::Widget>) -> gtk::Box {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    line.add_css_class("settings-row");
    let name = gtk::Label::new(Some(label));
    name.set_xalign(0.0);
    name.set_width_chars(14);
    line.append(&name);
    line.append(control);
    line
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
