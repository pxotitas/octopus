use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use gtk::prelude::*;

/// A panel built the first time it opens and dropped a while after it
/// closes, so the panels used now and then cost nothing in between.
pub struct Lazy<T> {
    pub holder: gtk::Box,
    value: RefCell<Option<Rc<T>>>,
    build: Box<dyn Fn() -> Rc<T>>,
    root: fn(&T) -> gtk::Widget,
    timer: Cell<Option<glib::SourceId>>,
}

const KEEP: Duration = Duration::from_secs(60);

impl<T: 'static> Lazy<T> {
    pub fn new(build: impl Fn() -> Rc<T> + 'static, root: fn(&T) -> gtk::Widget) -> Rc<Self> {
        Rc::new(Self {
            holder: gtk::Box::new(gtk::Orientation::Vertical, 0),
            value: RefCell::new(None),
            build: Box::new(build),
            root,
            timer: Cell::new(None),
        })
    }

    pub fn get(&self) -> Rc<T> {
        if let Some(id) = self.timer.take() {
            id.remove();
        }
        if let Some(value) = self.value.borrow().as_ref() {
            return value.clone();
        }
        let value = (self.build)();
        self.holder.append(&(self.root)(&value));
        *self.value.borrow_mut() = Some(value.clone());
        value
    }

    pub fn release_later(self: &Rc<Self>) {
        if let Some(id) = self.timer.take() {
            id.remove();
        }
        let weak = Rc::downgrade(self);
        self.timer
            .set(Some(glib::timeout_add_local_once(KEEP, move || {
                let Some(lazy) = weak.upgrade() else { return };
                lazy.timer.set(None);
                if lazy.value.borrow_mut().take().is_some() {
                    while let Some(child) = lazy.holder.first_child() {
                        lazy.holder.remove(&child);
                    }
                    crate::memory::release_soon();
                }
            })));
    }
}
