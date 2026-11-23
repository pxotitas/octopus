use crate::i18n::t;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::pango;
use gtk::prelude::*;

use crate::icons;
use crate::services::network::{Network, Wifi};
use crate::widgets;

pub fn build(wifi: &Rc<Wifi>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.add_css_class("surface");
    root.add_css_class("menu");

    let (header, switch) = widgets::switch_header(t("wi-fi"));
    root.append(&header);

    let list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_height(true)
        .max_content_height(360)
        .child(&list)
        .build();
    root.append(&scroller);

    let pending: Rc<RefCell<Option<String>>> = Rc::default();
    let password = gtk::Revealer::new();
    let form = gtk::Box::new(gtk::Orientation::Vertical, 8);
    form.add_css_class("password-form");
    let prompt = gtk::Label::new(None);
    prompt.set_xalign(0.0);
    let entry = gtk::PasswordEntry::new();
    entry.set_show_peek_icon(true);
    let join = gtk::Button::with_label(t("conectar"));
    join.add_css_class("pill");
    join.add_css_class("filled");
    form.append(&prompt);
    form.append(&entry);
    form.append(&join);
    password.set_child(Some(&form));
    root.append(&password);

    let status = gtk::Label::new(None);
    status.add_css_class("menu-status");
    status.set_wrap(true);
    status.set_xalign(0.0);
    root.append(&status);

    let footer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    footer.add_css_class("menu-footer");
    let scan = gtk::Button::with_label(t("procurar redes"));
    scan.add_css_class("pill");
    let settings = gtk::Button::with_label(t("configurações"));
    settings.add_css_class("pill");
    settings.set_hexpand(true);
    settings.set_halign(gtk::Align::End);
    footer.append(&scan);
    footer.append(&settings);
    root.append(&footer);

    let handler;
    {
        let wifi = wifi.clone();
        handler = switch.connect_state_set(move |_, on| {
            wifi.set_enabled(on);
            gtk::glib::Propagation::Proceed
        });
    }
    {
        let wifi = wifi.clone();
        scan.connect_clicked(move |_| wifi.rescan());
    }
    settings.connect_clicked(|_| crate::exec::spawn("nm-connection-editor", &[]));

    let submit = {
        let (wifi, pending, entry, password) = (
            wifi.clone(),
            pending.clone(),
            entry.clone(),
            password.clone(),
        );
        move || {
            if let Some(ssid) = pending.borrow_mut().take() {
                wifi.connect(&ssid, Some(entry.text().as_str()));
            }
            entry.set_text("");
            password.set_reveal_child(false);
        }
    };
    let submit = Rc::new(submit);
    {
        let submit = submit.clone();
        join.connect_clicked(move |_| submit());
    }
    entry.connect_activate(move |_| submit());

    let weak = Rc::downgrade(wifi);
    wifi.state.subscribe(move |state| {
        let Some(wifi) = weak.upgrade() else { return };
        if switch.is_active() != state.enabled {
            switch.block_signal(&handler);
            switch.set_active(state.enabled);
            switch.set_state(state.enabled);
            switch.unblock_signal(&handler);
        }
        widgets::clear(&list);
        scroller.set_visible(state.enabled);
        footer.set_visible(state.enabled);
        for network in &state.networks {
            let busy = state.busy.as_deref() == Some(network.ssid.as_str());
            list.append(&row(
                network, busy, &wifi, &pending, &password, &prompt, &entry,
            ));
        }
        status.set_label(state.error.as_deref().unwrap_or(""));
        status.set_visible(state.error.is_some() || !state.enabled);
        if !state.enabled {
            status.set_label(t("o wi-fi está desligado"));
        }
    });

    root
}

fn row(
    network: &Network,
    busy: bool,
    wifi: &Rc<Wifi>,
    pending: &Rc<RefCell<Option<String>>>,
    password: &gtk::Revealer,
    prompt: &gtk::Label,
    entry: &gtk::PasswordEntry,
) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("menu-row");
    if network.active {
        button.add_css_class("active");
    }
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.append(&icons::icon(icons::wifi(network.signal)));

    let text = gtk::Box::new(gtk::Orientation::Vertical, 0);
    text.set_hexpand(true);
    let name = gtk::Label::new(Some(&network.ssid));
    name.set_xalign(0.0);
    name.set_ellipsize(pango::EllipsizeMode::End);
    text.append(&name);
    let detail = match (busy, network.active, network.known) {
        (true, _, _) => Some(t("conectando…")),
        (_, true, _) => Some(t("conectada · toque pra desconectar")),
        (_, _, true) => Some(t("salva")),
        _ => None,
    };
    if let Some(detail) = detail {
        let label = gtk::Label::new(Some(detail));
        label.add_css_class("dim");
        label.set_xalign(0.0);
        text.append(&label);
    }
    content.append(&text);
    if network.secure {
        let lock = icons::icon("lock");
        lock.add_css_class("dim");
        content.append(&lock);
    }
    button.set_child(Some(&content));

    let (wifi, pending, password, prompt, entry) = (
        wifi.clone(),
        pending.clone(),
        password.clone(),
        prompt.clone(),
        entry.clone(),
    );
    let network = network.clone();
    button.connect_clicked(move |_| {
        if network.active {
            wifi.disconnect(&network.ssid);
        } else if network.secure && !network.known {
            prompt.set_label(&crate::i18n::tf("senha de {}", &network.ssid));
            *pending.borrow_mut() = Some(network.ssid.clone());
            password.set_reveal_child(true);
            entry.grab_focus();
        } else {
            password.set_reveal_child(false);
            wifi.connect(&network.ssid, None);
        }
    });
    button
}
