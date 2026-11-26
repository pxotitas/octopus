//! área de trabalho: its widgets, the contributions calendar and the dock.

use std::rc::Rc;

use gtk::prelude::*;

use super::rows::*;
use super::Ctx;
use crate::i18n::t;

const WIDGETS: [(&str, &str, &str, &str); 5] = [
    ("weather", "partly_cloudy_day", "clima", "agora num cookie, e os próximos dias"),
    ("contributions", "commit", "contribuições", "os quadradinhos do seu perfil no github"),
    ("clock", "schedule", "relógio", "analógico, num cookie"),
    ("media", "music_note", "música", "o que está tocando, com os controles"),
    ("calendar", "calendar_month", "calendário", "o mês inteiro"),
];

pub fn widgets(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("widgets"), t("cartões embaixo das janelas, onde nenhuma janela cobre"));
    let c = ctx.clone();
    let enabled = switch(ctx.get().desktop.enabled, move |on| c.edit(|cfg| cfg.desktop.enabled = on));
    let arrange = action("dashboard_customize", t("organizar agora"), || {
        crate::exec::spawn("octopus", &["desktop", "arrange"]);
    });
    column.append(&group(&[
        &inline("desktop_windows", t("mostrar widgets"), t("desligado, a área de trabalho fica só com o wallpaper"), &enabled),
        &inline(
            "drag_pan",
            t("organizar"),
            t("ou clique com o botão direito na área de trabalho: os widgets passam a mover e mudar de tamanho, e a maleta embaixo guarda os outros"),
            &arrange,
        ),
    ]));

    column.append(&label(t("na área de trabalho")));
    let mut items: Vec<gtk::Widget> = Vec::new();
    for (id, icon, name, about) in WIDGETS {
        let c = ctx.clone();
        let on = ctx.get().desktop.widgets.iter().any(|w| w == id || (id == "contributions" && w == "github"));
        let toggle = switch(on, move |active| {
            c.edit(|cfg| {
                let list = &mut cfg.desktop.widgets;
                list.retain(|w| w != id && !(id == "contributions" && w == "github"));
                if active {
                    list.push(id.into());
                }
            })
        });
        items.push(inline(icon, t(name), t(about), &toggle));
    }
    let refs: Vec<&gtk::Widget> = items.iter().collect();
    column.append(&group(&refs));
    column.append(&hint(t("o octopus reinicia sozinho ao aplicar mudanças aqui")));
    scroll(&column).upcast()
}

pub fn contributions(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("contribuições"), t("o calendário do seu perfil, atualizado a cada meia hora"));
    let c = ctx.clone();
    let user = entry(&ctx.get().desktop.github, t("seu usuário"), move |text| {
        c.edit(|cfg| cfg.desktop.github = text.trim().trim_start_matches('@').to_owned())
    });
    let c = ctx.clone();
    let colors = choices(&[("github", "verdes do github"), ("theme", "cor do wallpaper")], &ctx.get().desktop.contributions_colors, move |v| {
        c.edit(|cfg| cfg.desktop.contributions_colors = v.into())
    });
    column.append(&group(&[
        &stacked("person", t("usuário do github"), t("sem token: só o que o perfil público mostra"), &user),
        &stacked("palette", t("cores"), t("os quadradinhos em verde ou no primário do tema"), &colors),
    ]));
    column.append(&hint(t("o octopus reinicia sozinho ao aplicar mudanças aqui")));
    scroll(&column).upcast()
}

pub fn dock(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("dock"), t("apps fixados e abertos, embaixo da tela"));
    let c = ctx.clone();
    let enabled = switch(ctx.get().dock.enabled, move |on| c.edit(|cfg| cfg.dock.enabled = on));
    let c = ctx.clone();
    let size = slider(28.0, 64.0, 2.0, ctx.get().dock.size as f64, |v| format!("{v:.0}px"), move |v| c.edit(|cfg| cfg.dock.size = v as u32));
    let c = ctx.clone();
    let magnify = switch(ctx.get().dock.magnify, move |on| c.edit(|cfg| cfg.dock.magnify = on));
    let c = ctx.clone();
    let mode = choices(&[("fixed", "fixa"), ("autohide", "esconde sozinha"), ("smart", "inteligente")], &ctx.get().dock.mode, move |v| {
        c.edit(|cfg| cfg.dock.mode = v.into())
    });
    column.append(&group(&[
        &inline("dock_to_bottom", t("mostrar a dock"), t("reinicia o octopus ao aplicar"), &enabled),
        &stacked("photo_size_select_small", t("tamanho dos ícones"), t("em pixels"), &size),
        &inline("zoom_in", t("magnificação"), t("os ícones crescem embaixo do ponteiro"), &magnify),
        &stacked(
            "visibility",
            t("comportamento"),
            t("fixa ocupa espaço; esconde sozinha aparece na borda; inteligente sai da frente das janelas"),
            &mode,
        ),
    ]));
    column.append(&hint(t("clique direito num ícone da dock pra fixar ou desafixar")));
    scroll(&column).upcast()
}
