//! barra › presets: whole bars to start from, each shown as it would look.

use std::rc::Rc;

use gtk::prelude::*;

use super::bar_preview::still;
use super::rows::*;
use super::Ctx;
use crate::bar::presets::{self, PRESETS};
use crate::i18n::t;

pub fn build(ctx: &Rc<Ctx>) -> gtk::Widget {
    let column = page(t("presets"), t("uma barra inteira de uma vez; depois dá pra ajustar no editor"));
    let flow = gtk::FlowBox::new();
    flow.add_css_class("preset-grid");
    flow.set_selection_mode(gtk::SelectionMode::None);
    flow.set_homogeneous(true);
    flow.set_max_children_per_line(5);
    flow.set_min_children_per_line(3);
    flow.set_row_spacing(12);
    flow.set_column_spacing(12);

    for preset in PRESETS.iter() {
        let mut bar = ctx.get().bar.clone();
        presets::apply(preset, &mut bar);
        let card = gtk::Button::new();
        card.add_css_class("preset-card");
        let inside = gtk::Box::new(gtk::Orientation::Vertical, 10);
        inside.append(&still(&bar, 300));
        let name = gtk::Label::new(Some(t(preset.name)));
        name.add_css_class("module-name");
        let about = gtk::Label::new(Some(t(preset.about)));
        about.add_css_class("settings-item-sub");
        about.set_wrap(true);
        about.set_justify(gtk::Justification::Center);
        about.set_max_width_chars(18);
        inside.append(&name);
        inside.append(&about);
        card.set_child(Some(&inside));
        let c = ctx.clone();
        card.connect_clicked(move |_| c.edit(|cfg| presets::apply(preset, &mut cfg.bar)));
        flow.insert(&card, -1);
    }
    column.append(&flow);
    column.append(&hint(t("escolher um preset troca os módulos da barra; como cada um se parece fica como estava")));
    scroll(&column).upcast()
}
