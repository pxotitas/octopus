use crate::i18n::t;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::panels::wallpapers::Wallpapers;

use crate::icons;
use crate::panels::lyrics::LyricsView;
use crate::services::weather::Reading;
use crate::services::Services;
use crate::widgets::{self, media, notification};

pub struct Dash {
    pub root: gtk::Box,
    stack: gtk::Stack,
    lyrics: Rc<LyricsView>,
    walls: gtk::Box,
    tabs: gtk::Box,
    picker: RefCell<Option<Rc<Wallpapers>>>,
    release: Cell<Option<glib::SourceId>>,
}

impl Dash {
    pub fn new(s: &Rc<Services>, lyrics: bool) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 14);
        root.add_css_class("surface");
        root.add_css_class("dash");

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
        stack.set_transition_duration(260);
        stack.set_vhomogeneous(false);
        stack.set_interpolate_size(true);

        let lyrics = LyricsView::new(&s.media, lyrics);
        stack.add_named(&overview(s), Some("overview"));
        stack.add_named(&media_tab(s, &lyrics), Some("media"));
        stack.add_named(&weather_tab(s), Some("weather"));
        let walls = gtk::Box::new(gtk::Orientation::Vertical, 0);
        stack.add_named(&walls, Some("wallpapers"));

        let tabs = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        tabs.add_css_class("dash-tabs");
        tabs.set_homogeneous(true);
        let mut group: Option<gtk::ToggleButton> = None;
        for (name, icon, label) in [
            ("overview", "dashboard", t("visão geral")),
            ("media", "music_note", t("mídia")),
            ("weather", "partly_cloudy_day", t("clima")),
            ("wallpapers", "wallpaper", t("wallpapers")),
        ] {
            let button = tab_button(icon, label);
            button.set_widget_name(name);
            button.set_group(group.as_ref());
            if group.is_none() {
                button.set_active(true);
            }
            let stack = stack.clone();
            button.connect_toggled(move |b| {
                if b.is_active() {
                    stack.set_visible_child_name(name);
                }
            });
            group.get_or_insert(button.clone());
            tabs.append(&button);
        }

        root.append(&tabs);
        root.append(&stack);

        let dash = Rc::new(Self {
            root,
            stack,
            lyrics,
            walls,
            tabs: tabs.clone(),
            picker: RefCell::new(None),
            release: Cell::new(None),
        });
        let weak = Rc::downgrade(&dash);
        dash.stack.connect_visible_child_name_notify(move |stack| {
            if let Some(d) = weak.upgrade() {
                let name = stack.visible_child_name();
                d.lyrics.set_active(name.as_deref() == Some("media"));
                if name.as_deref() == Some("wallpapers") {
                    d.wallpapers();
                }
            }
        });
        dash
    }

    pub fn set_open(self: &Rc<Self>, open: bool) {
        let on_media = self.stack.visible_child_name().as_deref() == Some("media");
        self.lyrics.set_active(open && on_media);
        if let Some(id) = self.release.take() {
            id.remove();
        }
        if open {
            if self.stack.visible_child_name().as_deref() == Some("wallpapers") {
                self.wallpapers();
            }
            return;
        }
        // The thumbnails go a while after the dash closes.
        let weak = Rc::downgrade(self);
        self.release.set(Some(glib::timeout_add_local_once(
            std::time::Duration::from_secs(30),
            move || {
                let Some(dash) = weak.upgrade() else { return };
                dash.release.set(None);
                if dash.picker.borrow_mut().take().is_some() {
                    crate::widgets::clear(&dash.walls);
                    crate::memory::release_soon();
                }
            },
        )));
    }

    /// Switches to one of the tabs by name.
    pub fn select(&self, name: &str) {
        let mut child = self.tabs.first_child();
        while let Some(widget) = child {
            if widget.widget_name() == name {
                if let Some(button) = widget.downcast_ref::<gtk::ToggleButton>() {
                    button.set_active(true);
                }
            }
            child = widget.next_sibling();
        }
    }

    fn wallpapers(&self) {
        if self.picker.borrow().is_some() {
            return;
        }
        let picker = Wallpapers::new();
        self.walls.append(&picker.root);
        *self.picker.borrow_mut() = Some(picker);
    }
}

fn tab_content(icon: &str, label: &str) -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let glyph = icons::icon(icon);
    glyph.add_css_class("dash-tab-icon");
    content.append(&glyph);
    content.append(&gtk::Label::new(Some(label)));
    content
}

fn tab_button(icon: &str, label: &str) -> gtk::ToggleButton {
    let button = gtk::ToggleButton::new();
    button.add_css_class("dash-tab");
    button.set_child(Some(&tab_content(icon, label)));
    button
}

fn overview(s: &Rc<Services>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    root.add_css_class("dash-overview");

    let left = gtk::Box::new(gtk::Orientation::Vertical, 12);
    left.set_size_request(400, -1);

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let clock = gtk::Box::new(gtk::Orientation::Vertical, 0);
    clock.add_css_class("card");
    clock.add_css_class("dash-clock");
    clock.set_hexpand(true);
    let time = gtk::Label::new(None);
    time.add_css_class("dash-time");
    let date = gtk::Label::new(None);
    date.add_css_class("dim");
    clock.append(&time);
    clock.append(&date);
    crate::shell::every_minute(move |now| {
        time.set_label(&now.format("%H:%M").to_string());
        date.set_label(&crate::i18n::date(&now));
    });

    let weather = gtk::Box::new(gtk::Orientation::Vertical, 2);
    weather.add_css_class("card");
    weather.add_css_class("dash-weather-mini");
    let w_row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let w_icon = icons::icon("cloud");
    w_icon.add_css_class("big-icon");
    let w_temp = gtk::Label::new(None);
    w_temp.add_css_class("dash-temp");
    w_row.append(&w_icon);
    w_row.append(&w_temp);
    let w_desc = gtk::Label::new(None);
    w_desc.add_css_class("dim");
    w_desc.set_xalign(0.0);
    weather.append(&w_row);
    weather.append(&w_desc);
    s.weather.reading.subscribe(move |r| {
        if let Some(r) = r {
            w_icon.set_label(&r.icon);
            w_temp.set_label(&format!("{}°", r.temperature));
            w_desc.set_label(&r.description.to_lowercase());
        } else {
            w_temp.set_label("--°");
            w_desc.set_label(t("sem dados"));
        }
    });
    top.append(&clock);
    top.append(&weather);
    left.append(&top);

    let calendar = gtk::Calendar::new();
    calendar.add_css_class("card");
    calendar.add_css_class("dash-calendar");
    left.append(&calendar);

    let player = media::player(&s.media, media::Size::Card, None);
    player.add_css_class("card");
    left.append(&player);
    root.append(&left);

    let right = gtk::Box::new(gtk::Orientation::Vertical, 8);
    right.add_css_class("card");
    right.set_size_request(360, -1);
    right.set_hexpand(true);
    let clear = icons::button("clear_all", t("limpar tudo"));
    {
        let n = s.notifications.clone();
        clear.connect_clicked(move |_| n.clear());
    }
    right.append(&widgets::heading(
        t("notificações"),
        Some(clear.upcast_ref()),
    ));
    right.append(&notification::list(&s.notifications));
    root.append(&right);
    root
}

fn media_tab(s: &Rc<Services>, lyrics: &Rc<LyricsView>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    root.add_css_class("dash-media");
    root.set_size_request(-1, 520);

    let player = media::player(&s.media, media::Size::Hero, None);
    player.add_css_class("card");
    player.set_size_request(320, -1);
    root.append(&player);

    let card = gtk::Box::new(gtk::Orientation::Vertical, 0);
    card.add_css_class("card");
    card.set_hexpand(true);
    card.append(&lyrics.root);
    root.append(&card);
    root
}

fn weather_tab(s: &Rc<Services>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 12);
    root.add_css_class("dash-weather");

    let hero = gtk::Box::new(gtk::Orientation::Horizontal, 20);
    hero.add_css_class("card");
    hero.add_css_class("weather-hero");
    let icon = icons::icon("cloud");
    icon.add_css_class("hero-icon");
    let main = gtk::Box::new(gtk::Orientation::Vertical, 2);
    main.set_valign(gtk::Align::Center);
    main.set_hexpand(true);
    let temp = gtk::Label::new(None);
    temp.add_css_class("hero-temp");
    temp.set_xalign(0.0);
    let desc = gtk::Label::new(None);
    desc.set_xalign(0.0);
    let place = gtk::Label::new(None);
    place.add_css_class("dim");
    place.set_xalign(0.0);
    main.append(&temp);
    main.append(&desc);
    main.append(&place);
    hero.append(&icon);
    hero.append(&main);

    let readings = gtk::Grid::new();
    readings.set_column_spacing(24);
    readings.set_row_spacing(8);
    readings.set_valign(gtk::Align::Center);
    hero.append(&readings);
    root.append(&hero);

    let hours = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    hours.set_homogeneous(true);
    let hours_card = gtk::Box::new(gtk::Orientation::Vertical, 10);
    hours_card.add_css_class("card");
    hours_card.append(&widgets::heading(t("próximas horas"), None));
    hours_card.append(&hours);
    root.append(&hours_card);

    s.weather.reading.subscribe(move |reading| {
        widgets::clear(&hours);
        while let Some(child) = readings.first_child() {
            readings.remove(&child);
        }
        let Some(r) = reading else {
            temp.set_label("--°");
            desc.set_label(t("sem dados do clima"));
            place.set_label("");
            return;
        };
        icon.set_label(&r.icon);
        temp.set_label(&format!("{}°", r.temperature));
        desc.set_label(&r.description.to_lowercase());
        place.set_label(&r.place);
        fill_readings(&readings, r);
        for h in upcoming(r) {
            let cell = gtk::Box::new(gtk::Orientation::Vertical, 4);
            cell.add_css_class("hour");
            let label = gtk::Label::new(Some(&format!("{:02}h", h.hour)));
            label.add_css_class("dim");
            cell.append(&label);
            cell.append(&icons::icon(&h.icon));
            cell.append(&gtk::Label::new(Some(&format!("{}°", h.temperature))));
            let rain = gtk::Label::new(Some(&format!("{}%", h.rain)));
            rain.add_css_class("rain");
            rain.set_visible(h.rain > 0);
            cell.append(&rain);
            hours.append(&cell);
        }
    });
    root
}

fn fill_readings(grid: &gtk::Grid, r: &Reading) {
    let rows = [
        ("thermostat", crate::i18n::tf("sensação {}°", r.feels_like)),
        (
            "humidity_percentage",
            crate::i18n::tf("umidade {}%", r.humidity),
        ),
        ("air", crate::i18n::tf("vento {} km/h", r.wind)),
        (
            "arrow_downward",
            crate::i18n::tf("mín {}° · máx {}°", r.low).replacen("{}", &r.high.to_string(), 1),
        ),
    ];
    for (i, (icon, text)) in rows.into_iter().enumerate() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.append(&icons::icon(icon));
        row.append(&gtk::Label::new(Some(&text)));
        grid.attach(&row, (i % 2) as i32, (i / 2) as i32, 1, 1);
    }
}

/// The next eight three-hour blocks from now.
fn upcoming(r: &Reading) -> impl Iterator<Item = &crate::services::weather::Hour> {
    let now = chrono::Timelike::hour(&chrono::Local::now());
    r.hourly
        .iter()
        .filter(move |h| h.tomorrow || h.hour + 3 > now)
        .take(8)
}
