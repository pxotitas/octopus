mod blur;
mod pam;
mod surface;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;
use gtk4_session_lock::Instance;

use crate::exec;
use crate::services::Services;
use surface::Surface;

/// The session lock (ext-session-lock): if the daemon dies while locked,
/// the compositor keeps the screen locked.
pub struct Lock {
    app: gtk::Application,
    services: Rc<Services>,
    lyrics: bool,
    instance: RefCell<Option<Instance>>,
    surfaces: RefCell<Vec<Rc<Surface>>>,
    background: RefCell<Option<gtk::gdk::Texture>>,
    checking: Cell<bool>,
}

impl Lock {
    pub fn new(app: &gtk::Application, services: &Rc<Services>, lyrics: bool) -> Rc<Self> {
        Rc::new(Self {
            app: app.clone(),
            services: services.clone(),
            lyrics,
            instance: RefCell::new(None),
            surfaces: RefCell::new(Vec::new()),
            background: RefCell::new(None),
            checking: Cell::new(false),
        })
    }

    pub fn start(self: &Rc<Self>) {
        if self.instance.borrow().is_some() || !gtk4_session_lock::is_supported() {
            return;
        }
        let instance = Instance::new();

        let weak = Rc::downgrade(self);
        instance.connect_monitor(move |instance, monitor| {
            let Some(lock) = weak.upgrade() else { return };
            let submit: Rc<dyn Fn(String)> = {
                let weak = Rc::downgrade(&lock);
                Rc::new(move |password| {
                    if let Some(lock) = weak.upgrade() {
                        lock.check(password);
                    }
                })
            };
            let surface = Surface::new(&lock.app, &lock.services, lock.lyrics, submit);
            if let Some(texture) = lock.background.borrow().as_ref() {
                surface.set_background(texture);
            }
            instance.assign_window_to_monitor(&surface.window, monitor);
            surface.present();
            lock.surfaces.borrow_mut().push(surface);
        });

        let weak = Rc::downgrade(self);
        instance.connect_failed(move |_| {
            eprintln!("octopus: the compositor refused the lock");
            if let Some(lock) = weak.upgrade() {
                lock.finish();
            }
        });
        let weak = Rc::downgrade(self);
        instance.connect_unlocked(move |_| {
            if let Some(lock) = weak.upgrade() {
                lock.finish();
            }
        });

        *self.instance.borrow_mut() = Some(instance.clone());
        self.load_background();
        if !instance.lock() {
            self.finish();
        }
    }

    fn load_background(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let query = exec::output("awww", &["query"]).await.unwrap_or_default();
            let Some(path) = query
                .lines()
                .find_map(|l| l.split("image: ").nth(1))
                .map(|p| p.trim().to_owned())
            else {
                return;
            };
            let texture =
                gtk::gio::spawn_blocking(move || blur::blurred(&path, 320, 6, 0.62, 0.35))
                    .await
                    .ok()
                    .flatten();
            let (Some(lock), Some(texture)) = (weak.upgrade(), texture) else {
                return;
            };
            for surface in lock.surfaces.borrow().iter() {
                surface.set_background(&texture);
            }
            *lock.background.borrow_mut() = Some(texture);
        });
    }

    fn check(self: &Rc<Self>, password: String) {
        if self.checking.replace(true) {
            return;
        }
        for surface in self.surfaces.borrow().iter() {
            surface.busy(true);
        }
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let user = glib::user_name().to_string_lossy().into_owned();
            let result = gtk::gio::spawn_blocking(move || {
                pam::authenticate(pam::service(), &user, &password)
            })
            .await
            .unwrap_or_else(|_| Err("falha na autenticação".to_owned()));
            let Some(lock) = weak.upgrade() else { return };
            lock.checking.set(false);
            match result {
                Ok(()) => lock.release(),
                Err(_) => {
                    for surface in lock.surfaces.borrow().iter() {
                        surface.fail("senha incorreta");
                    }
                }
            }
        });
    }

    fn release(self: &Rc<Self>) {
        let surfaces = self.surfaces.borrow().clone();
        let remaining = Rc::new(Cell::new(surfaces.len()));
        for surface in surfaces {
            let weak = Rc::downgrade(self);
            let remaining = remaining.clone();
            surface.leave(move || {
                remaining.set(remaining.get().saturating_sub(1));
                if remaining.get() == 0 {
                    if let Some(lock) = weak.upgrade() {
                        if let Some(instance) = lock.instance.borrow().as_ref() {
                            instance.unlock();
                        }
                    }
                }
            });
        }
    }

    fn finish(&self) {
        self.instance.borrow_mut().take();
        for surface in self.surfaces.borrow_mut().drain(..) {
            surface.window.destroy();
        }
    }
}
