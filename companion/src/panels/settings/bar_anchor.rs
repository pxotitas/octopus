//! barra › ancoragem: which edge it holds to, over or under the windows,
//! and how far from the screen's edges.

use std::rc::Rc;

use gtk::prelude::*;

use super::bar_preview::Preview;
use super::rows::*;
use super::Ctx;
use crate::i18n::t;

pub fn build(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("ancoragem"), t("onde a barra se segura e quanto ela se afasta das bordas"));
    let preview = Preview::new(ctx, None);

    let c = ctx.clone();
    let side = choices(&[("left", "esquerda"), ("right", "direita")], &ctx.get().bar.position, move |v| c.edit(|cfg| cfg.bar.position = v.into()));
    let c = ctx.clone();
    let layer = choices(
        &[("bottom", "embaixo"), ("top", "em cima"), ("overlay", "por cima de tudo")],
        &ctx.get().bar.layer,
        move |v| c.edit(|cfg| cfg.bar.layer = v.into()),
    );
    let c = ctx.clone();
    let exclusive = switch(ctx.get().bar.exclusive, move |on| c.edit(|cfg| cfg.bar.exclusive = on));

    let gap = |value: u32, set: fn(&mut crate::config::Bar, u32)| {
        let c = ctx.clone();
        slider(0.0, 48.0, 1.0, value as f64, |v| format!("{v:.0}px"), move |v| c.edit(|cfg| set(&mut cfg.bar, v as u32)))
    };
    let bar = ctx.get().bar.clone();
    let top = gap(bar.margin_top, |b, v| b.margin_top = v);
    let bottom = gap(bar.margin_bottom, |b, v| b.margin_bottom = v);
    let edge = gap(bar.margin_edge, |b, v| b.margin_edge = v);

    let controls = gtk::Box::new(gtk::Orientation::Vertical, 6);
    controls.set_hexpand(true);
    controls.append(&group(&[
        &stacked("swap_horiz", t("lado"), t("de que lado da tela"), &side),
        &stacked("layers", t("camada"), t("embaixo fica sob janelas flutuantes; por cima de tudo aparece até em tela cheia"), &layer),
        &inline("space_bar", t("reservar espaço"), t("as janelas não passam por baixo dela"), &exclusive),
    ]));
    controls.append(&label(t("distância das bordas")));
    controls.append(&group(&[
        &stacked("vertical_align_top", t("em cima"), "", &top),
        &stacked("vertical_align_bottom", t("embaixo"), "", &bottom),
        &stacked("border_left", t("na lateral"), t("da borda da tela até a barra"), &edge),
    ]));
    controls.append(&hint(t("encostada ignora as distâncias: ela fica colada na borda")));

    let split = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    split.append(&preview.root);
    split.append(&controls);
    column.append(&split);
    column.append(&hint(t("a barra muda ao aplicar, sem reiniciar")));
    let keep = preview.clone();
    column.connect_destroy(move |_| {
        let _ = &keep;
    });
    scroll(&column).upcast()
}
