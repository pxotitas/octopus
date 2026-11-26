//! painéis: the dash, notifications, weather and music.

use std::rc::Rc;

use gtk::prelude::*;

use super::rows::*;
use super::Ctx;
use crate::i18n::t;

const RESTART: &str = "vale depois que o octopus reinicia, sozinho ao aplicar";

pub fn dash(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("dash"), t("o painel que abre no relógio da barra"));
    let c = ctx.clone();
    let tab = choices(
        &[("overview", "visão geral"), ("media", "mídia"), ("weather", "clima"), ("wallpapers", "wallpapers")],
        &ctx.get().dash.page,
        move |v| c.edit(|cfg| cfg.dash.page = v.into()),
    );
    let c = ctx.clone();
    let lyrics = switch(ctx.get().media.lyrics, move |on| c.edit(|cfg| cfg.media.lyrics = on));
    column.append(&group(&[
        &stacked("space_dashboard", t("abre em"), t("a aba que aparece primeiro"), &tab),
        &inline("lyrics", t("letras no dash"), t("a letra da música, linha por linha"), &lyrics),
    ]));
    column.append(&hint(t(RESTART)));
    scroll(&column).upcast()
}

pub fn notifications(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("notificações"), t("os cartões que aparecem quando chega alguma coisa"));
    let c = ctx.clone();
    let seconds = slider(
        2.0,
        30.0,
        1.0,
        (ctx.get().notifications.timeout_ms / 1000) as f64,
        |v| format!("{v:.0}s"),
        move |v| c.edit(|cfg| cfg.notifications.timeout_ms = v as u32 * 1000),
    );
    let c = ctx.clone();
    let count = slider(
        1.0,
        6.0,
        1.0,
        ctx.get().notifications.max_popups as f64,
        |v| format!("{v:.0}"),
        move |v| c.edit(|cfg| cfg.notifications.max_popups = v as usize),
    );
    let c = ctx.clone();
    let corner = choices(
        &[("top-left", "em cima, à esquerda"), ("top-right", "em cima, à direita"), ("bottom-left", "embaixo, à esquerda"), ("bottom-right", "embaixo, à direita")],
        &ctx.get().notifications.position,
        move |v| c.edit(|cfg| cfg.notifications.position = v.into()),
    );
    column.append(&group(&[
        &stacked("timer", t("tempo na tela"), t("quanto cada cartão fica antes de ir pra central"), &seconds),
        &stacked("stacks", t("de uma vez"), t("quantos cartões cabem na pilha"), &count),
        &stacked("picture_in_picture", t("canto"), t("onde a pilha aparece"), &corner),
    ]));
    column.append(&hint(t(RESTART)));
    scroll(&column).upcast()
}

pub fn weather(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("clima"), t("do open-meteo, sem conta nem chave"));
    let c = ctx.clone();
    let place = entry(&ctx.get().weather.place, t("pelo ip"), move |text| c.edit(|cfg| cfg.weather.place = text));
    let c = ctx.clone();
    let unit = choices(&[("celsius", "celsius"), ("fahrenheit", "fahrenheit")], &ctx.get().weather.unit, move |v| {
        c.edit(|cfg| cfg.weather.unit = v.into())
    });
    column.append(&group(&[
        &stacked("location_on", t("cidade"), t("um nome ou \"lat,lon\"; vazio usa o ip, que é aproximado"), &place),
        &stacked("thermometer", t("unidade"), t("no dash, nos widgets e na tela de bloqueio"), &unit),
    ]));
    column.append(&hint(t(RESTART)));
    scroll(&column).upcast()
}

pub fn music(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("música"), t("as letras vêm do lrclib, sincronizadas quando dá"));
    let c = ctx.clone();
    let offset = slider(
        -1.5,
        1.5,
        0.05,
        ctx.get().media.lyrics_offset_ms as f64 / 1000.0,
        |v| format!("{v:+.2}s"),
        move |v| {
            let ms = (v * 1000.0).round() as i64;
            crate::services::lyrics::set_offset(ms);
            c.edit(|cfg| cfg.media.lyrics_offset_ms = ms);
        },
    );
    column.append(&group(&[&stacked(
        "sync",
        t("tempo da letra"),
        t("adiantar (+) ou atrasar (-) as letras, se não baterem com a música; muda na hora"),
        &offset,
    )]));
    scroll(&column).upcast()
}
