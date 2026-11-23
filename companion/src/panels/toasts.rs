use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::services::notifications::{Notification, Notifications};
use crate::widgets::notification;

/// Popups for new notifications, stacked in the top-right corner.
pub struct Toasts {
    window: gtk::Window,
    column: gtk::Box,
    items: RefCell<Vec<(u32, gtk::Revealer)>>,
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
    ) -> Rc<Self> {
        let window = gtk::Window::builder()
            .application(app)
            .decorated(false)
            .build();
        window.add_css_class("octopus-popup");
        window.init_layer_shell();
        window.set_namespace(Some("octopus-notifications"));
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_margin(Edge::Top, 8);
        window.set_margin(Edge::Right, 8);

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
            let oldest = self.items.borrow().first().map(|(id, _)| *id);
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
        let revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::SlideDown)
            .transition_duration(220)
            .child(&card)
            .build();
        self.column.append(&revealer);
        self.items.borrow_mut().push((id, revealer.clone()));
        self.window.set_visible(true);
        glib::idle_add_local_once(move || revealer.set_reveal_child(true));

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
        let Some(revealer) = self
            .items
            .borrow()
            .iter()
            .find(|(i, _)| *i == id)
            .map(|(_, r)| r.clone())
        else {
            return;
        };
        let weak = Rc::downgrade(self);
        revealer.connect_child_revealed_notify(move |r| {
            if !r.is_child_revealed() {
                if let Some(t) = weak.upgrade() {
                    t.remove_now(id);
                }
            }
        });
        revealer.set_reveal_child(false);
    }

    fn remove_now(&self, id: u32) {
        let mut items = self.items.borrow_mut();
        if let Some(pos) = items.iter().position(|(i, _)| *i == id) {
            let (_, revealer) = items.remove(pos);
            self.column.remove(&revealer);
        }
        if items.is_empty() {
            self.window.set_visible(false);
        }
    }
}
