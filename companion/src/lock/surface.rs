use crate::i18n::t;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::icons;
use crate::services::lyrics::{self, Lyrics};
use crate::services::Services;
use crate::widgets::anim::{self, Tween};
use crate::widgets::shape::{Kind, Shape};

const RESTING_WIDTH: i32 = 300;
const CLOCK: i32 = 220;
const CLOCK_PLAYING: i32 = 132;
const COVER: i32 = 168;
const TYPING_WIDTH: i32 = 380;

/// One monitor's lock: the blurred wallpaper, the clock with hours and
/// minutes in two M3 shapes that turn and change at every key, the password
/// pill, what is playing with the lyric being sung, the weather, the battery
/// and the session buttons.
pub struct Surface {
    pub window: gtk::Window,
    background: gtk::Picture,
    content: gtk::Overlay,
    shapes: [Shape; 2],
    pill: gtk::Box,
    resting: gtk::Box,
    field: gtk::Text,
    send: gtk::Button,
    status: gtk::Label,
    shown: Rc<Tween>,
    width: Rc<Tween>,
    shake: Rc<Tween>,
    typing: Cell<bool>,
    length: Cell<usize>,
}

impl Surface {
    pub fn new(
        app: &gtk::Application,
        s: &Rc<Services>,
        lyrics: bool,
        on_submit: Rc<dyn Fn(String)>,
    ) -> Rc<Self> {
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .build();
        window.add_css_class("octopus-lock");

        let root = gtk::Overlay::new();
        let background = gtk::Picture::new();
        background.set_content_fit(gtk::ContentFit::Cover);
        root.set_child(Some(&background));
        let scrim = gtk::Box::new(gtk::Orientation::Vertical, 0);
        scrim.add_css_class("lock-scrim");
        scrim.set_can_target(false);
        root.add_overlay(&scrim);

        let content = gtk::Overlay::new();
        content.set_opacity(0.0);
        content.set_child(Some(&gtk::Box::new(gtk::Orientation::Vertical, 0)));
        root.add_overlay(&content);

        let center = gtk::Box::new(gtk::Orientation::Vertical, 0);
        center.set_halign(gtk::Align::Center);
        center.set_valign(gtk::Align::Center);
        center.set_margin_bottom(40);
        let date = gtk::Label::new(None);
        date.add_css_class("lock-date");
        center.append(&date);

        let hours = gtk::Label::new(None);
        hours.add_css_class("lock-digits");
        let minutes = gtk::Label::new(None);
        minutes.add_css_class("lock-digits");
        let first = Shape::new(Kind::Cookie9, CLOCK, true, Some(&hours));
        first.add_css_class("lock-hours");
        let second = Shape::new(Kind::Clover4, CLOCK, true, Some(&minutes));
        second.add_css_class("lock-minutes");
        let clock = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        clock.add_css_class("lock-clock");
        clock.set_halign(gtk::Align::Center);
        clock.append(&first);
        clock.append(&second);
        center.append(&clock);

        // Velatura's place for the music: between the clock and the account,
        // the clock drawn smaller while it is there.
        let compact: Rc<dyn Fn(bool)> = {
            let (first, second, clock) = (first.downgrade(), second.downgrade(), clock.downgrade());
            Rc::new(move |playing: bool| {
                let size = if playing { CLOCK_PLAYING } else { CLOCK };
                for shape in [first.upgrade(), second.upgrade()].into_iter().flatten() {
                    shape.set_size(size);
                }
                if let Some(clock) = clock.upgrade() {
                    if playing {
                        clock.add_css_class("compact");
                    } else {
                        clock.remove_css_class("compact");
                    }
                }
            })
        };
        center.append(&music(s, lyrics, compact));

        let pill = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        pill.add_css_class("lock-pill");
        pill.set_halign(gtk::Align::Center);
        pill.set_margin_top(40);
        pill.set_size_request(RESTING_WIDTH, 60);
        pill.append(&avatar());

        let resting = gtk::Box::new(gtk::Orientation::Vertical, 1);
        resting.add_css_class("lock-resting");
        resting.set_valign(gtk::Align::Center);
        let name = gtk::Label::new(Some(&display_name()));
        name.add_css_class("lock-name");
        name.set_xalign(0.0);
        let hint = gtk::Label::new(Some(t("digite sua senha")));
        hint.add_css_class("lock-hint");
        hint.set_xalign(0.0);
        resting.append(&name);
        resting.append(&hint);

        let field = gtk::Text::new();
        field.add_css_class("lock-field");
        field.set_visibility(false);
        field.set_invisible_char('●');
        field.set_hexpand(true);
        field.set_valign(gtk::Align::Center);
        field.set_opacity(0.0);

        // A hidden field cannot hold the keyboard, so it stays mapped,
        // invisible over the name until something is typed.
        let middle = gtk::Overlay::new();
        middle.set_hexpand(true);
        middle.set_valign(gtk::Align::Center);
        middle.set_child(Some(&resting));
        middle.add_overlay(&field);
        pill.append(&middle);

        let send = gtk::Button::new();
        send.add_css_class("lock-send");
        send.set_child(Some(&icons::icon("arrow_forward")));
        send.set_valign(gtk::Align::Center);
        send.set_opacity(0.0);
        send.set_can_target(false);
        pill.append(&send);
        center.append(&pill);

        let status = gtk::Label::new(None);
        status.add_css_class("lock-status");
        status.set_margin_top(12);
        center.append(&status);
        content.add_overlay(&center);

        content.add_overlay(&weather(s));
        content.add_overlay(&battery());
        content.add_overlay(&session());
        window.set_child(Some(&root));

        let surface = Rc::new(Self {
            window,
            background,
            content,
            shapes: [first, second],
            pill,
            resting,
            field,
            send: send.clone(),
            status,
            shown: Tween::new(0.0),
            width: Tween::new(RESTING_WIDTH as f64),
            shake: Tween::new(0.0),
            typing: Cell::new(false),
            length: Cell::new(0),
        });

        let submit: Rc<dyn Fn()> = {
            let weak = Rc::downgrade(&surface);
            Rc::new(move || {
                let Some(s) = weak.upgrade() else { return };
                let text = s.field.text().to_string();
                if !text.is_empty() {
                    on_submit(text);
                }
            })
        };
        {
            let again = submit.clone();
            surface.field.connect_activate(move |_| again());
            send.connect_clicked(move |_| submit());
        }
        {
            let weak = Rc::downgrade(&surface);
            surface.field.connect_changed(move |field| {
                let Some(s) = weak.upgrade() else { return };
                let length = field.text().chars().count();
                if length > 0 && length != s.length.get() {
                    s.adapt();
                }
                s.length.set(length);
                s.set_typing(length > 0);
            });
        }

        let weak = Rc::downgrade(&surface);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed(move |_, key, _, _| {
            let Some(s) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            if key == gdk::Key::Escape {
                s.field.set_text("");
                s.status.set_label("");
                return glib::Propagation::Stop;
            }
            // A key that lands before the field holds focus still counts.
            if !s.field.has_focus() {
                if let Some(c) = key.to_unicode().filter(|c| !c.is_control()) {
                    s.field.grab_focus_without_selecting();
                    let mut at = s.field.text().chars().count() as i32;
                    s.field.insert_text(&c.to_string(), &mut at);
                    s.field.set_position(at);
                    return glib::Propagation::Stop;
                }
                s.focus();
            }
            glib::Propagation::Proceed
        });
        surface.window.add_controller(keys);

        crate::shell::every_minute_for(&surface.window, move |now| {
            hours.set_label(&now.format("%H").to_string());
            minutes.set_label(&now.format("%M").to_string());
            date.set_label(&crate::i18n::date(&now));
        });

        surface
    }

    pub fn present(self: &Rc<Self>) {
        self.window.present();
        let content = self.content.clone();
        self.shown.animate(
            &self.content,
            1.0,
            600,
            anim::EMPHASIZED_DECELERATE,
            move |v| content.set_opacity(v),
            || {},
        );
        self.focus();
    }

    /// Focus without selecting: grabbing focus the usual way selects the
    /// whole field, so every key typed would replace the one before.
    fn focus(&self) {
        if !self.field.has_focus() {
            self.field.grab_focus_without_selecting();
        }
    }

    pub fn set_background(&self, texture: &gdk::Texture) {
        self.background.set_paintable(Some(texture));
    }

    /// At every key each shape adapts on its own, never into its neighbour.
    fn adapt(&self) {
        let mut taken = Vec::new();
        for shape in &self.shapes {
            taken.push(shape.adapt(&taken));
        }
    }

    fn set_typing(&self, typing: bool) {
        if self.typing.replace(typing) == typing {
            return;
        }
        self.field.set_opacity(if typing { 1.0 } else { 0.0 });
        self.resting.set_opacity(if typing { 0.0 } else { 1.0 });
        if typing {
            self.pill.add_css_class("typing");
            self.status.set_label("");
            self.pill.remove_css_class("failed");
        } else {
            self.pill.remove_css_class("typing");
        }
        self.send.set_opacity(if typing { 1.0 } else { 0.0 });
        self.send.set_can_target(typing);
        let pill = self.pill.clone();
        self.width.animate(
            &self.pill,
            if typing {
                TYPING_WIDTH as f64
            } else {
                RESTING_WIDTH as f64
            },
            380,
            anim::EMPHASIZED_DECELERATE,
            move |w| pill.set_size_request(w.round() as i32, 60),
            || {},
        );
    }

    pub fn busy(&self, on: bool) {
        self.field.set_sensitive(!on);
        self.send.set_sensitive(!on);
        self.status.remove_css_class("error");
        self.status
            .set_label(if on { t("verificando…") } else { "" });
    }

    pub fn fail(&self, message: &str) {
        self.busy(false);
        self.field.set_text("");
        self.focus();
        self.status.set_label(message);
        self.status.add_css_class("error");
        self.pill.add_css_class("failed");
        let pill = self.pill.clone();
        let shake = self.shake.clone();
        shake.set(0.0);
        let steps = Rc::new(RefCell::new(vec![-9.0, 9.0, -9.0, 9.0, 0.0]));
        shake_step(pill, shake, steps);
    }

    /// Fades everything out, then reports so the session can unlock.
    pub fn leave(&self, done: impl FnOnce() + 'static) {
        self.status.set_label("");
        let content = self.content.clone();
        self.shown.animate(
            &self.content,
            0.0,
            260,
            anim::EMPHASIZED_ACCELERATE,
            move |v| content.set_opacity(v),
            done,
        );
    }
}

fn shake_step(pill: gtk::Box, shake: Rc<Tween>, steps: Rc<RefCell<Vec<f64>>>) {
    if steps.borrow().is_empty() {
        return;
    }
    let to = steps.borrow_mut().remove(0);
    let (target, next_pill, next_shake) = (pill.clone(), pill.clone(), shake.clone());
    shake.animate(
        &pill,
        to,
        55,
        anim::STANDARD,
        move |x| {
            let x = x.round() as i32;
            target.set_margin_start((x * 2).max(0));
            target.set_margin_end((-x * 2).max(0));
        },
        move || shake_step(next_pill, next_shake, steps),
    );
}

fn display_name() -> String {
    let real = glib::real_name().to_string_lossy().into_owned();
    if real.is_empty() || real == "Unknown" {
        glib::user_name().to_string_lossy().into_owned()
    } else {
        real.split_whitespace().next().unwrap_or(&real).to_owned()
    }
}

fn avatar() -> gtk::Widget {
    let user = glib::user_name().to_string_lossy().into_owned();
    let candidates = [
        glib::home_dir().join(".face"),
        glib::home_dir().join(".face.icon"),
        std::path::PathBuf::from(format!("/var/lib/AccountsService/icons/{user}")),
    ];
    let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    holder.add_css_class("lock-avatar");
    holder.set_size_request(48, 48);
    holder.set_valign(gtk::Align::Center);
    holder.set_overflow(gtk::Overflow::Hidden);
    match candidates
        .iter()
        .find_map(|p| gdk::Texture::from_filename(p).ok())
    {
        Some(t) => {
            let image = gtk::Image::from_paintable(Some(&t));
            image.set_pixel_size(48);
            holder.append(&image);
        }
        None => {
            let initial = display_name()
                .chars()
                .next()
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_default();
            let label = gtk::Label::new(Some(&initial));
            label.set_valign(gtk::Align::Center);
            label.set_size_request(48, 48);
            holder.append(&label);
        }
    }
    holder.upcast()
}

/// The player, after Velatura's: the cover, title, artist, a progress line
/// with the times and the controls, with a few lines of the lyrics beside
/// it when there are any. Gone when nothing plays.
fn music(s: &Rc<Services>, enabled: bool, compact: Rc<dyn Fn(bool)>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 44);
    row.add_css_class("lock-music");
    row.set_halign(gtk::Align::Center);
    row.set_margin_top(26);
    row.set_visible(false);

    let player = gtk::Box::new(gtk::Orientation::Vertical, 8);
    player.set_size_request(COVER, -1);
    let picture = gtk::Picture::new();
    picture.set_content_fit(gtk::ContentFit::Cover);
    picture.set_can_shrink(true);
    let art = icons::icon("music_note");
    art.add_css_class("lock-art-fallback");
    let cover_content = gtk::Overlay::new();
    cover_content.set_child(Some(&art));
    cover_content.add_overlay(&picture);
    let cover = Shape::new(Kind::Rounded, COVER, true, Some(&cover_content));
    cover.add_css_class("lock-cover");
    player.append(&cover);

    let title = gtk::Label::new(None);
    title.add_css_class("lock-title");
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title.set_max_width_chars(1);
    title.set_hexpand(true);
    let artist = gtk::Label::new(None);
    artist.add_css_class("lock-artist");
    artist.set_ellipsize(gtk::pango::EllipsizeMode::End);
    artist.set_max_width_chars(1);
    let texts = gtk::Box::new(gtk::Orientation::Vertical, 1);
    texts.set_margin_top(4);
    texts.append(&title);
    texts.append(&artist);
    player.append(&texts);

    let progress = gtk::ProgressBar::new();
    progress.add_css_class("lock-progress");
    let at = gtk::Label::new(Some("0:00"));
    at.add_css_class("lock-time");
    let total = gtk::Label::new(Some("0:00"));
    total.add_css_class("lock-time");
    total.set_hexpand(true);
    total.set_xalign(1.0);
    let times = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    times.append(&at);
    times.append(&total);
    let line = gtk::Box::new(gtk::Orientation::Vertical, 4);
    line.append(&progress);
    line.append(&times);
    player.append(&line);

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    controls.set_halign(gtk::Align::Center);
    let previous = control("skip_previous");
    let play = control("pause");
    play.add_css_class("main");
    let next = control("skip_next");
    controls.append(&previous);
    controls.append(&play);
    controls.append(&next);
    player.append(&controls);
    {
        let m = s.media.clone();
        previous.connect_clicked(move |_| m.previous());
        let m = s.media.clone();
        play.connect_clicked(move |_| m.play_pause());
        let m = s.media.clone();
        next.connect_clicked(move |_| m.next());
    }
    row.append(&player);

    let sheet = gtk::Box::new(gtk::Orientation::Vertical, 10);
    sheet.add_css_class("lock-sheet");
    sheet.set_valign(gtk::Align::Center);
    sheet.set_size_request(360, -1);
    sheet.set_visible(false);
    let lines: Vec<gtk::Label> = (0..3)
        .map(|i| {
            let label = gtk::Label::new(None);
            label.add_css_class(if i == 1 { "lock-line" } else { "lock-line-dim" });
            label.set_xalign(0.0);
            label.set_wrap(true);
            label.set_max_width_chars(28);
            sheet.append(&label);
            label
        })
        .collect();
    row.append(&sheet);

    let current: Rc<RefCell<Option<Lyrics>>> = Rc::default();
    {
        let widgets = (
            row.downgrade(),
            picture.downgrade(),
            title.downgrade(),
            artist.downgrade(),
            total.downgrade(),
            play.downgrade(),
            sheet.downgrade(),
        );
        let current = current.clone();
        let last = RefCell::new(String::new());
        s.media.track.subscribe(move |track| {
            let (
                Some(row),
                Some(picture),
                Some(title),
                Some(artist),
                Some(total),
                Some(play),
                Some(sheet),
            ) = (
                widgets.0.upgrade(),
                widgets.1.upgrade(),
                widgets.2.upgrade(),
                widgets.3.upgrade(),
                widgets.4.upgrade(),
                widgets.5.upgrade(),
                widgets.6.upgrade(),
            )
            else {
                return;
            };
            let playing = track.is_some();
            if row.is_visible() != playing {
                row.set_visible(playing);
                compact(playing);
            }
            let Some(track) = track.as_ref() else { return };
            title.set_label(&track.title);
            artist.set_label(&track.artist);
            artist.set_visible(!track.artist.is_empty());
            total.set_label(&clock(track.length));
            if let Some(glyph) = play.child().and_downcast::<gtk::Label>() {
                glyph.set_label(if track.playing { "pause" } else { "play_arrow" });
            }
            picture.set_filename(
                track
                    .art
                    .as_deref()
                    .map(|p| p.strip_prefix("file://").unwrap_or(p)),
            );
            picture.set_visible(track.art.is_some());
            let key = format!("{}\n{}", track.title, track.artist);
            if !enabled || last.replace(key.clone()) == key {
                return;
            }
            current.borrow_mut().take();
            sheet.set_visible(false);
            let (track, current) = (track.clone(), current.clone());
            glib::spawn_future_local(async move {
                *current.borrow_mut() = lyrics::fetch(&track)
                    .await
                    .filter(|l| l.available && !l.instrumental && !l.lines.is_empty());
            });
        });
    }

    // Often enough for the words to light on time.
    let media = s.media.clone();
    let widgets = (
        row.downgrade(),
        progress.downgrade(),
        at.downgrade(),
        sheet.downgrade(),
    );
    glib::timeout_add_local(Duration::from_millis(80), move || {
        let (Some(row), Some(progress), Some(at), Some(sheet)) = (
            widgets.0.upgrade(),
            widgets.1.upgrade(),
            widgets.2.upgrade(),
            widgets.3.upgrade(),
        ) else {
            return glib::ControlFlow::Break;
        };
        if !row.is_visible() {
            return glib::ControlFlow::Continue;
        }
        let now = media.now();
        let length = media.track.get().as_ref().map(|t| t.length).unwrap_or(0.0);
        progress.set_fraction(if length > 0.0 {
            (now / length).clamp(0.0, 1.0)
        } else {
            0.0
        });
        at.set_label(&clock(now));
        let lyrics = current.borrow();
        sheet.set_visible(lyrics.is_some());
        if let Some(l) = lyrics.as_ref() {
            let index = l.current(now);
            for (slot, label) in lines.iter().enumerate() {
                let at = index
                    .and_then(|i| (i + slot).checked_sub(1))
                    .or(if slot == 2 { Some(0) } else { None });
                let text = at
                    .and_then(|i| l.lines.get(i))
                    .map(|line| line.text.as_str())
                    .unwrap_or("");
                if label.label() != text {
                    label.set_label(text);
                }
            }
        }
        glib::ControlFlow::Continue
    });
    row
}

fn control(glyph: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("lock-control");
    button.set_child(Some(&icons::icon(glyph)));
    button
}

fn clock(seconds: f64) -> String {
    let total = seconds.max(0.0) as u64;
    format!("{}:{:02}", total / 60, total % 60)
}

fn weather(s: &Rc<Services>) -> gtk::Box {
    let pill = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    pill.add_css_class("lock-weather");
    pill.set_halign(gtk::Align::Start);
    pill.set_valign(gtk::Align::End);
    pill.set_margin_start(24);
    pill.set_margin_bottom(24);
    let icon = icons::icon("cloud");
    let temperature = gtk::Label::new(None);
    pill.append(&icon);
    pill.append(&temperature);
    let (holder, icon, temperature) = (pill.downgrade(), icon.downgrade(), temperature.downgrade());
    s.weather.reading.subscribe(move |reading| {
        let (Some(holder), Some(icon), Some(temperature)) =
            (holder.upgrade(), icon.upgrade(), temperature.upgrade())
        else {
            return;
        };
        holder.set_visible(reading.is_some());
        if let Some(r) = reading.as_ref() {
            icon.set_label(&r.icon);
            temperature.set_label(&format!("{}°", r.temperature));
            holder.set_tooltip_text(Some(&r.description));
        }
    });
    pill
}

fn battery() -> gtk::Box {
    let holder = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    holder.add_css_class("lock-battery");
    holder.set_halign(gtk::Align::Center);
    holder.set_valign(gtk::Align::Start);
    holder.set_margin_top(20);
    let icon = icons::icon("battery_full");
    let text = gtk::Label::new(None);
    holder.append(&icon);
    holder.append(&text);
    let path = std::fs::read_dir("/sys/class/power_supply")
        .ok()
        .and_then(|dir| {
            dir.flatten().map(|e| e.path()).find(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("BAT"))
            })
        });
    let Some(path) = path else {
        holder.set_visible(false);
        return holder;
    };
    crate::shell::every_minute_for(&holder, move |_| {
        let capacity = std::fs::read_to_string(path.join("capacity"))
            .ok()
            .and_then(|c| c.trim().parse::<u32>().ok());
        let charging =
            std::fs::read_to_string(path.join("status")).is_ok_and(|s| s.trim() == "Charging");
        if let Some(c) = capacity {
            text.set_label(&format!("{c}%"));
            icon.set_label(if charging {
                "battery_charging_full"
            } else {
                match c {
                    0..=15 => "battery_alert",
                    16..=40 => "battery_3_bar",
                    41..=75 => "battery_5_bar",
                    _ => "battery_full",
                }
            });
        }
    });
    holder
}

/// Session buttons, top right. The first press arms one (it widens, turns
/// red and names itself), the second carries it out; it disarms after 3 s.
fn session() -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("lock-session");
    row.set_halign(gtk::Align::End);
    row.set_valign(gtk::Align::Start);
    row.set_margin_top(20);
    row.set_margin_end(20);
    let armed: Rc<RefCell<Option<(gtk::Button, glib::SourceId)>>> = Rc::default();
    let actions: [(&str, &str, &[&str]); 4] = [
        ("bedtime", t("suspender"), &["systemctl", "suspend"]),
        (
            "logout",
            t("sair"),
            &["niri", "msg", "action", "quit", "--skip-confirmation"],
        ),
        ("restart_alt", t("reiniciar"), &["systemctl", "reboot"]),
        (
            "power_settings_new",
            t("desligar"),
            &["systemctl", "poweroff"],
        ),
    ];
    for (icon, caption, args) in actions {
        let chip = gtk::Button::new();
        chip.add_css_class("lock-chip");
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 7);
        content.set_halign(gtk::Align::Center);
        content.append(&icons::icon(icon));
        let name = gtk::Label::new(Some(caption));
        name.add_css_class("lock-chip-name");
        let reveal = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideRight)
            .transition_duration(140)
            .child(&name)
            .build();
        content.append(&reveal);
        chip.set_child(Some(&content));
        let armed = armed.clone();
        chip.connect_clicked(move |chip| {
            let is_armed = armed.borrow().as_ref().is_some_and(|(b, _)| b == chip);
            if let Some((button, timer)) = armed.borrow_mut().take() {
                timer.remove();
                button.remove_css_class("armed");
                if let Some(r) = button
                    .child()
                    .and_then(|c| c.last_child())
                    .and_downcast::<gtk::Revealer>()
                {
                    r.set_reveal_child(false);
                }
            }
            if is_armed {
                crate::exec::spawn(args[0], &args[1..]);
                return;
            }
            chip.add_css_class("armed");
            reveal.set_reveal_child(true);
            let (slot, button, reveal) = (armed.clone(), chip.clone(), reveal.clone());
            let timer = glib::timeout_add_local_once(Duration::from_secs(3), move || {
                slot.borrow_mut().take();
                button.remove_css_class("armed");
                reveal.set_reveal_child(false);
            });
            *armed.borrow_mut() = Some((chip.clone(), timer));
        });
        row.append(&chip);
    }
    row
}
