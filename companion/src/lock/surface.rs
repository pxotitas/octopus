use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::icons;
use crate::panels::lyrics::LyricsView;
use crate::services::Services;
use crate::widgets::anim::{self, Tween};
use crate::widgets::media;

const IDLE: Duration = Duration::from_secs(20);
const RESTING_WIDTH: i32 = 300;
const TYPING_WIDTH: i32 = 380;

/// One monitor's lock, after Velatura's: the blurred, greyed wallpaper, a big
/// stacked clock, what is playing with its lyrics, and a black pill at the
/// bottom that turns into the password field once something is typed.
pub struct Surface {
    pub window: gtk::Window,
    background: gtk::Picture,
    wash: gtk::Picture,
    content: gtk::Box,
    clock: gtk::Box,
    account: gtk::Box,
    pill: gtk::Box,
    resting: gtk::Box,
    field: gtk::Text,
    send: gtk::Button,
    status: gtk::Label,
    lyrics: Rc<LyricsView>,
    shown: Rc<Tween>,
    awake: Rc<Tween>,
    width: Rc<Tween>,
    shake: Rc<Tween>,
    idle: Cell<Option<glib::SourceId>>,
    typing: Cell<bool>,
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

        let wash = gtk::Picture::new();
        wash.set_content_fit(gtk::ContentFit::Cover);
        wash.set_opacity(0.0);
        wash.set_can_target(false);
        root.add_overlay(&wash);

        let scrim = gtk::Box::new(gtk::Orientation::Vertical, 0);
        scrim.add_css_class("lock-scrim");
        scrim.set_can_target(false);
        root.add_overlay(&scrim);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.set_opacity(0.0);
        root.add_overlay(&content);

        let clock = gtk::Box::new(gtk::Orientation::Vertical, 0);
        clock.add_css_class("lock-clock");
        clock.set_halign(gtk::Align::Center);
        let date = gtk::Label::new(None);
        date.add_css_class("lock-date");
        let hours = gtk::Label::new(None);
        hours.add_css_class("lock-hours");
        let minutes = gtk::Label::new(None);
        minutes.add_css_class("lock-minutes");
        clock.append(&date);
        clock.append(&hours);
        clock.append(&minutes);

        let top = gtk::Box::new(gtk::Orientation::Vertical, 0);
        top.set_vexpand(true);
        top.set_valign(gtk::Align::Center);
        top.append(&clock);

        let music = gtk::Box::new(gtk::Orientation::Horizontal, 56);
        music.add_css_class("lock-music");
        music.set_halign(gtk::Align::Center);
        music.set_margin_top(28);
        let player = media::player(&s.media, media::Size::Hero, None);
        player.set_valign(gtk::Align::Center);
        music.append(&player);
        let lyrics = LyricsView::new(&s.media, lyrics);
        lyrics.root.set_size_request(500, 380);
        lyrics.root.add_css_class("lock-lyrics");
        music.append(&lyrics.root);
        top.append(&music);
        content.append(&top);

        let account = gtk::Box::new(gtk::Orientation::Vertical, 14);
        account.set_halign(gtk::Align::Center);
        account.set_margin_bottom(60);
        account.set_opacity(0.0);
        let pill = gtk::Box::new(gtk::Orientation::Horizontal, 14);
        pill.add_css_class("lock-pill");
        pill.set_halign(gtk::Align::Center);
        pill.set_size_request(RESTING_WIDTH, 60);
        pill.append(&avatar());

        let resting = gtk::Box::new(gtk::Orientation::Vertical, 1);
        resting.add_css_class("lock-resting");
        resting.set_valign(gtk::Align::Center);
        let name = gtk::Label::new(Some(&display_name()));
        name.add_css_class("lock-name");
        name.set_xalign(0.0);
        let hint = gtk::Label::new(Some("digite sua senha"));
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

        let middle = gtk::Stack::new();
        middle.set_transition_type(gtk::StackTransitionType::Crossfade);
        middle.set_transition_duration(140);
        middle.set_hhomogeneous(false);
        middle.set_vhomogeneous(false);
        middle.set_valign(gtk::Align::Center);
        middle.set_hexpand(true);
        middle.add_named(&resting, Some("resting"));
        middle.add_named(&field, Some("typing"));
        pill.append(&middle);

        let send = gtk::Button::new();
        send.add_css_class("lock-send");
        send.set_child(Some(&icons::icon("arrow_forward")));
        send.set_valign(gtk::Align::Center);
        send.set_opacity(0.0);
        send.set_can_target(false);
        pill.append(&send);

        let status = gtk::Label::new(None);
        status.add_css_class("lock-status");
        account.append(&pill);
        account.append(&status);
        content.append(&account);

        root.add_overlay(&power());
        root.add_overlay(&battery());
        window.set_child(Some(&root));

        let surface = Rc::new(Self {
            window,
            background,
            wash,
            content,
            clock: clock.clone(),
            account,
            pill,
            resting,
            field,
            send: send.clone(),
            status,
            lyrics,
            shown: Tween::new(0.0),
            awake: Tween::new(0.0),
            width: Tween::new(RESTING_WIDTH as f64),
            shake: Tween::new(0.0),
            idle: Cell::new(None),
            typing: Cell::new(false),
        });

        let submit: Rc<dyn Fn()> = {
            let weak = Rc::downgrade(&surface);
            let on_submit = on_submit.clone();
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
            let middle = middle.clone();
            surface.field.connect_changed(move |field| {
                if let Some(s) = weak.upgrade() {
                    s.set_typing(!field.text().is_empty(), &middle);
                }
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
                s.sleep();
                return glib::Propagation::Stop;
            }
            s.wake();
            glib::Propagation::Proceed
        });
        surface.window.add_controller(keys);

        let weak = Rc::downgrade(&surface);
        let pointer = gtk::EventControllerMotion::new();
        let last = Cell::new((f64::NAN, f64::NAN));
        pointer.connect_motion(move |_, x, y| {
            let (lx, ly) = last.replace((x, y));
            if !lx.is_nan() && (x - lx).abs() + (y - ly).abs() > 4.0 {
                if let Some(s) = weak.upgrade() {
                    s.wake();
                }
            }
        });
        surface.window.add_controller(pointer);

        crate::shell::every_minute_for(&surface.window, move |now| {
            hours.set_label(&now.format("%H").to_string());
            minutes.set_label(&now.format("%M").to_string());
            date.set_label(
                &now.format_localized("%A, %-d de %B", chrono::Locale::pt_BR)
                    .to_string(),
            );
        });

        let weak = Rc::downgrade(&surface);
        s.media.track.subscribe(move |track| {
            let Some(surface) = weak.upgrade() else {
                return;
            };
            let playing = track.is_some();
            music.set_visible(playing);
            if playing {
                surface.clock.add_css_class("compact");
            } else {
                surface.clock.remove_css_class("compact");
            }
            surface.set_cover(track.as_ref().and_then(|t| t.art.clone()));
        });

        surface
    }

    pub fn present(self: &Rc<Self>) {
        self.window.present();
        self.lyrics.set_active(true);
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

    fn set_cover(&self, path: Option<String>) {
        let wash = self.wash.clone();
        let Some(path) = path else {
            wash.set_opacity(0.0);
            return;
        };
        glib::spawn_future_local(async move {
            let texture =
                gtk::gio::spawn_blocking(move || super::blur::blurred(&path, 64, 3, 0.55, 0.6))
                    .await
                    .ok()
                    .flatten();
            wash.set_paintable(texture.as_ref());
            wash.set_opacity(if texture.is_some() { 0.9 } else { 0.0 });
        });
    }

    fn set_typing(&self, typing: bool, middle: &gtk::Stack) {
        if self.typing.replace(typing) == typing {
            return;
        }
        middle.set_visible_child_name(if typing { "typing" } else { "resting" });
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
        let _ = &self.resting;
    }

    fn wake(self: &Rc<Self>) {
        if let Some(id) = self.idle.take() {
            id.remove();
        }
        let weak = Rc::downgrade(self);
        self.idle
            .set(Some(glib::timeout_add_local_once(IDLE, move || {
                if let Some(s) = weak.upgrade() {
                    s.idle.set(None);
                    if s.field.text().is_empty() {
                        s.sleep();
                    }
                }
            })));
        if self.awake.get() < 1.0 {
            self.clock.add_css_class("awake");
            let account = self.account.clone();
            self.awake.animate(
                &self.account,
                1.0,
                380,
                anim::EMPHASIZED_DECELERATE,
                move |v| account.set_opacity(v),
                || {},
            );
        }
        self.focus();
    }

    fn sleep(&self) {
        self.status.set_label("");
        self.clock.remove_css_class("awake");
        let account = self.account.clone();
        self.awake.animate(
            &self.account,
            0.0,
            300,
            anim::EMPHASIZED_ACCELERATE,
            move |v| account.set_opacity(v),
            || {},
        );
    }

    pub fn busy(&self, on: bool) {
        self.field.set_sensitive(!on);
        self.send.set_sensitive(!on);
        if on {
            self.status.set_label("verificando…");
        }
    }

    pub fn fail(&self, message: &str) {
        self.busy(false);
        self.field.set_text("");
        self.focus();
        self.status.set_label(message);
        self.pill.add_css_class("failed");
        let pill = self.pill.clone();
        let shake = self.shake.clone();
        shake.set(0.0);
        let steps = Rc::new(RefCell::new(vec![-9.0, 9.0, -9.0, 9.0, 0.0]));
        shake_step(pill, shake, steps);
    }

    /// Fades everything out, then reports so the session can unlock.
    pub fn leave(&self, done: impl FnOnce() + 'static) {
        self.lyrics.set_active(false);
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
            label.set_vexpand(true);
            holder.append(&label);
        }
    }
    holder.upcast()
}

/// Velatura's chips: the first press arms one (it widens, turns red and
/// names itself), the second carries it out; it disarms after 3 s.
fn power() -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("lock-power");
    row.set_halign(gtk::Align::Start);
    row.set_valign(gtk::Align::End);
    row.set_margin_start(24);
    row.set_margin_bottom(24);
    let armed: Rc<RefCell<Option<(gtk::Button, glib::SourceId)>>> = Rc::default();
    for (icon, caption, args) in [
        ("bedtime", "suspender", ["systemctl", "suspend"]),
        ("restart_alt", "reiniciar", ["systemctl", "reboot"]),
        ("power_settings_new", "desligar", ["systemctl", "poweroff"]),
    ] {
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

fn battery() -> gtk::Box {
    let holder = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    holder.add_css_class("lock-battery");
    holder.set_halign(gtk::Align::End);
    holder.set_valign(gtk::Align::Start);
    holder.set_margin_top(20);
    holder.set_margin_end(20);
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
