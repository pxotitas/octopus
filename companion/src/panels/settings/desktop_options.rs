//! What each desktop widget can change, shown under its row when expanded.

use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use super::rows::*;
use super::Ctx;
use crate::i18n::t;

/// The widget's options, or None when it has none.
pub fn options(ctx: &Rc<Ctx>, id: &str) -> Option<gtk::Widget> {
    let items: Vec<gtk::Widget> = match id {
        "weather" => weather(ctx),
        "contributions" => contributions(ctx),
        "clock" => clock(ctx),
        "agenda" => agenda(ctx),
        "system" => system(ctx),
        "battery" => battery(ctx),
        "note" => note(ctx),
        "photo" => photo(ctx),
        _ => return None,
    };
    let column = gtk::Box::new(gtk::Orientation::Vertical, 2);
    column.add_css_class("widget-options");
    for item in &items {
        column.append(item);
    }
    Some(column.upcast())
}

fn weather(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    let c = ctx.clone();
    let days = slider(0.0, 6.0, 1.0, ctx.get().desktop.weather.days as f64, |v| format!("{v:.0}"), move |v| {
        c.edit(|cfg| cfg.desktop.weather.days = v as u32)
    });
    let c = ctx.clone();
    let details = switch(ctx.get().desktop.weather.details, move |on| c.edit(|cfg| cfg.desktop.weather.details = on));
    vec![
        stacked("date_range", t("próximos dias"), t("quantos aparecem embaixo; 0 esconde"), &days),
        inline("info", t("detalhes"), t("o lugar e a sensação térmica, do lado do cookie"), &details),
    ]
}

fn contributions(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    let c = ctx.clone();
    let user = entry(&ctx.get().desktop.github, t("seu usuário"), move |text| {
        c.edit(|cfg| cfg.desktop.github = text.trim().trim_start_matches('@').to_owned())
    });
    let c = ctx.clone();
    let colors = choices(&[("github", "verdes do github"), ("theme", "cor do wallpaper")], &ctx.get().desktop.contributions_colors, move |v| {
        c.edit(|cfg| cfg.desktop.contributions_colors = v.into())
    });
    vec![
        stacked("person", t("usuário do github"), t("sem token: só o que o perfil público mostra"), &user),
        stacked("palette", t("cores"), t("os quadradinhos em verde ou no primário do tema"), &colors),
    ]
}

fn clock(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    let c = ctx.clone();
    let face = choices(&[("analog", "analógico"), ("digital", "digital")], &ctx.get().desktop.clock.face, move |v| {
        c.edit(|cfg| cfg.desktop.clock.face = v.into())
    });
    let c = ctx.clone();
    let seconds = switch(ctx.get().desktop.clock.seconds, move |on| c.edit(|cfg| cfg.desktop.clock.seconds = on));
    let c = ctx.clone();
    let hours = choices(&[("24", "24 horas"), ("12", "12 horas")], &ctx.get().desktop.clock.hours.to_string(), move |v| {
        c.edit(|cfg| cfg.desktop.clock.hours = v.parse().unwrap_or(24))
    });
    vec![
        stacked("schedule", t("mostrador"), t("ponteiros num cookie, ou horas sobre minutos"), &face),
        inline("timer", t("segundos"), t("a onda que cresce em volta, no analógico"), &seconds),
        stacked("pin", t("formato"), t("no digital"), &hours),
    ]
}

fn agenda(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    let c = ctx.clone();
    let count = slider(1.0, 15.0, 1.0, ctx.get().desktop.agenda.count as f64, |v| format!("{v:.0}"), move |v| {
        c.edit(|cfg| cfg.desktop.agenda.count = v as u32)
    });
    vec![stacked("format_list_numbered", t("quantos eventos"), t("os próximos, de todas as agendas"), &count)]
}

fn system(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    [("cpu", "memory", "processador"), ("memory", "memory_alt", "memória"), ("disk", "hard_drive", "disco")]
        .into_iter()
        .map(|(key, icon, name)| {
            let c = ctx.clone();
            let on = ctx.get().desktop.system.meters.iter().any(|m| m == key);
            let toggle = switch(on, move |active| {
                c.edit(|cfg| {
                    let meters = &mut cfg.desktop.system.meters;
                    meters.retain(|m| m != key);
                    if active {
                        // kept in the widget's order
                        let order = ["cpu", "memory", "disk"];
                        meters.push(key.into());
                        meters.sort_by_key(|m| order.iter().position(|o| o == m));
                    }
                })
            });
            inline(icon, t(name), "", &toggle)
        })
        .collect()
}

fn battery(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    let c = ctx.clone();
    let time = switch(ctx.get().desktop.battery.time, move |on| c.edit(|cfg| cfg.desktop.battery.time = on));
    vec![inline("hourglass", t("tempo restante"), t("quanto falta pra encher ou acabar"), &time)]
}

fn note(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    let c = ctx.clone();
    let size = slider(11.0, 28.0, 1.0, ctx.get().desktop.note.size as f64, |v| format!("{v:.0}px"), move |v| {
        c.edit(|cfg| cfg.desktop.note.size = v as u32)
    });
    vec![stacked("format_size", t("tamanho do texto"), "", &size)]
}

fn photo(ctx: &Rc<Ctx>) -> Vec<gtk::Widget> {
    let pick = action("add_photo_alternate", t("escolher"), || {});
    pick.connect_clicked(|button| {
        let filter = gtk::FileFilter::new();
        filter.add_mime_type("image/*");
        let filters = gio::ListStore::new::<gtk::FileFilter>();
        filters.append(&filter);
        let dialog = gtk::FileDialog::builder().title(t("trocar a foto")).filters(&filters).build();
        let start = crate::desktop::photo::chosen()
            .and_then(|f| f.parent().map(|d| d.to_path_buf()))
            .or_else(|| glib::user_special_dir(glib::UserDirectory::Pictures));
        if let Some(dir) = start {
            dialog.set_initial_folder(Some(&gio::File::for_path(dir)));
        }
        let window = button.root().and_downcast::<gtk::Window>();
        dialog.open(window.as_ref(), gio::Cancellable::NONE, |result| {
            if let Some(file) = result.ok().and_then(|f| f.path()) {
                crate::desktop::photo::choose(&file);
            }
        });
    });
    let c = ctx.clone();
    let fit = choices(&[("cover", "preencher"), ("contain", "inteira")], &ctx.get().desktop.photo.fit, move |v| {
        c.edit(|cfg| cfg.desktop.photo.fit = v.into())
    });
    let c = ctx.clone();
    let shuffle = choices(
        &[("0", "uma só"), ("5", "5 min"), ("15", "15 min"), ("60", "1 hora")],
        &ctx.get().desktop.photo.shuffle.to_string(),
        move |v| c.edit(|cfg| cfg.desktop.photo.shuffle = v.parse().unwrap_or(0)),
    );
    vec![
        inline("image", t("a foto"), t("troca na hora, sem aplicar"), &pick),
        stacked("crop", t("encaixe"), t("recortada pra preencher o card, ou inteira"), &fit),
        stacked("shuffle", t("trocar sozinha"), t("passa pelas fotos da mesma pasta"), &shuffle),
    ]
}
