use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::services::notifications::{Notification, Notifications};
use crate::widgets::anim::{self, Tween};
use crate::widgets::motion::{Motion, Origin};
use crate::widgets::notification;

struct Toast {
    id: u32,
    motion: Motion,
    tween: Rc<Tween>,
}

/// Popups for new notifications, stacked in the top-right corner.
pub struct Toasts {
    window: gtk::Window,
    column: gtk::Box,
    items: RefCell<Vec<Toast>>,
    service: Rc<Notifications>,
    timeout_ms: u32,
    max: usize,
}

impl Toasts {
    pub fn new(
        app: &gtk::Application,
        service: &Rc<Notifications>,
        timeout_ms: u32,
        max: usize,
        position: &str,
    ) -> Rc<Self> {
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .resizable(false)
            .build();
        window.add_css_class("octopus-popup");
        window.init_layer_shell();
        window.set_namespace(Some("octopus-notifications"));
        window.set_layer(Layer::Top);
        window.set_keyboard_mode(KeyboardMode::None);
        let vertical = if position.starts_with("bottom") { Edge::Bottom } else { Edge::Top };
        let side = if position.ends_with("left") { Edge::Left } else { Edge::Right };
        for edge in [vertical, side] {
            window.set_anchor(edge, true);
            window.set_margin(edge, 8);
        }

        let column = gtk::Box::new(gtk::Orientation::Vertical, 0);
        column.add_css_class("toasts");
        window.set_child(Some(&column));

        let toasts = Rc::new(Self {
            window,
            column,
            items: RefCell::new(Vec::new()),
            service: service.clone(),
            timeout_ms,
            max: max.max(1),
        });

        let weak = Rc::downgrade(&toasts);
        service.on_arrival(move |n| {
            if let Some(t) = weak.upgrade() {
                t.show(n);
            }
        });
        let weak = Rc::downgrade(&toasts);
        service.on_removal(move |id| {
            if let Some(t) = weak.upgrade() {
                t.remove(id);
            }
        });
        toasts
    }

    fn show(self: &Rc<Self>, n: &Rc<Notification>) {
        self.remove_now(n.id);
        while self.items.borrow().len() >= self.max {
            let oldest = self.items.borrow().first().map(|t| t.id);
            match oldest {
                Some(id) => self.remove_now(id),
                None => break,
            }
        }

        let weak = Rc::downgrade(self);
        let id = n.id;
        let card = notification::card(n, &self.service, move || {
            if let Some(t) = weak.upgrade() {
                t.remove(id);
            }
        });
        card.add_css_class("toast");
        let motion = Motion::new(&card, Origin::Right);
        self.column.append(&motion);
        let tween = Tween::new(0.0);
        self.window.set_visible(true);
        {
            let motion = motion.clone();
            tween.animate(
                &card,
                1.0,
                380,
                anim::EMPHASIZED_DECELERATE,
                move |p| motion.set_progress(p),
                || {},
            );
        }
        self.items.borrow_mut().push(Toast { id, motion, tween });

        let timeout = match n.timeout {
            t if n.critical && t <= 0 => None,
            t if t > 0 => Some(t as u64),
            _ => Some(self.timeout_ms as u64),
        };
        if let Some(ms) = timeout {
            let weak = Rc::downgrade(self);
            glib::timeout_add_local_once(Duration::from_millis(ms), move || {
                if let Some(t) = weak.upgrade() {
                    t.remove(id);
                }
            });
        }
    }

    /// Slides out; the history in the control center keeps it.
    fn remove(self: &Rc<Self>, id: u32) {
        let Some((motion, tween)) = self
            .items
            .borrow()
            .iter()
            .find(|t| t.id == id)
            .map(|t| (t.motion.clone(), t.tween.clone()))
        else {
            return;
        };
        let weak = Rc::downgrade(self);
        let target = motion.clone();
        tween.animate(
            &motion,
            0.0,
            200,
            anim::EMPHASIZED_ACCELERATE,
            move |p| target.set_progress(p),
            move || {
                if let Some(t) = weak.upgrade() {
                    t.remove_now(id);
                }
            },
        );
    }

    fn remove_now(&self, id: u32) {
        let mut items = self.items.borrow_mut();
        if let Some(pos) = items.iter().position(|t| t.id == id) {
            let toast = items.remove(pos);
            toast.tween.stop();
            self.column.remove(&toast.motion);
        }
        if items.is_empty() {
            self.window.set_visible(false);
        }
    }
}
