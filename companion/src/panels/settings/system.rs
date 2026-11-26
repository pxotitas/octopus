//! sistema: the lock screen, and about octopus.

use std::rc::Rc;

use gtk::prelude::*;

use super::rows::*;
use super::Ctx;
use crate::i18n::t;

pub fn lock(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("tela de bloqueio"), t("shapes que giram a cada tecla, a hora e a música"));
    let c = ctx.clone();
    let lyrics = switch(ctx.get().lock.lyrics, move |on| c.edit(|cfg| cfg.lock.lyrics = on));
    let test = action("lock", t("travar agora"), || crate::exec::spawn("octopus", &["lock"]));
    column.append(&group(&[
        &inline("lyrics", t("letra da música"), t("a linha sendo cantada, embaixo da hora"), &lyrics),
        &inline("lock_open", t("testar"), t("trava a tela; sua senha destrava"), &test),
    ]));
    column.append(&hint(t("vale depois que o octopus reinicia, sozinho ao aplicar")));
    scroll(&column).upcast()
}

pub fn about(_ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("sobre"), t("um shell m3 cozy pro niri"));

    let hero = gtk::Box::new(gtk::Orientation::Horizontal, 18);
    hero.add_css_class("settings-about");
    let logo = super::bar_preview::glyph_widget("octopus", 80);
    logo.set_valign(gtk::Align::Center);
    hero.append(&logo);
    let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
    texts.set_valign(gtk::Align::Center);
    let name = gtk::Label::new(Some("octopus"));
    name.add_css_class("settings-title");
    name.set_xalign(0.0);
    let version = gtk::Label::new(Some(&format!("{} {}", t("versão"), env!("CARGO_PKG_VERSION"))));
    version.add_css_class("settings-subtitle");
    version.set_xalign(0.0);
    texts.append(&name);
    texts.append(&version);
    hero.append(&texts);
    column.append(&hero);

    let open = |url: &'static str| move || crate::exec::spawn("xdg-open", &[url]);
    let site = action("public", t("abrir"), open("https://pxotitas.github.io/octopus/"));
    let repo = action("code", t("abrir"), open("https://github.com/pxotitas/octopus"));
    let docs = action("menu_book", t("abrir"), open("https://pxotitas.github.io/octopus/docs/"));
    column.append(&group(&[
        &inline("public", t("site"), "pxotitas.github.io/octopus", &site),
        &inline("menu_book", t("docs"), t("atalhos, configuração e tema"), &docs),
        &inline("code", "github", "pxotitas/octopus", &repo),
    ]));

    let config = crate::exec::config_dir().join("octopus.toml");
    let path = config.to_string_lossy().into_owned();
    let file = action("edit_document", t("abrir"), move || crate::exec::spawn("xdg-open", &[&path]));
    let restart = action("restart_alt", t("reiniciar"), || crate::exec::spawn("octopus", &["restart"]));
    column.append(&group(&[
        &inline("description", "octopus.toml", t("tudo o que essa janela muda, em texto"), &file),
        &inline("restart_alt", t("reiniciar o octopus"), t("relê tudo do começo"), &restart),
    ]));
    scroll(&column).upcast()
}
