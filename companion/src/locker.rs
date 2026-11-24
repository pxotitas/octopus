use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

use gtk::{gio, glib};

use crate::exec;
use crate::i18n::t;
use crate::services::lyrics::{self, Lyrics};
use crate::services::media::{Media, Track};

/// Locks with hyprlock and feeds its labels: what is playing, the lyric
/// being sung, the date and the battery, through `octopus now <field>`.
pub struct Locker {
    media: Rc<Media>,
    lyrics: RefCell<Option<(String, Option<Lyrics>)>>,
    fetching: Cell<bool>,
    running: Cell<bool>,
    enabled: bool,
}

fn dir() -> PathBuf {
    exec::cache_dir().join("lock")
}

fn key(track: &Track) -> String {
    format!("{}\n{}\n{}", track.player, track.artist, track.title)
}

fn escape(text: &str) -> String {
    glib::markup_escape_text(text).to_string()
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let cut: String = text.chars().take(max - 1).collect();
    format!("{}…", cut.trim_end())
}

impl Locker {
    pub fn new(media: &Rc<Media>, lyrics: bool) -> Rc<Self> {
        Rc::new(Self {
            media: media.clone(),
            lyrics: RefCell::new(None),
            fetching: Cell::new(false),
            running: Cell::new(false),
            enabled: lyrics,
        })
    }

    pub fn start(self: &Rc<Self>) {
        if self.running.replace(true) {
            return;
        }
        let dir = dir();
        let _ = std::fs::create_dir_all(&dir);
        let strings = format!(
            "$placeholder = {}\n$fail = {}\n",
            t("digite sua senha"),
            t("senha incorreta")
        );
        let _ = std::fs::write(dir.join("strings.conf"), strings);
        self.media.watch(true);

        let config = exec::config_dir().join("hyprlock.conf");
        let prepare = exec::script("lock.py");
        let command = format!(
            "[ -f '{d}/colors.conf' ] || python3 '{}'; exec hyprlock -c '{}' 2>'{d}/hyprlock.log'",
            prepare,
            config.display(),
            d = dir.display()
        );
        let child = gio::Subprocess::newv(
            &["sh".as_ref(), "-c".as_ref(), command.as_str().as_ref()],
            gio::SubprocessFlags::NONE,
        );
        let this = Rc::downgrade(self);
        let finish = move || {
            if let Some(this) = this.upgrade() {
                this.running.set(false);
                this.media.watch(false);
                crate::memory::release_soon();
            }
        };
        match child {
            Ok(child) => child.wait_async(gio::Cancellable::NONE, move |_| finish()),
            Err(err) => {
                eprintln!("octopus: cannot start hyprlock: {err}");
                finish();
            }
        }
    }

    pub fn field(self: &Rc<Self>, name: &str) -> String {
        match name {
            "date" => crate::i18n::date(&chrono::Local::now()),
            "battery" => battery(),
            "art" => self
                .media
                .track
                .get()
                .as_ref()
                .and_then(|t| t.art.clone())
                .map(|p| p.strip_prefix("file://").unwrap_or(&p).to_owned())
                .unwrap_or_else(|| dir().join("blank.png").to_string_lossy().into_owned()),
            "song" => match self.media.track.get().as_ref() {
                Some(t) if !t.title.is_empty() => {
                    let song = if t.artist.is_empty() {
                        t.title.clone()
                    } else {
                        format!("{} · {}", t.title, t.artist)
                    };
                    escape(&clip(&song, 56))
                }
                _ => String::new(),
            },
            "line" => self.line(),
            _ => String::new(),
        }
    }

    fn line(self: &Rc<Self>) -> String {
        let Some(track) = self.media.track.get().clone() else {
            return String::new();
        };
        if !self.enabled {
            return String::new();
        }
        let key = key(&track);
        let known = self.lyrics.borrow().as_ref().map(|(k, _)| k == &key);
        if known != Some(true) {
            self.fetch(track, key);
            return String::new();
        }
        let position = self.media.now();
        let lyrics = self.lyrics.borrow();
        let Some((_, Some(lyrics))) = lyrics.as_ref() else {
            return String::new();
        };
        if lyrics.instrumental {
            return "♪".into();
        }
        lyrics
            .current(position)
            .map(|i| escape(&clip(&lyrics.lines[i].text, 60)))
            .unwrap_or_default()
    }

    fn fetch(self: &Rc<Self>, track: Track, key: String) {
        if self.fetching.replace(true) {
            return;
        }
        let this = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let result = lyrics::fetch(&track).await.filter(|l| l.available);
            if let Some(this) = this.upgrade() {
                *this.lyrics.borrow_mut() = Some((key, result));
                this.fetching.set(false);
            }
        });
    }
}

fn battery() -> String {
    let Some(path) = std::fs::read_dir("/sys/class/power_supply")
        .ok()
        .and_then(|dir| {
            dir.flatten().map(|e| e.path()).find(|p| {
                p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("BAT"))
            })
        })
    else {
        return String::new();
    };
    let read = |name: &str| std::fs::read_to_string(path.join(name)).unwrap_or_default();
    let capacity: u32 = read("capacity").trim().parse().unwrap_or(0);
    let charging = matches!(read("status").trim(), "Charging" | "Full");
    let glyph = if charging {
        "battery_charging_full".to_owned()
    } else if capacity >= 95 {
        "battery_full".to_owned()
    } else {
        format!("battery_{}_bar", (capacity / 15).min(6))
    };
    format!(
        "<span font_family=\"Material Symbols Rounded\" size=\"larger\" rise=\"-3pt\">{glyph}</span>  {capacity}%"
    )
}
