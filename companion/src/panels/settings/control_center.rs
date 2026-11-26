//! painéis › central de controle: its parts and their order, which buttons
//! and sliders, and what the header shows.

use std::rc::Rc;

use gtk::prelude::*;

use super::ordered;
use super::rows::*;
use super::Ctx;
use crate::i18n::t;

const SECTIONS: &[(&str, &str, &str, &str)] = &[
    ("header", "schedule", "cabeçalho", "a hora, a data, o clima e os botões"),
    ("tiles", "toggle_on", "botões rápidos", "wi-fi, bluetooth, cafeína…"),
    ("sliders", "tune", "controles deslizantes", "volume, microfone, brilho"),
    ("media", "music_note", "player", "o que está tocando"),
    ("notifications", "notifications", "notificações", "a lista e o não perturbe"),
];

const TILES: &[(&str, &str, &str, &str)] = &[
    ("wifi", "wifi", "wi-fi", "liga e desliga; a seta abre as redes"),
    ("bluetooth", "bluetooth", "bluetooth", "liga e desliga; a seta abre os aparelhos"),
    ("mic", "mic", "microfone", "muta; a seta abre o som"),
    ("theme", "dark_mode", "tema", "claro, escuro ou automático"),
    ("caffeine", "coffee", "cafeína", "a tela não apaga nem trava enquanto estiver ligada"),
    ("dnd", "notifications_off", "não perturbe", "as notificações esperam na lista"),
];

const SLIDERS: &[(&str, &str, &str, &str)] = &[
    ("volume", "volume_up", "volume", ""),
    ("mic", "mic", "microfone", ""),
    ("brightness", "brightness_6", "brilho da tela", "some se a tela não deixar"),
    ("keyboard", "keyboard", "luz do teclado", "some se o teclado não tiver"),
];

pub fn build(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("central de controle"), t("o que aparece, em que ordem, e quais botões"));

    column.append(&label(t("partes")));
    column.append(&ordered::group(ctx, SECTIONS, |c| &mut c.control_center.sections));

    column.append(&label(t("cabeçalho")));
    let switches: [(&str, &str, &str, fn(&mut crate::config::Config) -> &mut bool); 4] = [
        ("calendar_today", "data", "embaixo da hora", |c| &mut c.control_center.date),
        ("partly_cloudy_day", "clima", "a temperatura e o céu agora", |c| &mut c.control_center.weather),
        ("settings", "engrenagem", "abre as configurações", |c| &mut c.control_center.settings),
        ("power_settings_new", "energia", "o menu de desligar, reiniciar e sair", |c| &mut c.control_center.power),
    ];
    let mut items: Vec<gtk::Widget> = Vec::new();
    for (icon, name, about, pick) in switches {
        let c = ctx.clone();
        let on = *pick(&mut ctx.get().clone());
        let toggle = switch(on, move |v| c.edit(|cfg| *pick(cfg) = v));
        items.push(inline(icon, t(name), t(about), &toggle));
    }
    let refs: Vec<&gtk::Widget> = items.iter().collect();
    column.append(&group(&refs));

    column.append(&label(t("botões rápidos")));
    column.append(&ordered::group(ctx, TILES, |c| &mut c.control_center.tiles));

    column.append(&label(t("controles deslizantes")));
    column.append(&ordered::group(ctx, SLIDERS, |c| &mut c.control_center.sliders));

    column.append(&label(t("player")));
    let c = ctx.clone();
    let visualizer = switch(ctx.get().control_center.visualizer, move |v| c.edit(|cfg| cfg.control_center.visualizer = v));
    column.append(&group(&[&inline("graphic_eq", t("visualizador"), t("as barrinhas que dançam com a música"), &visualizer)]));

    column.append(&hint(t("o octopus reinicia sozinho ao aplicar mudanças aqui")));
    scroll(&column).upcast()
}
