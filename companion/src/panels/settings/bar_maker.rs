//! Making a module of your own, or changing one: a dialog with the module
//! drawn live on a piece of the bar, and the choices in order: its name and
//! icon, what it shows, what a click does, how it looks.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::glib;

use super::bar_preview::{module, place};
use super::rows::*;
use super::{bar_pickers, Ctx};
use crate::config::{Custom, Module};
use crate::i18n::t;
use crate::icons;

const ICONS: [&str; 60] = [
    "star", "bolt", "pets", "favorite", "auto_awesome", "rocket_launch", "spa", "eco", "local_cafe", "coffee",
    "terminal", "code", "bug_report", "deployed_code", "extension", "keyboard", "mouse", "monitor", "devices", "screenshot_monitor",
    "music_note", "headphones", "mic", "videocam", "photo_camera", "palette", "brush", "edit", "book", "translate",
    "cloud", "wb_sunny", "dark_mode", "thermostat", "water_drop", "air", "public", "language", "home", "folder",
    "download", "upload", "sync", "wifi", "bluetooth", "battery_full", "memory", "storage", "speed", "timer",
    "schedule", "alarm", "calendar_month", "mail", "chat", "notifications", "lock", "key", "sports_esports", "fitness_center",
];

struct Maker {
    ctx: Rc<Ctx>,
    /// The module being made, and its look.
    custom: RefCell<Custom>,
    look: RefCell<Module>,
    preview: gtk::Box,
    shows_value: RefCell<bool>,
    clicks: RefCell<bool>,
}

impl Maker {
    fn redraw(&self) {
        crate::widgets::clear(&self.preview);
        let mut bar = self.ctx.get().bar.clone();
        let custom = self.custom.borrow().clone();
        bar.custom.retain(|c| c.id != custom.id);
        bar.custom.push(custom.clone());
        let mut look = self.look.borrow().clone();
        look.text = *self.shows_value.borrow();
        let mut fake = custom.clone();
        if look.text && fake.exec.trim().is_empty() {
            fake.exec = "…".into();
        }
        bar.custom.retain(|c| c.id != fake.id);
        bar.custom.push(fake);
        for (glyph, sample) in [("schedule", false), ("", true), ("battery_full", false)] {
            if sample {
                self.preview.append(&module(&custom.id, &look, &bar));
            } else {
                let other = icons::icon(glyph);
                other.add_css_class("pm-icon");
                other.add_css_class("maker-neighbor");
                self.preview.append(&other);
            }
        }
    }
}

fn section(title: &str, number: usize) -> gtk::Box {
    let head = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    head.add_css_class("maker-section");
    let n = gtk::Label::new(Some(&number.to_string()));
    n.add_css_class("step-number");
    head.append(&n);
    let label = gtk::Label::new(Some(title));
    label.add_css_class("maker-section-title");
    label.set_xalign(0.0);
    head.append(&label);
    head
}

/// Opens the dialog. With `id`, it changes that module; without, it makes a
/// new one in the zone `zone` (0 top, 1 middle, 2 bottom). `done` hears the
/// module's id.
pub fn open(parent: &gtk::Window, ctx: &Rc<Ctx>, id: Option<&str>, zone: usize, done: impl Fn(&str) + 'static) {
    let editing = id.is_some();
    let custom = match id.and_then(|id| ctx.get().bar.custom(id).cloned()) {
        Some(c) => c,
        None => Custom { id: ctx.get().bar.next_custom_id(), name: t("meu módulo").into(), ..Custom::default() },
    };
    let look = match id {
        Some(id) => ctx.get().bar.module(id),
        None => Module { background: "tonal".into(), shape: "soft".into(), ..Module::default() },
    };
    let shows_value = !custom.exec.trim().is_empty() && look.text;
    let clicks = !custom.click.trim().is_empty();

    let window = gtk::Window::builder()
        .title(if editing { t("editar módulo") } else { t("criar um módulo") })
        .transient_for(parent)
        .modal(true)
        .default_width(760)
        .default_height(640)
        .build();
    window.add_css_class("octopus-settings");
    window.add_css_class("maker");

    let preview = gtk::Box::new(gtk::Orientation::Vertical, 8);
    preview.add_css_class("bar-preview");
    preview.add_css_class("floating");
    preview.add_css_class("maker-bar");
    preview.set_size_request(ctx.get().bar.width as i32, -1);
    preview.set_valign(gtk::Align::Center);
    preview.set_halign(gtk::Align::Center);

    let maker = Rc::new(Maker {
        ctx: ctx.clone(),
        custom: RefCell::new(custom.clone()),
        look: RefCell::new(look.clone()),
        preview: preview.clone(),
        shows_value: RefCell::new(shows_value),
        clicks: RefCell::new(clicks),
    });

    let form = gtk::Box::new(gtk::Orientation::Vertical, 10);
    form.add_css_class("maker-form");

    // 1. name and icon
    form.append(&section(t("nome e ícone"), 1));
    let m = maker.clone();
    let name = entry(&custom.name, t("meu módulo"), move |text| {
        m.custom.borrow_mut().name = text;
    });
    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some(t("procurar, ou escrever o nome de um material symbol")));
    let grid = gtk::FlowBox::new();
    grid.add_css_class("picker");
    grid.add_css_class("icon-grid");
    grid.set_selection_mode(gtk::SelectionMode::None);
    grid.set_homogeneous(true);
    grid.set_max_children_per_line(10);
    grid.set_min_children_per_line(6);
    grid.set_row_spacing(4);
    grid.set_column_spacing(4);
    let mut group: Option<gtk::ToggleButton> = None;
    let mut chips = Vec::new();
    for icon in ICONS {
        let chip = gtk::ToggleButton::new();
        chip.add_css_class("picker-chip");
        chip.set_child(Some(&icons::icon(icon)));
        chip.set_tooltip_text(Some(icon));
        chip.set_group(group.as_ref());
        chip.set_active(custom.icon == icon);
        group.get_or_insert(chip.clone());
        let m = maker.clone();
        chip.connect_toggled(move |b| {
            if b.is_active() {
                m.custom.borrow_mut().icon = icon.into();
                m.redraw();
            }
        });
        grid.insert(&chip, -1);
        chips.push((icon, chip));
    }
    let chips = Rc::new(chips);
    {
        let (m, chips) = (maker.clone(), chips.clone());
        search.connect_search_changed(move |s| {
            let text = s.text().trim().to_lowercase().replace(' ', "_");
            for (icon, chip) in chips.iter() {
                if let Some(child) = chip.parent() {
                    child.set_visible(text.is_empty() || icon.contains(&text));
                }
            }
            // a name that isn't in the grid is taken as it is
            if !text.is_empty() && !chips.iter().any(|(i, _)| *i == text) {
                m.custom.borrow_mut().icon = text;
                m.redraw();
            }
        });
    }
    let icon_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
    icon_box.append(&search);
    icon_box.append(&grid);
    form.append(&group_of(&[
        &stacked("label", t("nome"), t("aparece quando o ponteiro para em cima"), &name),
        &stacked("emoji_symbols", t("ícone"), "", &icon_box),
    ]));

    // 2. what it shows
    form.append(&section(t("o que ele mostra"), 2));
    let value_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    let m = maker.clone();
    let exec = entry(&custom.exec, "uptime -p | cut -d' ' -f2-", move |text| {
        m.custom.borrow_mut().exec = text;
    });
    exec.set_hexpand(true);
    let output = gtk::Label::new(None);
    output.add_css_class("maker-output");
    output.set_xalign(0.0);
    output.set_wrap(true);
    let test = action("play_arrow", t("testar"), {
        let (exec, output) = (exec.clone(), output.clone());
        move || {
            let command = exec.text().to_string();
            let output = output.clone();
            output.set_label(t("rodando…"));
            glib::spawn_future_local(async move {
                let answer = crate::exec::run("sh", &["-c".to_owned(), command]).await;
                output.set_label(&match answer {
                    Ok(text) if text.trim().is_empty() => t("(não escreveu nada)").to_owned(),
                    Ok(text) => format!("→ {}", text.lines().next().unwrap_or_default().trim()),
                    Err(err) => format!("✕ {err}"),
                });
            });
        }
    });
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    line.append(&exec);
    line.append(&test);
    value_box.append(&line);
    value_box.append(&output);
    let m = maker.clone();
    let every = slider(1.0, 300.0, 1.0, custom.interval as f64, |v| format!("{v:.0}s"), move |v| m.custom.borrow_mut().interval = v as u32);
    value_box.append(&every);
    let value_reveal = gtk::Revealer::builder().child(&value_box).reveal_child(shows_value).build();
    let (m, r) = (maker.clone(), value_reveal.clone());
    let shows = choices(&[("icon", "só o ícone"), ("value", "ícone e um valor")], if shows_value { "value" } else { "icon" }, move |v| {
        *m.shows_value.borrow_mut() = v == "value";
        r.set_reveal_child(v == "value");
        m.redraw();
    });
    let shows_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    shows_box.append(&shows);
    shows_box.append(&value_reveal);
    form.append(&group_of(&[&stacked(
        "pin",
        t("mostrar"),
        t("o valor é o que um comando escreve, embaixo do ícone"),
        &shows_box,
    )]));

    // 3. the click
    form.append(&section(t("ao clicar"), 3));
    let m = maker.clone();
    let click = entry(&custom.click, "kitty -e btop", move |text| m.custom.borrow_mut().click = text);
    let click_reveal = gtk::Revealer::builder().child(&click).reveal_child(clicks).build();
    let (m, r) = (maker.clone(), click_reveal.clone());
    let does = choices(&[("nothing", "nada"), ("command", "roda um comando")], if clicks { "command" } else { "nothing" }, move |v| {
        *m.clicks.borrow_mut() = v == "command";
        r.set_reveal_child(v == "command");
    });
    let click_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
    click_box.append(&does);
    click_box.append(&click_reveal);
    form.append(&group_of(&[&stacked("touch_app", t("um clique"), "", &click_box)]));

    // 4. the look
    form.append(&section(t("visual"), 4));
    let looks_holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    fn fill_looks(maker: &Rc<Maker>, holder: &gtk::Box) {
        crate::widgets::clear(holder);
        let bar = maker.ctx.get().bar.clone();
        let mut with = bar.clone();
        with.custom.retain(|c| c.id != maker.custom.borrow().id);
        with.custom.push(maker.custom.borrow().clone());
        let look = maker.look.borrow().clone();
        let (m, h) = (maker.clone(), holder.clone());
        let picker = bar_pickers::looks(&maker.custom.borrow().id.clone(), &look, &with, move |background, shape| {
            {
                let mut l = m.look.borrow_mut();
                l.background = background.into();
                l.shape = shape.into();
            }
            m.redraw();
            let (m, h) = (m.clone(), h.clone());
            glib::idle_add_local_once(move || fill_looks(&m, &h));
        });
        holder.append(&picker);
    }
    fill_looks(&maker, &looks_holder);
    form.append(&group_of(&[&stacked("auto_awesome", t("presets"), t("fundo e forma num toque; dá pra ajustar depois no inspetor"), &looks_holder)]));

    // the zone, for a new one
    let zone_now = Rc::new(RefCell::new(zone));
    if !editing {
        let z = zone_now.clone();
        let picker = choices(&[("0", "topo"), ("1", "meio"), ("2", "base")], &zone.to_string(), move |v| {
            *z.borrow_mut() = v.parse().unwrap_or(2);
        });
        form.append(&group_of(&[&stacked("vertical_align_center", t("pôr em"), "", &picker)]));
    }

    let side = gtk::Box::new(gtk::Orientation::Vertical, 12);
    side.add_css_class("maker-side");
    let caption = gtk::Label::new(Some(t("assim ele fica na barra")));
    caption.add_css_class("settings-item-sub");
    side.append(&preview);
    side.append(&caption);
    side.set_valign(gtk::Align::Center);

    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.append(&side);
    let form_scroll = scroll(&form);
    form_scroll.set_hexpand(true);
    form_scroll.set_vexpand(true);
    body.append(&form_scroll);

    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    footer.add_css_class("maker-footer");
    footer.set_halign(gtk::Align::End);
    let cancel = gtk::Button::with_label(t("cancelar"));
    cancel.add_css_class("text-button");
    let w = window.clone();
    cancel.connect_clicked(move |_| w.close());
    let confirm = gtk::Button::new();
    confirm.add_css_class("settings-fab");
    let inside = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    inside.append(&icons::icon(if editing { "check" } else { "add" }));
    inside.append(&gtk::Label::new(Some(if editing { t("salvar") } else { t("criar") })));
    confirm.set_child(Some(&inside));
    let (m, w, z) = (maker.clone(), window.clone(), zone_now.clone());
    let done = Rc::new(done);
    confirm.connect_clicked(move |_| {
        let mut custom = m.custom.borrow().clone();
        if !*m.clicks.borrow() {
            custom.click.clear();
        }
        let mut look = m.look.borrow().clone();
        look.text = *m.shows_value.borrow() && !custom.exec.trim().is_empty();
        if custom.name.trim().is_empty() {
            custom.name = t("meu módulo").into();
        }
        let id = custom.id.clone();
        let zone = *z.borrow();
        m.ctx.edit(|cfg| {
            match cfg.bar.custom.iter_mut().find(|c| c.id == id) {
                Some(existing) => *existing = custom.clone(),
                None => {
                    cfg.bar.custom.push(custom.clone());
                    place(&mut cfg.bar, &id, Some((zone, usize::MAX)));
                }
            }
            cfg.bar.modules.insert(id.clone(), look.clone());
        });
        done(&id);
        w.close();
    });
    footer.append(&cancel);
    footer.append(&confirm);

    let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root.append(&body);
    root.append(&footer);
    window.set_child(Some(&root));
    maker.redraw();
    window.present();
}

fn group_of(items: &[&gtk::Widget]) -> gtk::Box {
    group(items)
}
