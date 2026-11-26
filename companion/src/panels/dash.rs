use crate::i18n::t;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::panels::wallpapers::Wallpapers;

use crate::icons;
use crate::panels::lyrics::LyricsView;
use crate::panels::weather::{mini, WeatherPanel};
use crate::services::Services;
use crate::widgets::clock::Clock;
use crate::widgets::{self, media, notification};

pub struct Dash {
    pub root: gtk::Box,
    stack: gtk::Stack,
    lyrics: Rc<LyricsView>,
    weather: Rc<WeatherPanel>,
    walls: gtk::Box,
    rail: gtk::Box,
    picker: RefCell<Option<Rc<Wallpapers>>>,
    release: Cell<Option<glib::SourceId>>,
}

impl Dash {
    pub fn new(s: &Rc<Services>, lyrics: bool) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        root.add_css_class("surface");
        root.add_css_class("dash");

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::SlideUpDown);
        stack.set_transition_duration(260);
        stack.set_vhomogeneous(false);
        stack.set_interpolate_size(true);
        stack.set_hexpand(true);

        let rail = gtk::Box::new(gtk::Orientation::Vertical, 6);
        rail.add_css_class("dash-rail");
        rail.set_valign(gtk::Align::Fill);

        let lyrics = LyricsView::new(&s.media, lyrics);
        let weather = WeatherPanel::new(s);
        let walls = gtk::Box::new(gtk::Orientation::Vertical, 0);

        let dash = Rc::new_cyclic(|weak: &std::rc::Weak<Dash>| {
            let to_weather = {
                let weak = weak.clone();
                move || {
                    if let Some(d) = weak.upgrade() {
                        d.select("weather");
                    }
                }
            };
            stack.add_named(&overview(s, to_weather), Some("overview"));
            stack.add_named(&media_tab(s, &lyrics), Some("media"));
            stack.add_named(&weather.root, Some("weather"));
            stack.add_named(&walls, Some("wallpapers"));
            Self {
                root: root.clone(),
                stack: stack.clone(),
                lyrics: lyrics.clone(),
                weather: weather.clone(),
                walls: walls.clone(),
                rail: rail.clone(),
                picker: RefCell::new(None),
                release: Cell::new(None),
            }
        });

        let mut group: Option<gtk::ToggleButton> = None;
        for (name, icon, label) in [
            ("overview", "dashboard", t("visão geral")),
            ("media", "music_note", t("mídia")),
            ("weather", "partly_cloudy_day", t("clima")),
            ("wallpapers", "wallpaper", t("wallpapers")),
        ] {
            let button = rail_item(icon, label);
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
            rail.append(&button);
        }

        // the settings, apart from the tabs at the rail's foot
        let gap = gtk::Box::new(gtk::Orientation::Vertical, 0);
        gap.set_vexpand(true);
        rail.append(&gap);
        let settings = gtk::Button::new();
        settings.add_css_class("rail-item");
        settings.add_css_class("rail-settings");
        let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let indicator = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        indicator.add_css_class("rail-indicator");
        indicator.set_halign(gtk::Align::Center);
        let glyph = icons::icon("settings");
        glyph.set_hexpand(true);
        indicator.append(&glyph);
        content.append(&indicator);
        let text = gtk::Label::new(Some(t("ajustes")));
        text.add_css_class("rail-label");
        content.append(&text);
        settings.set_child(Some(&content));
        settings.set_tooltip_text(Some(t("configurações")));
        settings.connect_clicked(|_| crate::exec::spawn("octopus", &["settings"]));
        rail.append(&settings);

        root.append(&rail);
        root.append(&stack);

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
        let mut child = self.rail.first_child();
        while let Some(widget) = child {
            if widget.widget_name() == name {
                if let Some(button) = widget.downcast_ref::<gtk::ToggleButton>() {
                    button.set_active(true);
                }
            }
            child = widget.next_sibling();
        }
    }

    /// The weather tab, asking it something (`> amanhã`).
    pub fn ask_weather(&self, text: &str) {
        self.select("weather");
        if !text.trim().is_empty() {
            self.weather.ask(text);
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

/// An M3 navigation rail item: the icon in a pill that fills when active,
/// the label under it.
fn rail_item(icon: &str, label: &str) -> gtk::ToggleButton {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 4);
    let indicator = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    indicator.add_css_class("rail-indicator");
    indicator.set_halign(gtk::Align::Center);
    let glyph = icons::icon(icon);
    glyph.set_hexpand(true);
    indicator.append(&glyph);
    content.append(&indicator);
    let text = gtk::Label::new(Some(label));
    text.add_css_class("rail-label");
    content.append(&text);
    let button = gtk::ToggleButton::new();
    button.add_css_class("rail-item");
    button.set_child(Some(&content));
    button.set_tooltip_text(Some(label));
    button
}

fn overview(s: &Rc<Services>, to_weather: impl Fn() + 'static) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    root.add_css_class("dash-overview");

    let left = gtk::Box::new(gtk::Orientation::Vertical, 12);
    left.set_size_request(400, -1);

    let top = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    let clock = gtk::Box::new(gtk::Orientation::Vertical, 0);
    clock.add_css_class("card");
    clock.add_css_class("dash-clock");
    clock.set_hexpand(true);
    clock.set_valign(gtk::Align::Fill);
    let time = gtk::Label::new(None);
    time.add_css_class("dash-time");
    time.set_vexpand(true);
    time.set_valign(gtk::Align::End);
    let date = gtk::Label::new(None);
    date.add_css_class("dim");
    date.set_vexpand(true);
    date.set_valign(gtk::Align::Start);
    clock.append(&time);
    clock.append(&date);
    crate::shell::every_minute(move |now| {
        time.set_label(&now.format("%H:%M").to_string());
        date.set_label(&crate::i18n::date(&now));
    });

    let analog = Clock::new(112);
    analog.add_css_class("dash-analog");
    let face = gtk::Box::new(gtk::Orientation::Vertical, 0);
    face.add_css_class("card");
    face.add_css_class("dash-analog-card");
    face.append(&analog);
    top.append(&clock);
    top.append(&face);
    left.append(&top);

    let calendar = crate::widgets::agenda::calendar(&s.calendar, "dash-calendar");
    calendar.add_css_class("card");
    calendar.add_css_class("dash-calendar-card");
    left.append(&calendar);

    let player = media::player(&s.media, media::Size::Card, None);
    player.add_css_class("card");
    left.append(&player);
    root.append(&left);

    let right = gtk::Box::new(gtk::Orientation::Vertical, 12);
    right.set_size_request(340, -1);
    right.set_hexpand(true);
    right.set_homogeneous(true);

    let notes = gtk::Box::new(gtk::Orientation::Vertical, 8);
    notes.add_css_class("card");
    let clear = icons::button("clear_all", t("limpar tudo"));
    {
        let n = s.notifications.clone();
        clear.connect_clicked(move |_| n.clear());
    }
    notes.append(&widgets::heading(t("notificações"), Some(clear.upcast_ref())));
    notes.append(&notification::list_sized(&s.notifications, 190));
    right.append(&notes);

    let weather = mini::card(s, 5);
    weather.add_css_class("card");
    weather.add_css_class("activatable");
    weather.set_tooltip_text(Some(t("abrir o clima")));
    let click = gtk::GestureClick::new();
    click.connect_released(move |_, _, _, _| to_weather());
    weather.add_controller(click);
    right.append(&weather);
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
    card.set_size_request(320, -1);
    card.append(&lyrics.root);
    root.append(&card);
    root
}
