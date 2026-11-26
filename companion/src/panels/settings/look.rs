//! aparência: the palette and the language.

use std::rc::Rc;

use gtk::prelude::*;

use super::rows::*;
use super::Ctx;
use crate::i18n::t;

const SCHEMES: [(&str, &str); 9] = [
    ("tonal-spot", "tonal"),
    ("expressive", "expressivo"),
    ("vibrant", "vibrante"),
    ("fidelity", "fiel"),
    ("content", "conteúdo"),
    ("neutral", "neutro"),
    ("monochrome", "monocromático"),
    ("rainbow", "arco-íris"),
    ("fruit-salad", "salada de frutas"),
];

pub fn colors(ctx: &Rc<Ctx>) -> gtk::Widget {
    use crate::services::theme::Mode;
    let column = page(t("cores"), t("tudo sai do wallpaper; aqui você escolhe como"));

    let current = match *ctx.services.theme.mode.get() {
        Mode::Light => 0,
        Mode::Dark => 1,
        Mode::Auto => 2,
    };
    let palette = ctx.services.theme.clone();
    let mode = segmented(&[t("claro"), t("escuro"), t("automático")], current, move |i| {
        palette.set_mode([Mode::Light, Mode::Dark, Mode::Auto][i]);
    });

    let c = ctx.clone();
    let scheme = gtk::FlowBox::new();
    scheme.add_css_class("scheme-chips");
    scheme.set_selection_mode(gtk::SelectionMode::None);
    scheme.set_max_children_per_line(5);
    scheme.set_row_spacing(8);
    scheme.set_column_spacing(8);
    let mut first: Option<gtk::ToggleButton> = None;
    for (value, label) in SCHEMES {
        let chip = gtk::ToggleButton::with_label(t(label));
        chip.add_css_class("chip");
        chip.set_group(first.as_ref());
        chip.set_active(ctx.get().look.scheme == value);
        first.get_or_insert(chip.clone());
        let c = c.clone();
        chip.connect_toggled(move |b| {
            if b.is_active() {
                c.edit(|cfg| cfg.look.scheme = value.into());
            }
        });
        scheme.insert(&chip, -1);
    }

    let c = ctx.clone();
    let contrast = slider(
        -1.0,
        1.0,
        0.1,
        ctx.get().look.contrast,
        |v| format!("{v:+.1}"),
        move |v| c.edit(|cfg| cfg.look.contrast = v),
    );

    column.append(&group(&[
        &stacked("contrast", t("modo"), t("o automático escolhe pelo quanto o wallpaper é claro; muda na hora"), &mode),
        &stacked("palette", t("estilo da paleta"), t("como o matugen tira as cores do wallpaper"), &scheme),
        &stacked("tonality", t("contraste"), t("mais suave (-) ou mais forte (+)"), &contrast),
    ]));
    column.append(&hint(t("o estilo e o contraste repintam tudo ao aplicar")));
    scroll(&column).upcast()
}

pub fn language(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("idioma"), t("a língua dos painéis, das docs e da barra"));
    let c = ctx.clone();
    let chooser = choices(&[("auto", "sistema"), ("pt", "português"), ("en", "english")], &ctx.get().language, move |v| {
        c.edit(|cfg| cfg.language = v.into());
    });
    column.append(&group(&[&stacked("translate", t("idioma"), t("o octopus reinicia sozinho ao aplicar"), &chooser)]));
    scroll(&column).upcast()
}
