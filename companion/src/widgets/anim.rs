use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;

/// M3 motion curves as cubic béziers.
#[derive(Clone, Copy)]
pub struct Curve(f64, f64, f64, f64);

pub const EMPHASIZED_DECELERATE: Curve = Curve(0.05, 0.7, 0.1, 1.0);
pub const EMPHASIZED_ACCELERATE: Curve = Curve(0.3, 0.0, 0.8, 0.15);
pub const STANDARD: Curve = Curve(0.2, 0.0, 0.0, 1.0);

impl Curve {
    pub fn at(self, t: f64) -> f64 {
        let Curve(x1, y1, x2, y2) = self;
        let t = t.clamp(0.0, 1.0);
        let bez = |a: f64, b: f64, s: f64| {
            3.0 * a * s * (1.0 - s).powi(2) + 3.0 * b * s * s * (1.0 - s) + s.powi(3)
        };
        let mut s = t;
        for _ in 0..8 {
            let x = bez(x1, x2, s) - t;
            let dx = 3.0 * x1 * (1.0 - s).powi(2)
                + 6.0 * (x2 - x1) * s * (1.0 - s)
                + 3.0 * (1.0 - x2) * s * s;
            if dx.abs() < 1e-6 {
                break;
            }
            s = (s - x / dx).clamp(0.0, 1.0);
        }
        bez(y1, y2, s)
    }
}

/// A value animated on a widget's frame clock, so it runs at the display's
/// refresh rate and stops ticking when it arrives.
pub struct Tween {
    value: Cell<f64>,
    tick: RefCell<Option<gtk::TickCallbackId>>,
}

impl Tween {
    pub fn new(value: f64) -> Rc<Self> {
        Rc::new(Self {
            value: Cell::new(value),
            tick: RefCell::new(None),
        })
    }

    pub fn get(&self) -> f64 {
        self.value.get()
    }

    pub fn set(&self, value: f64) {
        self.stop();
        self.value.set(value);
    }

    pub fn stop(&self) {
        if let Some(id) = self.tick.borrow_mut().take() {
            id.remove();
        }
    }

    /// Calls `step` each frame with the new value and `done` on arrival.
    pub fn animate(
        self: &Rc<Self>,
        widget: &impl IsA<gtk::Widget>,
        to: f64,
        ms: u32,
        curve: Curve,
        step: impl Fn(f64) + 'static,
        done: impl FnOnce() + 'static,
    ) {
        self.stop();
        let from = self.value.get();
        if (from - to).abs() < 1e-4 || ms == 0 {
            self.value.set(to);
            step(to);
            done();
            return;
        }
        let start = Cell::new(None::<i64>);
        let duration = ms as f64 * 1000.0;
        let this = Rc::downgrade(self);
        let done = Cell::new(Some(done));
        let id = widget.add_tick_callback(move |_, clock| {
            let Some(this) = this.upgrade() else {
                return gtk::glib::ControlFlow::Break;
            };
            let now = clock.frame_time();
            let begin = match start.get() {
                Some(begin) => begin,
                None => {
                    start.set(Some(now));
                    now
                }
            };
            let t = ((now - begin) as f64 / duration).min(1.0);
            let v = from + (to - from) * curve.at(t);
            this.value.set(v);
            step(v);
            if t >= 1.0 {
                this.tick.borrow_mut().take();
                if let Some(done) = done.take() {
                    done();
                }
                return gtk::glib::ControlFlow::Break;
            }
            gtk::glib::ControlFlow::Continue
        });
        *self.tick.borrow_mut() = Some(id);
    }
}
