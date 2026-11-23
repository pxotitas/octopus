use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::services::lyrics::{self, Lyrics};
use crate::services::media::{Media, Track};
use crate::widgets;

/// Synced lyrics that follow the playing line; plain lyrics just scroll.
pub struct LyricsView {
    pub root: gtk::Stack,
    column: gtk::Box,
    scroller: gtk::ScrolledWindow,
    message: gtk::Label,
    lyrics: RefCell<Option<Lyrics>>,
    loaded_for: RefCell<String>,
    current: Cell<Option<usize>>,
    active: Cell<bool>,
    enabled: bool,
    media: Rc<Media>,
}

impl LyricsView {
    pub fn new(media: &Rc<Media>, enabled: bool) -> Rc<Self> {
        let root = gtk::Stack::new();
        root.set_transition_type(gtk::StackTransitionType::Crossfade);
        root.add_css_class("lyrics");
        root.set_hexpand(true);
        root.set_vexpand(true);

        let message = gtk::Label::new(None);
        message.add_css_class("dim");
        root.add_named(&message, Some("message"));

        let column = gtk::Box::new(gtk::Orientation::Vertical, 10);
        column.set_margin_top(140);
        column.set_margin_bottom(180);
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&column)
            .vexpand(true)
            .build();
        root.add_named(&scroller, Some("lines"));

        let view = Rc::new(Self {
            root,
            column,
            scroller,
            message,
            lyrics: RefCell::new(None),
            loaded_for: RefCell::new(String::new()),
            current: Cell::new(None),
            active: Cell::new(false),
            enabled,
            media: media.clone(),
        });

        let weak = Rc::downgrade(&view);
        media.track.subscribe(move |track| {
            if let Some(v) = weak.upgrade() {
                v.load(track.as_ref());
            }
        });
        let weak = Rc::downgrade(&view);
        media.position.subscribe(move |position| {
            if let Some(v) = weak.upgrade() {
                v.follow(*position);
            }
        });
        view
    }

    /// Fetches only while the media tab is on screen.
    pub fn set_active(self: &Rc<Self>, active: bool) {
        self.active.set(active);
        if active {
            let track = self.media.track.get().clone();
            self.load(track.as_ref());
        }
    }

    fn say(&self, text: &str) {
        self.message.set_label(text);
        self.root.set_visible_child_name("message");
    }

    fn load(self: &Rc<Self>, track: Option<&Track>) {
        let Some(track) = track.cloned() else {
            self.loaded_for.borrow_mut().clear();
            self.lyrics.borrow_mut().take();
            self.say("nada tocando");
            return;
        };
        if !self.enabled {
            self.say("letras desligadas no octopus.toml");
            return;
        }
        let key = format!("{}\n{}", track.artist, track.title);
        if !self.active.get() || *self.loaded_for.borrow() == key {
            return;
        }
        *self.loaded_for.borrow_mut() = key.clone();
        self.lyrics.borrow_mut().take();
        self.current.set(None);
        self.say("procurando a letra…");

        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let result = lyrics::fetch(&track).await;
            let Some(view) = weak.upgrade() else { return };
            if *view.loaded_for.borrow() != key {
                return;
            }
            view.render(result);
        });
    }

    fn render(&self, result: Option<Lyrics>) {
        widgets::clear(&self.column);
        let lyrics = match result {
            Some(l) if l.available && l.instrumental => return self.say("instrumental"),
            Some(l) if l.available && !l.lines.is_empty() => l,
            Some(_) => return self.say("sem letra pra essa música"),
            None => {
                self.loaded_for.borrow_mut().clear();
                return self.say("não deu pra buscar a letra");
            }
        };
        for line in &lyrics.lines {
            let label = gtk::Label::new(Some(&line.text));
            label.add_css_class("lyric");
            label.set_wrap(true);
            label.set_xalign(0.0);
            self.column.append(&label);
        }
        if !lyrics.synced {
            self.column.set_margin_top(8);
            self.column.set_margin_bottom(8);
            self.column.add_css_class("plain");
        } else {
            self.column.set_margin_top(140);
            self.column.set_margin_bottom(180);
            self.column.remove_css_class("plain");
        }
        self.scroller.vadjustment().set_value(0.0);
        *self.lyrics.borrow_mut() = Some(lyrics);
        self.root.set_visible_child_name("lines");
        self.follow(*self.media.position.get());
    }

    fn follow(&self, position: f64) {
        let index = self
            .lyrics
            .borrow()
            .as_ref()
            .and_then(|l| l.current(position));
        let Some(index) = index else { return };
        if self.current.get() == Some(index) {
            return;
        }
        if let Some(old) = self.current.get().and_then(|i| nth(&self.column, i)) {
            old.remove_css_class("current");
        }
        self.current.set(Some(index));
        let Some(line) = nth(&self.column, index) else {
            return;
        };
        line.add_css_class("current");
        if let Some(bounds) = line.compute_bounds(&self.column) {
            let adj = self.scroller.vadjustment();
            let target = bounds.y() as f64 + bounds.height() as f64 / 2.0 - adj.page_size() / 2.0;
            animate(&adj, target);
        }
    }
}

fn nth(column: &gtk::Box, index: usize) -> Option<gtk::Widget> {
    let mut child = column.first_child();
    for _ in 0..index {
        child = child?.next_sibling();
    }
    child
}

fn animate(adj: &gtk::Adjustment, target: f64) {
    let start = adj.value();
    let target = target.clamp(
        adj.lower(),
        (adj.upper() - adj.page_size()).max(adj.lower()),
    );
    let adj = adj.clone();
    let step = Rc::new(Cell::new(0u32));
    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        let n = step.get() + 1;
        step.set(n);
        let t = (n as f64 / 18.0).min(1.0);
        let eased = 1.0 - (1.0 - t).powi(3);
        adj.set_value(start + (target - start) * eased);
        if t >= 1.0 {
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}
