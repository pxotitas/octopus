use std::cell::{Cell, RefCell};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Track {
    pub player: String,
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// A local file, once remote art has been downloaded.
    pub art: Option<String>,
    pub length: f64,
}

pub struct Media {
    pub track: Rc<Store<Option<Track>>>,
    /// Seconds into the track as last read from the player; polled only
    /// while something shows it. `now()` runs it forward between reads.
    pub position: Rc<Store<f64>>,
    read_at: Rc<Cell<i64>>,
    watchers: Cell<u32>,
    ticker: RefCell<Option<glib::SourceId>>,
    _events: Option<exec::Stream>,
}

const FORMAT: &str = "{{playerName}}\t{{status}}\t{{xesam:title}}\t{{xesam:artist}}\t{{xesam:album}}\t{{mpris:artUrl}}\t{{mpris:length}}";

impl Media {
    pub fn new() -> Rc<Self> {
        let track: Rc<Store<Option<Track>>> = Store::new(None);
        let events = exec::stream(
            "playerctl",
            &["--follow", "metadata", "--format", FORMAT],
            {
                let track = track.clone();
                move |line| apply(&track, &line)
            },
        );
        let media = Rc::new(Self {
            track,
            position: Store::new(0.0),
            read_at: Rc::new(Cell::new(glib::monotonic_time())),
            watchers: Cell::new(0),
            ticker: RefCell::new(None),
            _events: events,
        });
        let weak = Rc::downgrade(&media);
        media.track.subscribe(move |_| {
            if let Some(media) = weak.upgrade() {
                media.poll();
            }
        });
        media
    }

    /// Panels showing progress call this while open, so nothing polls when
    /// nobody is looking.
    pub fn watch(self: &Rc<Self>, on: bool) {
        let count = if on {
            self.watchers.get() + 1
        } else {
            self.watchers.get().saturating_sub(1)
        };
        self.watchers.set(count);
        if count > 0 && self.ticker.borrow().is_none() {
            self.poll();
            let this = Rc::downgrade(self);
            let id = glib::timeout_add_local(Duration::from_millis(1000), move || {
                match this.upgrade() {
                    Some(this) => {
                        this.poll();
                        glib::ControlFlow::Continue
                    }
                    None => glib::ControlFlow::Break,
                }
            });
            *self.ticker.borrow_mut() = Some(id);
        } else if count == 0 {
            if let Some(id) = self.ticker.borrow_mut().take() {
                id.remove();
            }
        }
    }

    fn poll(&self) {
        let Some(player) = self.track.get().as_ref().map(|t| t.player.clone()) else {
            return;
        };
        let position = self.position.clone();
        let read_at = self.read_at.clone();
        glib::spawn_future_local(async move {
            if let Some(out) = exec::output("playerctl", &["-p", &player, "position"]).await {
                if let Ok(seconds) = out.trim().parse::<f64>() {
                    read_at.set(glib::monotonic_time());
                    position.set(seconds);
                }
            }
        });
    }

    /// The playing position right now, without asking the player.
    pub fn now(&self) -> f64 {
        let base = *self.position.get();
        match self.track.get().as_ref() {
            Some(t) if t.playing => {
                let elapsed = (glib::monotonic_time() - self.read_at.get()) as f64 / 1_000_000.0;
                let now = base + elapsed;
                if t.length > 0.0 {
                    now.min(t.length)
                } else {
                    now
                }
            }
            _ => base,
        }
    }

    fn command(&self, action: &str) {
        match self.track.get().as_ref() {
            Some(t) => exec::spawn("playerctl", &["-p", &t.player, action]),
            None => exec::spawn("playerctl", &[action]),
        }
    }

    pub fn play_pause(&self) {
        self.command("play-pause");
    }

    pub fn next(&self) {
        self.command("next");
    }

    pub fn previous(&self) {
        self.command("previous");
    }

    pub fn seek(&self, seconds: f64) {
        if let Some(t) = self.track.get().as_ref() {
            exec::spawn(
                "playerctl",
                &["-p", &t.player, "position", &format!("{seconds:.1}")],
            );
        }
        self.position.set(seconds);
    }
}

fn apply(store: &Rc<Store<Option<Track>>>, line: &str) {
    let f: Vec<&str> = line.split('\t').collect();
    if f.len() < 7 || f[2].is_empty() && f[1].is_empty() {
        store.set(None);
        return;
    }
    let art_url = f[5].to_owned();
    let previous_art = store
        .get()
        .as_ref()
        .filter(|t| t.title == f[2])
        .and_then(|t| t.art.clone());
    let track = Track {
        player: f[0].to_owned(),
        playing: f[1] == "Playing",
        title: f[2].to_owned(),
        artist: f[3].to_owned(),
        album: f[4].to_owned(),
        art: previous_art,
        length: f[6]
            .parse::<f64>()
            .map(|us| us / 1_000_000.0)
            .unwrap_or(0.0),
    };
    if *store.get() == Some(track.clone()) {
        return;
    }
    let title = track.title.clone();
    store.set(Some(track));
    resolve_art(store.clone(), title, art_url);
}

fn resolve_art(store: Rc<Store<Option<Track>>>, title: String, url: String) {
    glib::spawn_future_local(async move {
        let path = if let Some(path) = url.strip_prefix("file://") {
            Some(
                glib::Uri::unescape_string(path, None::<&str>)
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| path.to_owned()),
            )
        } else if url.starts_with("http") {
            download(&url).await
        } else {
            None
        };
        store.update(|track| {
            if let Some(t) = track.as_mut().filter(|t| t.title == title) {
                t.art = path;
            }
        });
    });
}

async fn download(url: &str) -> Option<String> {
    let dir = exec::cache_dir().join("art");
    std::fs::create_dir_all(&dir).ok()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hasher);
    let path = dir.join(format!("{:x}", hasher.finish()));
    let path_str = path.to_string_lossy().into_owned();
    if !path.exists() {
        exec::output("curl", &["-sfL", "--max-time", "10", "-o", &path_str, url]).await?;
    }
    Some(path_str)
}
