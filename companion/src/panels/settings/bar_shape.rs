//! barra › forma: side, width, style and spacing.

use std::rc::Rc;

use gtk::prelude::*;

use super::bar_preview::Preview;
use super::rows::*;
use super::Ctx;
use crate::i18n::t;

pub fn build(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("forma"), t("a coluna em si; a prévia acompanha"));

    let preview = Preview::new(ctx, None);
    let c = ctx.clone();
    let style = choices(&[("floating", "flutuante"), ("flush", "encostada")], &ctx.get().bar.style, move |v| c.edit(|cfg| cfg.bar.style = v.into()));
    let c = ctx.clone();
    let width = slider(40.0, 72.0, 2.0, ctx.get().bar.width as f64, |v| format!("{v:.0}px"), move |v| c.edit(|cfg| cfg.bar.width = v as u32));
    let c = ctx.clone();
    let spacing = slider(0.0, 16.0, 1.0, ctx.get().bar.spacing as f64, |v| format!("{v:.0}px"), move |v| c.edit(|cfg| cfg.bar.spacing = v as u32));

    let controls = gtk::Box::new(gtk::Orientation::Vertical, 6);
    controls.set_hexpand(true);
    controls.append(&group(&[
        &stacked("rounded_corner", t("estilo"), t("encostada fica colada na borda, com cantos côncavos"), &style),
        &stacked("width", t("largura"), t("em pixels"), &width),
        &stacked("density_medium", t("espaçamento"), t("entre um módulo e outro"), &spacing),
    ]));
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
