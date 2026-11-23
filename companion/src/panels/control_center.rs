use crate::i18n::t;
use std::rc::Rc;

use gtk::prelude::*;

use crate::icons;
use crate::services::audio::Kind;
use crate::services::Services;
use crate::widgets::shape::{Kind as ShapeKind, Shape};
use crate::widgets::{self, media, notification, slider::Slider, tile::Tile};

pub fn build(s: &Rc<Services>, open: Rc<dyn Fn(&str)>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.add_css_class("surface");
    root.add_css_class("control-center");

    root.append(&header(s, open.clone()));
    root.append(&tiles(s, open));
    root.append(&sliders(s));

    let player = media::player(&s.media, media::Size::Card, Some(&s.cava));
    player.add_css_class("card");
    root.append(&player);

    root.append(&notifications(s));
    root
}

fn header(s: &Rc<Services>, open: Rc<dyn Fn(&str)>) -> gtk::CenterBox {
    let bar = gtk::CenterBox::new();
    bar.add_css_class("cc-header");

    let left = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let time = gtk::Label::new(None);
    time.add_css_class("cc-time");
    time.set_xalign(0.0);
    let date = gtk::Label::new(None);
    date.add_css_class("cc-date");
    date.set_xalign(0.0);
    let weather = gtk::Box::new(gtk::Orientation::Horizontal, 4);
    weather.add_css_class("cc-weather");
    let weather_icon = icons::icon("cloud");
    let weather_text = gtk::Label::new(None);
    weather.append(&weather_icon);
    weather.append(&weather_text);
    left.append(&time);
    left.append(&date);
    left.append(&weather);
    bar.set_start_widget(Some(&left));

    let power = gtk::Button::new();
    power.set_tooltip_text(Some(t("energia")));
    power.set_child(Some(&Shape::new(
        ShapeKind::Cookie9,
        44,
        true,
        Some(&icons::icon("power_settings_new")),
    )));
    power.add_css_class("cc-power");
    power.set_valign(gtk::Align::Start);
    power.connect_clicked(move |_| open("power"));
    bar.set_end_widget(Some(&power));

    crate::shell::every_minute(move |now| {
        time.set_label(&now.format("%H:%M").to_string());
        date.set_label(&crate::i18n::date(&now));
    });
    s.weather.reading.subscribe(move |reading| {
        weather.set_visible(reading.is_some());
        if let Some(r) = reading {
            weather_icon.set_label(&r.icon);
            weather_text.set_label(&format!(
                "{}° · {}",
                r.temperature,
                r.description.to_lowercase()
            ));
        }
    });
    bar
}

fn tiles(s: &Rc<Services>, open: Rc<dyn Fn(&str)>) -> gtk::Grid {
    let grid = gtk::Grid::new();
    grid.add_css_class("tiles");
    grid.set_column_spacing(8);
    grid.set_row_spacing(8);
    grid.set_column_homogeneous(true);

    let wifi = Tile::new("wifi", t("wi-fi"), true);
    {
        let w = s.wifi.clone();
        wifi.on_toggle(move || {
            let on = w.state.get().enabled;
            w.set_enabled(!on);
        });
        let open = open.clone();
        wifi.on_menu(move || open("wifi"));
        let tile = wifi.clone();
        s.wifi.state.subscribe(move |st| {
            let (icon, sub) = match (st.enabled, st.active()) {
                (false, _) => ("wifi_off", t("desligado").to_owned()),
                (true, Some(n)) => (icons::wifi(n.signal), n.ssid.clone()),
                (true, None) => ("wifi", t("desconectado").to_owned()),
            };
            tile.set(st.enabled, icon, &sub);
        });
    }

    let bt = Tile::new("bluetooth", "bluetooth", true);
    {
        let b = s.bluetooth.clone();
        bt.on_toggle(move || {
            let on = b.state.get().powered;
            b.set_powered(!on);
        });
        let open = open.clone();
        bt.on_menu(move || open("bluetooth"));
        let tile = bt.clone();
        s.bluetooth.state.subscribe(move |st| {
            let (icon, sub) = match (st.powered, st.connected()) {
                (false, _) => ("bluetooth_disabled", t("desligado").to_owned()),
                (true, Some(d)) => ("bluetooth_connected", d.name.clone()),
                (true, None) => ("bluetooth", t("ligado").to_owned()),
            };
            tile.set(st.powered, icon, &sub);
        });
    }

    let mic = Tile::new("mic", t("microfone"), true);
    {
        let a = s.audio.clone();
        mic.on_toggle(move || a.toggle_mute(Kind::Source));
        mic.on_menu(move || open("audio"));
        let tile = mic.clone();
        s.audio.state.subscribe(move |st| {
            let muted = st.source().is_none_or(|d| d.muted);
            tile.set(
                !muted,
                if muted { "mic_off" } else { "mic" },
                if muted { t("mudo") } else { t("ativo") },
            );
        });
    }

    let theme = Tile::new("dark_mode", t("tema"), false);
    {
        let mode = s.theme.clone();
        theme.on_toggle(move || mode.toggle());
        let tile = theme.clone();
        let current = s.theme.mode.clone();
        s.theme.dark.subscribe(move |dark| {
            let auto = *current.get() == crate::services::theme::Mode::Auto;
            tile.set(
                *dark,
                if *dark { "dark_mode" } else { "light_mode" },
                if auto {
                    t("automático")
                } else if *dark {
                    t("escuro")
                } else {
                    t("claro")
                },
            );
        });
    }

    grid.attach(&wifi.root, 0, 0, 1, 1);
    grid.attach(&bt.root, 1, 0, 1, 1);
    grid.attach(&mic.root, 0, 1, 1, 1);
    grid.attach(&theme.root, 1, 1, 1, 1);
    grid
}

fn sliders(s: &Rc<Services>) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 4);
    column.add_css_class("card");
    column.add_css_class("sliders");

    for kind in [Kind::Sink, Kind::Source] {
        let audio = s.audio.clone();
        let slider = Slider::new("volume_up", t("silenciar"), move |v| {
            audio.set_volume(kind, v)
        });
        let audio = s.audio.clone();
        slider.on_icon(move || audio.toggle_mute(kind));
        column.append(&slider.root);
        s.audio.state.subscribe(move |st| {
            let device = match kind {
                Kind::Sink => st.sink(),
                Kind::Source => st.source(),
            };
            let Some(d) = device else { return };
            let icon = match kind {
                Kind::Sink => icons::volume(d.volume, d.muted),
                Kind::Source if d.muted => "mic_off",
                Kind::Source => "mic",
            };
            slider.set(d.volume, icon);
            slider.set_muted(d.muted);
        });
    }

    for (light, icon) in [(&s.screen, "brightness_6"), (&s.keyboard, "keyboard")] {
        let l = light.clone();
        let slider = Slider::new(icon, "", move |v| l.set(v));
        column.append(&slider.root);
        light.percent.subscribe(move |p| {
            slider.root.set_visible(p.is_some());
            if let Some(p) = p {
                slider.set(*p, icon);
            }
        });
    }
    column
}

fn notifications(s: &Rc<Services>) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.add_css_class("card");
    card.add_css_class("cc-notifications");

    let clear = icons::button("clear_all", t("limpar tudo"));
    {
        let n = s.notifications.clone();
        clear.connect_clicked(move |_| n.clear());
    }
    let dnd = icons::button("notifications", t("não perturbe"));
    {
        let n = s.notifications.clone();
        dnd.connect_clicked(move |_| {
            let on = *n.dnd.get();
            n.dnd.set(!on);
        });
        let button = dnd.clone();
        s.notifications.dnd.subscribe(move |on| {
            icons::set(
                &button,
                if *on {
                    "notifications_off"
                } else {
                    "notifications"
                },
            );
            if *on {
                button.add_css_class("on");
            } else {
                button.remove_css_class("on");
            }
        });
    }
    let actions = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    actions.append(&dnd);
    actions.append(&clear);
    card.append(&widgets::heading(
        t("notificações"),
        Some(actions.upcast_ref()),
    ));
    let list = notification::list(&s.notifications);
    list.set_size_request(-1, 120);
    card.append(&list);
    {
        let clear = clear.clone();
        s.notifications
            .list
            .subscribe(move |l| clear.set_visible(!l.is_empty()));
    }
    card
}
