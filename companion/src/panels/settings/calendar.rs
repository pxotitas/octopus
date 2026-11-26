//! painéis › calendário: iCal addresses, Google's secret one included,
//! for the dash's calendar and the agenda widget.

use std::rc::Rc;

use gtk::prelude::*;

use super::rows::*;
use super::Ctx;
use crate::i18n::t;
use crate::icons;

const STEPS: [&str; 4] = [
    "abra calendar.google.com no computador",
    "na engrenagem, vá em configurações e, na esquerda, clique no seu calendário",
    "em \"integrar agenda\", copie o \"endereço secreto no formato iCal\"",
    "cole no campo de endereço e aplique",
];

fn list(ctx: &Rc<Ctx>, holder: &gtk::Box) {
    crate::widgets::clear(holder);
    let urls = ctx.get().calendar.urls.clone();
    let mut rows: Vec<gtk::Widget> = Vec::new();
    for (index, url) in urls.iter().enumerate() {
        let line = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        line.add_css_class("settings-item");
        let glyph = icons::icon("event");
        glyph.add_css_class("settings-icon");
        line.append(&glyph);
        let shown = if url.contains("calendar.google.com") { t("google agenda") } else { t("calendário ical") };
        let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
        texts.set_hexpand(true);
        let name = gtk::Label::new(Some(shown));
        name.add_css_class("settings-item-title");
        name.set_xalign(0.0);
        // the secret part stays hidden
        let hidden: String = url.chars().take(36).collect::<String>() + "…";
        let sub = gtk::Label::new(Some(&hidden));
        sub.add_css_class("settings-item-sub");
        sub.set_xalign(0.0);
        texts.append(&name);
        texts.append(&sub);
        line.append(&texts);
        let remove = icons::button("delete", t("tirar"));
        let (c, h) = (ctx.clone(), holder.clone());
        remove.connect_clicked(move |_| {
            c.edit(|cfg| {
                if index < cfg.calendar.urls.len() {
                    cfg.calendar.urls.remove(index);
                }
            });
            list(&c, &h);
        });
        line.append(&remove);
        rows.push(line.upcast());
    }
    if !rows.is_empty() {
        let refs: Vec<&gtk::Widget> = rows.iter().collect();
        holder.append(&group(&refs));
    }
}

pub fn build(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("calendário"), t("os seus eventos no calendário do dash e no widget de agenda, sem login nem chave"));

    let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    list(ctx, &holder);
    column.append(&holder);

    let entry = gtk::Entry::new();
    entry.set_placeholder_text(Some("https://calendar.google.com/calendar/ical/…/basic.ics"));
    entry.set_hexpand(true);
    let add = action("add", t("adicionar"), {
        let (c, h, e) = (ctx.clone(), holder.clone(), entry.clone());
        move || {
            let url = e.text().trim().to_owned();
            if !(url.starts_with("http") || url.starts_with("webcal")) {
                e.add_css_class("error");
                return;
            }
            e.remove_css_class("error");
            c.edit(|cfg| {
                if !cfg.calendar.urls.contains(&url) {
                    cfg.calendar.urls.push(url.clone());
                }
            });
            e.set_text("");
            list(&c, &h);
        }
    });
    {
        let add = add.clone();
        entry.connect_activate(move |_| add.emit_clicked());
    }
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    line.append(&entry);
    line.append(&add);
    column.append(&group(&[&stacked("link", t("endereço ical"), t("de um google agenda, outlook, proton, nextcloud…"), &line)]));

    column.append(&label(t("como pegar o do google")));
    let steps = gtk::Box::new(gtk::Orientation::Vertical, 8);
    steps.add_css_class("settings-item");
    for (i, step) in STEPS.iter().enumerate() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let number = gtk::Label::new(Some(&(i + 1).to_string()));
        number.add_css_class("step-number");
        number.set_valign(gtk::Align::Start);
        row.append(&number);
        let text = gtk::Label::new(Some(t(step)));
        text.set_wrap(true);
        text.set_xalign(0.0);
        row.append(&text);
        steps.append(&row);
    }
    let open = action("open_in_new", t("abrir o google agenda"), || {
        crate::exec::spawn("xdg-open", &["https://calendar.google.com/calendar/r/settings"]);
    });
    open.set_halign(gtk::Align::Start);
    steps.append(&open);
    column.append(&group(&[steps.upcast_ref()]));
    column.append(&hint(t("o endereço secreto dá acesso de leitura a esse calendário: não compartilhe. ele é só lido, nada é escrito no google. atualiza a cada 15 minutos.")));

    let errors = ctx.services.calendar.errors.get().clone();
    for error in errors {
        let line = hint(&error);
        line.add_css_class("error");
        column.append(&line);
    }
    column.append(&hint(t("o octopus reinicia sozinho ao aplicar mudanças aqui")));
    scroll(&column).upcast()
}
