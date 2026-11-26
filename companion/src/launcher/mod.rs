mod apps;
mod calc;
mod files;
pub mod rofi;

use std::cell::{Cell, RefCell};
use std::f64::consts::TAU;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, gio, glib, pango};

use crate::exec;
use crate::i18n::t;
use crate::icons;
use crate::widgets::shape::{Kind, Shape};
use apps::Apps;

const LIMIT: usize = 8;

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Apps,
    Calc,
    Files,
    Power,
    Remind,
}

impl Mode {
    fn look(self) -> (Kind, &'static str) {
        match self {
            Mode::Apps => (Kind::Cookie9, "search"),
            Mode::Calc => (Kind::Cookie6, "calculate"),
            Mode::Files => (Kind::Clover4, "folder_open"),
            Mode::Power => (Kind::Sunny, "power_settings_new"),
            Mode::Remind => (Kind::Clover8, "alarm"),
        }
    }
}

enum Item {
    App(usize),
    Calc(f64),
    File(PathBuf),
    Power(&'static str),
    Remind(String, bool),
    Hint,
}

fn power_actions() -> [(&'static str, &'static str, &'static str); 5] {
    [
        ("lock", "lock", t("bloquear")),
        ("suspend", "bedtime", t("suspender")),
        ("logout", "logout", t("sair")),
        ("reboot", "restart_alt", t("reiniciar")),
        ("poweroff", "power_settings_new", t("desligar")),
    ]
}

/// The spotlight: apps and the actions their .desktop files offer, a
/// calculator, files behind `/` and the session behind `>`.
pub struct Launcher {
    pub root: gtk::Box,
    entry: gtk::Text,
    badge: Shape,
    glyph: gtk::Label,
    list: gtk::ListBox,
    apps: RefCell<Option<Apps>>,
    items: RefCell<Vec<Item>>,
    mode: Cell<Mode>,
    generation: Cell<u64>,
    close: Rc<dyn Fn()>,
    power: Rc<dyn Fn(&str)>,
    remind: Rc<dyn Fn(&str, bool)>,
}

impl Launcher {
    pub fn new(close: Rc<dyn Fn()>, power: Rc<dyn Fn(&str)>, remind: Rc<dyn Fn(&str, bool)>) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
        root.add_css_class("surface");
        root.add_css_class("launcher");

        let bar = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        bar.add_css_class("launcher-bar");
        let glyph = icons::icon("search");
        let badge = Shape::new(Kind::Cookie9, 46, true, Some(&glyph));
        badge.add_css_class("launcher-badge");
        let entry = gtk::Text::new();
        entry.add_css_class("launcher-entry");
        entry.set_hexpand(true);
        entry.set_placeholder_text(Some(t("buscar apps, arquivos e contas")));
        bar.append(&badge);
        bar.append(&entry);
        root.append(&bar);

        let list = gtk::ListBox::new();
        list.add_css_class("launcher-list");
        list.set_selection_mode(gtk::SelectionMode::Browse);
        list.set_activate_on_single_click(true);
        root.append(&list);

        let chips = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        chips.add_css_class("launcher-chips");
        root.append(&chips);

        let launcher = Rc::new(Self {
            root,
            entry: entry.clone(),
            badge,
            glyph,
            list: list.clone(),
            apps: RefCell::new(None),
            items: RefCell::new(Vec::new()),
            mode: Cell::new(Mode::Apps),
            generation: Cell::new(0),
            close,
            power,
            remind,
        });

        for (prefix, label) in [
            ("=", t("calcular")),
            ("/", t("arquivos")),
            ("!", t("lembrete")),
            (">", t("energia")),
        ] {
            let chip = gtk::Button::new();
            chip.add_css_class("launcher-chip");
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            let key = gtk::Label::new(Some(prefix));
            key.add_css_class("launcher-key");
            content.append(&key);
            content.append(&gtk::Label::new(Some(label)));
            chip.set_child(Some(&content));
            let entry = entry.clone();
            chip.connect_clicked(move |_| {
                entry.set_text(prefix);
                entry.grab_focus_without_selecting();
                entry.set_position(-1);
            });
            chips.append(&chip);
        }

        let weak = Rc::downgrade(&launcher);
        let length = Cell::new(0usize);
        entry.connect_changed(move |entry| {
            let Some(l) = weak.upgrade() else { return };
            let now = entry.text().chars().count();
            if now != length.replace(now) {
                l.badge.adapt(&[]);
            }
            l.query();
        });
        let weak = Rc::downgrade(&launcher);
        entry.connect_activate(move |_| {
            if let Some(l) = weak.upgrade() {
                if let Some(row) = l.list.selected_row() {
                    l.activate(row.index());
                }
            }
        });
        let weak = Rc::downgrade(&launcher);
        list.connect_row_activated(move |_, row| {
            if let Some(l) = weak.upgrade() {
                l.activate(row.index());
            }
        });

        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(&launcher);
        keys.connect_key_pressed(move |_, key, _, _| {
            let Some(l) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let step = match key {
                gdk::Key::Down | gdk::Key::Tab => 1,
                gdk::Key::Up | gdk::Key::ISO_Left_Tab => -1,
                _ => return glib::Propagation::Proceed,
            };
            l.step(step);
            glib::Propagation::Stop
        });
        launcher.root.add_controller(keys);
        launcher
    }

    pub fn open(self: &Rc<Self>) {
        if self.apps.borrow().is_none() {
            *self.apps.borrow_mut() = Some(Apps::load());
        }
        self.entry.set_text("");
        self.badge.morph(Kind::Cookie9);
        self.query();
        self.entry.grab_focus_without_selecting();
    }

    fn step(&self, by: i32) {
        let count = self.items.borrow().len() as i32;
        if count == 0 {
            return;
        }
        let current = self.list.selected_row().map(|r| r.index()).unwrap_or(0);
        let next = (current + by).rem_euclid(count);
        if let Some(row) = self.list.row_at_index(next) {
            self.list.select_row(Some(&row));
            row.grab_focus();
            self.entry.grab_focus_without_selecting();
        }
    }

    fn set_mode(&self, mode: Mode) {
        if self.mode.replace(mode) == mode {
            return;
        }
        let (kind, glyph) = mode.look();
        if self.entry.text().is_empty() {
            self.badge.morph(kind);
            self.badge.turn(TAU / 4.0, 520);
        }
        self.glyph.set_label(glyph);
    }

    fn query(self: &Rc<Self>) {
        let text = self.entry.text().to_string();
        let generation = self.generation.get() + 1;
        self.generation.set(generation);

        if let Some(rest) = text.strip_prefix('/') {
            self.set_mode(Mode::Files);
            let rest = rest.trim().to_owned();
            if rest.is_empty() {
                self.show(vec![Item::Hint]);
                return;
            }
            let weak = Rc::downgrade(self);
            glib::timeout_add_local_once(Duration::from_millis(140), move || {
                let Some(l) = weak.upgrade() else { return };
                if l.generation.get() != generation {
                    return;
                }
                glib::spawn_future_local(async move {
                    let found = files::search(&rest, LIMIT).await;
                    if l.generation.get() == generation {
                        l.show(found.into_iter().map(Item::File).collect());
                    }
                });
            });
            return;
        }
        if let Some(rest) = text.strip_prefix('!') {
            self.set_mode(Mode::Remind);
            let rest = rest.trim().to_owned();
            let items = match crate::services::reminders::parse::parse(&rest, chrono::Local::now()) {
                Some(q) if q.alarm => vec![Item::Remind(rest.clone(), true), Item::Remind(rest, false)],
                Some(_) => vec![Item::Remind(rest.clone(), false), Item::Remind(rest, true)],
                None => vec![Item::Hint],
            };
            self.show(items);
            return;
        }
        if let Some(rest) = text.strip_prefix('>') {
            self.set_mode(Mode::Power);
            let rest = rest.trim().to_lowercase();
            let items = power_actions()
                .into_iter()
                .filter(|(_, _, label)| label.to_lowercase().contains(&rest))
                .map(|(action, _, _)| Item::Power(action))
                .collect();
            self.show(items);
            return;
        }

        let mut items = Vec::new();
        match calc::evaluate(&text) {
            Some(value) => {
                self.set_mode(Mode::Calc);
                items.push(Item::Calc(value));
            }
            None => self.set_mode(Mode::Apps),
        }
        if let Some(apps) = self.apps.borrow().as_ref() {
            let room = LIMIT - items.len();
            items.extend(apps.search(&text, room).into_iter().map(Item::App));
        }
        self.show(items);
    }

    fn show(&self, items: Vec<Item>) {
        while let Some(row) = self.list.row_at_index(0) {
            self.list.remove(&row);
        }
        for (i, item) in items.iter().enumerate() {
            let row = self.row(item);
            row.add_css_class(&format!("d{}", i.min(7)));
            self.list.append(&row);
        }
        self.list.set_visible(!items.is_empty());
        if let Some(first) = self.list.row_at_index(0) {
            self.list.select_row(Some(&first));
        }
        *self.items.borrow_mut() = items;
        crate::popup::fit(&self.root);
    }

    fn row(&self, item: &Item) -> gtk::ListBoxRow {
        let row = gtk::ListBoxRow::new();
        row.add_css_class("launcher-row");
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        let (lead, title, detail): (gtk::Widget, String, String) = match item {
            Item::App(i) => {
                let apps = self.apps.borrow();
                let entry = apps.as_ref().unwrap().get(*i);
                let image = match &entry.icon {
                    Some(icon) => gtk::Image::from_gicon(icon),
                    None => gtk::Image::from_icon_name("application-x-executable"),
                };
                image.set_pixel_size(32);
                image.add_css_class("launcher-icon");
                let detail = if entry.action.is_some() {
                    format!("{} · {}", t("ação"), entry.detail)
                } else {
                    entry.detail.clone()
                };
                (image.upcast(), entry.name.clone(), detail)
            }
            Item::Calc(value) => (
                badge("equal"),
                calc::format(*value),
                t("enter copia o resultado").to_owned(),
            ),
            Item::File(path) => {
                let (icon, _) = gio::content_type_guess(Some(path), None);
                let image = gtk::Image::from_gicon(&gio::content_type_get_icon(&icon));
                image.set_pixel_size(32);
                image.add_css_class("launcher-icon");
                let name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                let folder = path.parent().map(files::pretty).unwrap_or_default();
                (image.upcast(), name, folder)
            }
            Item::Power(action) => {
                let (_, glyph, label) = power_actions()
                    .into_iter()
                    .find(|(a, _, _)| a == action)
                    .unwrap_or(("", "power_settings_new", ""));
                (badge(glyph), label.to_owned(), t("sessão").to_owned())
            }
            Item::Remind(text, alarm) => {
                let q = crate::services::reminders::parse::parse(text, chrono::Local::now());
                let when = q.as_ref().map(|q| crate::services::reminders::when(&q.at)).unwrap_or_default();
                let title = q.map(|q| q.title).filter(|t| !t.is_empty());
                let kind = if *alarm { t("alarme") } else { t("lembrete") };
                (
                    badge(if *alarm { "alarm" } else { "notifications_active" }),
                    match title {
                        Some(title) => format!("{title} · {when}"),
                        None => format!("{kind} · {when}"),
                    },
                    if *alarm { t("criar alarme: toca até você parar").to_owned() } else { t("criar lembrete: avisa uma vez").to_owned() },
                )
            }
            Item::Hint if self.mode.get() == Mode::Remind => (
                badge("alarm"),
                t("quando e o quê").to_owned(),
                t("10 min chá · 18h pão · amanhã 9h dentista · alarme 6:45").to_owned(),
            ),
            Item::Hint => (
                badge("folder_open"),
                t("digite o nome de um arquivo").to_owned(),
                t("busca na sua pasta pessoal").to_owned(),
            ),
        };
        content.append(&lead);
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 1);
        texts.set_valign(gtk::Align::Center);
        texts.set_hexpand(true);
        let name = gtk::Label::new(Some(&title));
        name.add_css_class("launcher-title");
        name.set_xalign(0.0);
        name.set_ellipsize(pango::EllipsizeMode::End);
        texts.append(&name);
        if !detail.is_empty() {
            let sub = gtk::Label::new(Some(&detail));
            sub.add_css_class("launcher-detail");
            sub.set_xalign(0.0);
            sub.set_ellipsize(pango::EllipsizeMode::Middle);
            texts.append(&sub);
        }
        content.append(&texts);
        let enter = icons::icon("keyboard_return");
        enter.add_css_class("launcher-enter");
        content.append(&enter);
        row.set_child(Some(&content));
        if matches!(item, Item::Calc(_)) {
            row.add_css_class("calc");
        }
        row
    }

    fn activate(&self, index: i32) {
        let item = {
            let mut items = self.items.borrow_mut();
            let index = index.max(0) as usize;
            if index >= items.len() {
                return;
            }
            std::mem::replace(&mut items[index], Item::Hint)
        };
        match &item {
            Item::App(i) => {
                if let Some(apps) = self.apps.borrow_mut().as_mut() {
                    apps.launch(*i);
                }
                (self.close)();
            }
            Item::Calc(value) => {
                exec::spawn("wl-copy", &[&calc::format(*value)]);
                (self.close)();
            }
            Item::File(path) => {
                let uri = gio::File::for_path(path).uri();
                let context = gdk::Display::default().map(|d| d.app_launch_context());
                if let Err(err) = gio::AppInfo::launch_default_for_uri(&uri, context.as_ref()) {
                    eprintln!("octopus: cannot open {}: {err}", path.display());
                }
                (self.close)();
            }
            Item::Power(action) => {
                let action = *action;
                (self.close)();
                (self.power)(action);
            }
            Item::Remind(text, alarm) => {
                (self.remind)(text, *alarm);
                (self.close)();
            }
            Item::Hint => {}
        }
    }
}

fn badge(glyph: &str) -> gtk::Widget {
    let shape = Shape::new(Kind::Cookie6, 36, true, Some(&icons::icon(glyph)));
    shape.add_css_class("launcher-glyph");
    shape.upcast()
}
