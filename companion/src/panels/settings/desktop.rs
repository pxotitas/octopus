//! área de trabalho: its widgets, how the cards look, and the dock.

use std::rc::Rc;

use gtk::prelude::*;

use super::desktop_options;
use super::rows::*;
use super::Ctx;
use crate::i18n::t;

const WIDGETS: [(&str, &str, &str, &str); 10] = [
    ("weather", "partly_cloudy_day", "clima", "agora num cookie, e os próximos dias"),
    ("contributions", "commit", "contribuições", "os quadradinhos do seu perfil no github"),
    ("clock", "schedule", "relógio", "analógico ou digital, num cookie"),
    ("media", "music_note", "música", "o que está tocando, com os controles"),
    ("calendar", "calendar_month", "calendário", "o mês, com os seus eventos"),
    ("agenda", "event_upcoming", "agenda", "os próximos eventos do seu calendário"),
    ("system", "monitor_heart", "sistema", "processador, memória e disco"),
    ("battery", "battery_charging_full", "bateria", "quanto tem, e quanto dura"),
    ("note", "sticky_note_2", "nota", "um bilhete que se guarda sozinho"),
    ("photo", "photo", "foto", "uma imagem sua, recortada no card"),
];

pub fn widgets(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("widgets"), t("cartões embaixo das janelas, onde nenhuma janela cobre"));
    let c = ctx.clone();
    let enabled = switch(ctx.get().desktop.enabled, move |on| c.edit(|cfg| cfg.desktop.enabled = on));
    let arrange = action("dashboard_customize", t("organizar agora"), || {
        crate::exec::spawn("octopus", &["desktop", "arrange"]);
    });
    let c = ctx.clone();
    let aside = switch(ctx.get().desktop.hide_under_windows, move |on| c.edit(|cfg| cfg.desktop.hide_under_windows = on));
    column.append(&group(&[
        &inline("desktop_windows", t("mostrar widgets"), t("desligado, a área de trabalho fica só com o wallpaper"), &enabled),
        &inline(
            "visibility_off",
            t("sair da frente das janelas"),
            t("cada widget some enquanto uma janela está em cima dele, e para de atualizar até voltar"),
            &aside,
        ),
        &inline(
            "drag_pan",
            t("organizar"),
            t("ou clique com o botão direito na área de trabalho: os widgets passam a mover e mudar de tamanho, e a maleta embaixo guarda os outros"),
            &arrange,
        ),
    ]));

    column.append(&label(t("os cards")));
    let c = ctx.clone();
    let card = choices(
        &[("surface", "superfície"), ("tonal", "tonal"), ("primary", "primária"), ("none", "sem fundo")],
        &ctx.get().desktop.card,
        move |v| c.edit(|cfg| cfg.desktop.card = v.into()),
    );
    let c = ctx.clone();
    let corners = slider(0.0, 48.0, 2.0, ctx.get().desktop.corners as f64, |v| format!("{v:.0}px"), move |v| {
        c.edit(|cfg| cfg.desktop.corners = v as u32)
    });
    let c = ctx.clone();
    let snap = choices(
        &[("0", "solto"), ("8", "8px"), ("16", "16px"), ("32", "32px")],
        &ctx.get().desktop.snap.to_string(),
        move |v| c.edit(|cfg| cfg.desktop.snap = v.parse().unwrap_or(0)),
    );
    column.append(&group(&[
        &stacked("format_color_fill", t("cor"), t("o fundo de todos os widgets, nas cores do wallpaper"), &card),
        &stacked("rounded_corner", t("cantos"), t("o arredondado dos cards"), &corners),
        &stacked("grid_4x4", t("grade"), t("ao organizar, os widgets encaixam numa grade"), &snap),
    ]));

    column.append(&label(t("na área de trabalho")));
    let mut items: Vec<gtk::Widget> = Vec::new();
    for (id, icon, name, about) in WIDGETS {
        items.push(widget_row(ctx, id, icon, name, about));
    }
    let refs: Vec<&gtk::Widget> = items.iter().collect();
    column.append(&group(&refs));
    column.append(&hint(t("o octopus reinicia sozinho ao aplicar mudanças aqui")));
    scroll(&column).upcast()
}

/// A widget's switch, and its options under a tune button.
fn widget_row(ctx: &Rc<Ctx>, id: &'static str, icon: &str, name: &'static str, about: &'static str) -> gtk::Widget {
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
    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let Some(options) = desktop_options::options(ctx, id) else {
        controls.append(&toggle);
        return inline(icon, t(name), t(about), &controls);
    };
    let revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::SlideDown)
        .transition_duration(220)
        .child(&options)
        .build();
    let tune = gtk::ToggleButton::new();
    tune.set_child(Some(&crate::icons::icon("tune")));
    tune.add_css_class("widget-tune");
    tune.set_tooltip_text(Some(t("opções")));
    let r = revealer.clone();
    tune.connect_toggled(move |b| r.set_reveal_child(b.is_active()));
    controls.append(&tune);
    controls.append(&toggle);
    let item = gtk::Box::new(gtk::Orientation::Vertical, 0);
    item.add_css_class("widget-item");
    item.append(&inline(icon, t(name), t(about), &controls));
    item.append(&revealer);
    item.upcast()
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
