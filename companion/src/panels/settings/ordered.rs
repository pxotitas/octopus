//! A group where each item is turned on or off and moved up or down: the
//! order of the list is the order on screen.

use std::rc::Rc;

use gtk::prelude::*;

use super::rows::*;
use super::Ctx;
use crate::config::Config;
use crate::i18n::t;
use crate::icons;

pub type Pick = fn(&mut Config) -> &mut Vec<String>;

/// `items` are (id, icon, name, about); those on come first, in their order.
pub fn group(ctx: &Rc<Ctx>, items: &'static [(&'static str, &'static str, &'static str, &'static str)], pick: Pick) -> gtk::Box {
    let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    fill(ctx, &holder, items, pick);
    holder
}

fn fill(ctx: &Rc<Ctx>, holder: &gtk::Box, items: &'static [(&'static str, &'static str, &'static str, &'static str)], pick: Pick) {
    crate::widgets::clear(holder);
    let chosen: Vec<String> = pick(&mut ctx.get().clone()).clone();
    let mut ordered: Vec<_> = chosen.iter().filter_map(|id| items.iter().find(|i| i.0 == id)).collect();
    ordered.extend(items.iter().filter(|i| !chosen.iter().any(|c| c == i.0)));

    let mut rows: Vec<gtk::Widget> = Vec::new();
    for (index, &&(id, icon, name, about)) in ordered.iter().enumerate() {
        let on = chosen.iter().any(|c| c == id);
        let controls = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        let refill = {
            let (ctx, holder) = (ctx.clone(), holder.clone());
            move || {
                let (ctx, holder) = (ctx.clone(), holder.clone());
                gtk::glib::idle_add_local_once(move || fill(&ctx, &holder, items, pick));
            }
        };
        if on {
            let position = chosen.iter().position(|c| c == id).unwrap_or(index);
            for (glyph, tip, step) in [("arrow_upward", "subir", -1i32), ("arrow_downward", "descer", 1)] {
                let target = position as i32 + step;
                let button = icons::button(glyph, t(tip));
                button.add_css_class("order-step");
                button.set_sensitive(target >= 0 && (target as usize) < chosen.len());
                let (ctx, refill) = (ctx.clone(), refill.clone());
                button.connect_clicked(move |_| {
                    ctx.edit(|cfg| pick(cfg).swap(position, target as usize));
                    refill();
                });
                controls.append(&button);
            }
        }
        let (c, refill) = (ctx.clone(), refill.clone());
        let toggle = switch(on, move |active| {
            c.edit(|cfg| {
                let list = pick(cfg);
                list.retain(|x| x != id);
                if active {
                    list.push(id.into());
                }
            });
            refill();
        });
        controls.append(&toggle);
        let row = inline(icon, t(name), t(about), &controls);
        if !on {
            row.add_css_class("off");
        }
        rows.push(row);
    }
    let refs: Vec<&gtk::Widget> = rows.iter().collect();
    holder.append(&super::rows::group(&refs));
}
