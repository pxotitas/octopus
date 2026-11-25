//! The bluetooth menu: the adapter switch and the known devices.

use crate::i18n::t;
use std::rc::Rc;

use gtk::pango;
use gtk::prelude::*;

use crate::services::bluetooth::{Bluetooth, Device};
use crate::widgets;

pub fn build(bt: &Rc<Bluetooth>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.add_css_class("surface");
    root.add_css_class("menu");

    let (header, switch) = widgets::switch_header("bluetooth");
    root.append(&header);

    let list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_height(true)
        .max_content_height(360)
        .child(&list)
        .build();
    root.append(&scroller);

    let status = gtk::Label::new(None);
    status.add_css_class("menu-status");
    status.set_wrap(true);
    status.set_xalign(0.0);
    root.append(&status);

    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    footer.add_css_class("menu-footer");
    let scan = gtk::Button::with_label(t("procurar dispositivos"));
    scan.add_css_class("pill");
    footer.append(&scan);
    root.append(&footer);

    let handler;
    {
        let bt = bt.clone();
        handler = switch.connect_state_set(move |_, on| {
            bt.set_powered(on);
            gtk::glib::Propagation::Proceed
        });
    }
    {
        let bt = bt.clone();
        scan.connect_clicked(move |_| bt.scan());
    }

    let weak = Rc::downgrade(bt);
    bt.state.subscribe(move |state| {
        let Some(bt) = weak.upgrade() else { return };
        switch.set_sensitive(state.available);
        if switch.is_active() != state.powered {
            switch.block_signal(&handler);
            switch.set_active(state.powered);
            switch.set_state(state.powered);
            switch.unblock_signal(&handler);
        }
        scan.set_label(if state.scanning {
            t("procurando…")
        } else {
            t("procurar dispositivos")
        });
        scan.set_sensitive(!state.scanning);
        widgets::clear(&list);
        let on = state.available && state.powered;
        scroller.set_visible(on && !state.devices.is_empty());
        footer.set_visible(on);
        for device in &state.devices {
            let busy = state.busy.as_deref() == Some(device.address.as_str());
            list.append(&row(device, busy, &bt));
        }
        let message = if !state.available {
            Some(t("nenhum adaptador bluetooth"))
        } else if !state.powered {
            Some(t("o bluetooth está desligado"))
        } else {
            state.error.as_deref()
        };
        status.set_label(message.unwrap_or(""));
        status.set_visible(message.is_some());
    });

    root
}

fn row(device: &Device, busy: bool, bt: &Rc<Bluetooth>) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("menu-row");
    if device.connected {
        button.add_css_class("active");
    }
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.append(&crate::widgets::shape::badge(
        if device.connected { "bluetooth_connected" } else { "bluetooth" },
        device.connected,
    ));
    let text = gtk::Box::new(gtk::Orientation::Vertical, 0);
    text.set_hexpand(true);
    let name = gtk::Label::new(Some(&device.name));
    name.set_xalign(0.0);
    name.set_ellipsize(pango::EllipsizeMode::End);
    text.append(&name);
    let detail = match (busy, device.connected, device.paired) {
        (true, _, _) => t("aguarde…"),
        (_, true, _) => t("conectado"),
        (_, _, true) => t("pareado"),
        _ => t("toque pra parear"),
    };
    let label = gtk::Label::new(Some(detail));
    label.add_css_class("dim");
    label.set_xalign(0.0);
    text.append(&label);
    content.append(&text);
    button.set_child(Some(&content));

    let (bt, address) = (bt.clone(), device.address.clone());
    button.connect_clicked(move |_| bt.toggle(&address));
    button
}
