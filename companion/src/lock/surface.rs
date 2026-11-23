use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::icons;
use crate::panels::lyrics::LyricsView;
use crate::services::Services;
use crate::widgets::anim::{self, Tween};
use crate::widgets::media;
use crate::widgets::shape::{Kind, Shape};

const IDLE: Duration = Duration::from_secs(20);

/// One monitor's lock: the blurred wallpaper, the clock, what is playing with
/// its lyrics, and the password at the bottom once something is touched.
pub struct Surface {
    pub window: gtk::Window,
    background: gtk::Picture,
    wash: gtk::Picture,
    content: gtk::Box,
    account: gtk::Box,
    entry: gtk::PasswordEntry,
    status: gtk::Label,
    lyrics: Rc<LyricsView>,
    shown: Rc<Tween>,
    awake: Rc<Tween>,
    idle: Cell<Option<glib::SourceId>>,
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
        background.add_css_class("lock-background");
        root.set_child(Some(&background));

        let wash = gtk::Picture::new();
        wash.set_content_fit(gtk::ContentFit::Cover);
        wash.set_opacity(0.0);
        wash.set_can_target(false);
        root.add_overlay(&wash);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.set_opacity(0.0);
        root.add_overlay(&content);

        let clock = gtk::Box::new(gtk::Orientation::Vertical, 0);
        clock.add_css_class("lock-clock");
        clock.set_halign(gtk::Align::Center);
        clock.set_margin_top(72);
        let date = gtk::Label::new(None);
        date.add_css_class("lock-date");
        let hours = gtk::Label::new(None);
        hours.add_css_class("lock-hours");
        let minutes = gtk::Label::new(None);
        minutes.add_css_class("lock-minutes");
        clock.append(&date);
        clock.append(&hours);
        clock.append(&minutes);
        content.append(&clock);

        let music = gtk::Box::new(gtk::Orientation::Horizontal, 56);
        music.add_css_class("lock-music");
        music.set_halign(gtk::Align::Center);
        music.set_valign(gtk::Align::Center);
        music.set_vexpand(true);
        let player = media::player(&s.media, media::Size::Hero, None);
        player.set_valign(gtk::Align::Center);
        music.append(&player);
        let lyrics = LyricsView::new(&s.media, lyrics);
        lyrics.root.set_size_request(500, 380);
        lyrics.root.add_css_class("lock-lyrics");
        music.append(&lyrics.root);
        let spacer = gtk::Box::new(gtk::Orientation::Vertical, 0);
        spacer.set_vexpand(true);
        content.append(&music);
        content.append(&spacer);

        let account = gtk::Box::new(gtk::Orientation::Vertical, 8);
        account.set_halign(gtk::Align::Center);
        account.set_margin_bottom(56);
        account.set_opacity(0.0);
        let pill = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        pill.add_css_class("lock-account");
        pill.append(&avatar());
        let entry = gtk::PasswordEntry::new();
        entry.set_hexpand(true);
        entry.set_property(
            "placeholder-text",
            format!("oi, {}", display_name()).to_lowercase(),
        );
        pill.append(&entry);
        let go = gtk::Button::new();
        go.add_css_class("lock-go");
        go.set_child(Some(&Shape::new(
            Kind::Cookie6,
            40,
            true,
            Some(&icons::icon("arrow_forward")),
        )));
        pill.append(&go);
        let status = gtk::Label::new(None);
        status.add_css_class("lock-status");
        account.append(&pill);
        account.append(&status);
        content.append(&account);

        root.add_overlay(&power(s));
        root.add_overlay(&battery());
        window.set_child(Some(&root));

        let surface = Rc::new(Self {
            window,
            background,
            wash,
            content,
            account,
            entry,
            status,
            lyrics,
            shown: Tween::new(0.0),
            awake: Tween::new(0.0),
            idle: Cell::new(None),
        });

        {
            let submit = {
                let (entry, on_submit) = (surface.entry.clone(), on_submit.clone());
                move || {
                    let text = entry.text().to_string();
                    if !text.is_empty() {
                        on_submit(text);
                    }
                }
            };
            let submit = Rc::new(submit);
            let again = submit.clone();
            surface.entry.connect_activate(move |_| again());
            go.connect_clicked(move |_| submit());
        }

        let weak = Rc::downgrade(&surface);
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        keys.connect_key_pressed(move |_, _, _, _| {
            if let Some(s) = weak.upgrade() {
                s.wake();
            }
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

        {
            let (hours, minutes, date) = (hours.clone(), minutes.clone(), date.clone());
            crate::shell::every_minute_for(&surface.window, move |now| {
                hours.set_label(&now.format("%H").to_string());
                minutes.set_label(&now.format("%M").to_string());
                date.set_label(
                    &now.format_localized("%A, %-d de %B", chrono::Locale::pt_BR)
                        .to_string(),
                );
            });
        }

        let weak = Rc::downgrade(&surface);
        s.media.track.subscribe(move |track| {
            let Some(surface) = weak.upgrade() else {
                return;
            };
            let playing = track.is_some();
            music.set_visible(playing);
            spacer.set_visible(!playing);
            if playing {
                clock.add_css_class("compact");
            } else {
                clock.remove_css_class("compact");
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
        self.entry.grab_focus();
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
                gtk::gio::spawn_blocking(move || super::blur::blurred(&path, 64, 3, 0.55, 0.7))
                    .await
                    .ok()
                    .flatten();
            wash.set_paintable(texture.as_ref());
            wash.set_opacity(if texture.is_some() { 0.85 } else { 0.0 });
        });
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
                    s.sleep();
                }
            })));
        if self.awake.get() < 1.0 {
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
        self.entry.grab_focus();
    }

    fn sleep(&self) {
        self.entry.set_text("");
        self.status.set_label("");
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
        self.entry.set_sensitive(!on);
        self.status.set_label(if on { "verificando…" } else { "" });
    }

    pub fn fail(&self, message: &str) {
        self.busy(false);
        self.entry.set_text("");
        self.entry.grab_focus();
        self.status.set_label(message);
        self.account.add_css_class("shake");
        let account = self.account.clone();
        glib::timeout_add_local_once(Duration::from_millis(420), move || {
            account.remove_css_class("shake")
        });
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
    let texture = candidates
        .iter()
        .find_map(|p| gdk::Texture::from_filename(p).ok());
    let child: gtk::Widget = match texture {
        Some(t) => {
            let image = gtk::Image::from_paintable(Some(&t));
            image.set_pixel_size(44);
            image.upcast()
        }
        None => icons::icon("person").upcast(),
    };
    let shape = Shape::new(Kind::Cookie9, 44, true, Some(&child));
    shape.add_css_class("lock-avatar");
    shape.upcast()
}

fn power(_s: &Rc<Services>) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    row.add_css_class("lock-power");
    row.set_halign(gtk::Align::Start);
    row.set_valign(gtk::Align::End);
    row.set_margin_start(28);
    row.set_margin_bottom(28);
    for (icon, tip, args) in [
        ("bedtime", "suspender", ["systemctl", "suspend"]),
        ("restart_alt", "reiniciar", ["systemctl", "reboot"]),
        ("power_settings_new", "desligar", ["systemctl", "poweroff"]),
    ] {
        let button = icons::button(icon, tip);
        button.connect_clicked(move |_| crate::exec::spawn(args[0], &args[1..]));
        row.append(&button);
    }
    row
}

fn battery() -> gtk::Box {
    let holder = gtk::Box::new(gtk::Orientation::Vertical, 0);
    holder.add_css_class("lock-battery");
    holder.set_halign(gtk::Align::End);
    holder.set_valign(gtk::Align::Start);
    holder.set_margin_top(28);
    holder.set_margin_end(28);
    let level = gtk::LevelBar::for_interval(0.0, 100.0);
    level.set_orientation(gtk::Orientation::Vertical);
    level.set_inverted(true);
    level.set_size_request(20, 44);
    holder.append(&level);
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
    let read = move || {
        let capacity = std::fs::read_to_string(path.join("capacity"))
            .ok()?
            .trim()
            .parse::<f64>()
            .ok()?;
        let charging =
            std::fs::read_to_string(path.join("status")).is_ok_and(|s| s.trim() == "Charging");
        Some((capacity, charging))
    };
    let update = {
        let level = level.clone();
        move || {
            if let Some((capacity, charging)) = read() {
                level.set_value(capacity);
                level.set_tooltip_text(Some(&format!("{capacity}%")));
                if charging {
                    level.add_css_class("charging");
                } else {
                    level.remove_css_class("charging");
                }
            }
        }
    };
    crate::shell::every_minute_for(&holder, move |_| update());
    holder
}
