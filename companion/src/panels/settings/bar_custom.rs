//! A module made in the editor: its name, icon, command and click.

use std::rc::Rc;

use gtk::prelude::*;

use super::rows::*;
use super::Ctx;
use crate::config::Custom;
use crate::i18n::t;
use crate::icons;

fn edit(ctx: &Rc<Ctx>, id: &str, change: impl FnOnce(&mut Custom)) {
    ctx.edit(|cfg| {
        if let Some(custom) = cfg.bar.custom.iter_mut().find(|c| c.id == id) {
            change(custom);
        }
    });
}

pub fn fields(ctx: &Rc<Ctx>, id: &str) -> Vec<gtk::Widget> {
    let custom = ctx.get().bar.custom(id).cloned().unwrap_or_default();
    let mut rows = Vec::new();

    let (c, i) = (ctx.clone(), id.to_owned());
    let name = entry(&custom.name, t("meu módulo"), move |text| edit(&c, &i, |m| m.name = text));
    rows.push(stacked("label", t("nome"), t("aparece quando o ponteiro para em cima"), &name));

    let shown = icons::icon(&custom.icon);
    shown.add_css_class("custom-icon-preview");
    let (c, i, s) = (ctx.clone(), id.to_owned(), shown.clone());
    let glyph = entry(&custom.icon, "star", move |text| {
        let name = text.trim().replace(' ', "_").to_lowercase();
        s.set_label(&name);
        edit(&c, &i, |m| m.icon = name);
    });
    glyph.set_hexpand(true);
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    line.append(&shown);
    line.append(&glyph);
    rows.push(stacked(
        "emoji_symbols",
        t("ícone"),
        t("o nome de um material symbol, tipo star, bolt, pets ou coffee"),
        &line,
    ));

    let (c, i) = (ctx.clone(), id.to_owned());
    let exec = entry(&custom.exec, t("vazio: só o ícone"), move |text| edit(&c, &i, |m| m.exec = text));
    rows.push(stacked(
        "terminal",
        t("comando"),
        t("o que ele escreve vira o valor embaixo do ícone; liga \"mostrar o valor\""),
        &exec,
    ));

    let (c, i) = (ctx.clone(), id.to_owned());
    let every = slider(1.0, 300.0, 1.0, custom.interval as f64, |v| format!("{v:.0}s"), move |v| edit(&c, &i, |m| m.interval = v as u32));
    rows.push(stacked("timer", t("a cada"), t("quanto tempo entre uma execução e outra"), &every));

    let (c, i) = (ctx.clone(), id.to_owned());
    let click = entry(&custom.click, "kitty -e btop", move |text| edit(&c, &i, |m| m.click = text));
    rows.push(stacked("touch_app", t("ao clicar"), t("um comando; vazio não faz nada"), &click));
    rows
}
