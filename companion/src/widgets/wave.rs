use std::cell::{Cell, RefCell};
use std::f64::consts::TAU;
use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::services::media::Media;

const WAVELENGTH: f64 = 22.0;
const AMPLITUDE: f64 = 3.0;
const GAP: f64 = 5.0;

/// The timeline: the played part is a little worm that waves while the track
/// plays and lies flat when paused. Frames are only drawn while it moves.
pub struct Wave {
    pub area: gtk::DrawingArea,
    media: Rc<Media>,
    amplitude: Cell<f64>,
    phase: Cell<f64>,
    drag: Cell<Option<f64>>,
    tick: RefCell<Option<gtk::TickCallbackId>>,
}

impl Wave {
    pub fn new(media: &Rc<Media>) -> Rc<Self> {
        let area = gtk::DrawingArea::new();
        area.add_css_class("wave");
        area.set_content_height(18);
        area.set_hexpand(true);

        let wave = Rc::new(Self {
            area,
            media: media.clone(),
            amplitude: Cell::new(0.0),
            phase: Cell::new(0.0),
            drag: Cell::new(None),
            tick: RefCell::new(None),
        });

        let weak = Rc::downgrade(&wave);
        wave.area.set_draw_func(move |area, cr, w, h| {
            if let Some(wave) = weak.upgrade() {
                wave.draw(area, cr, w as f64, h as f64);
            }
        });

        let drag = gtk::GestureDrag::new();
        {
            let weak = Rc::downgrade(&wave);
            drag.connect_drag_begin(move |_, x, _| {
                if let Some(wave) = weak.upgrade() {
                    wave.scrub(x);
                }
            });
            let weak = Rc::downgrade(&wave);
            drag.connect_drag_update(move |gesture, dx, _| {
                if let (Some(wave), Some((x, _))) = (weak.upgrade(), gesture.start_point()) {
                    wave.scrub(x + dx);
                }
            });
            let weak = Rc::downgrade(&wave);
            drag.connect_drag_end(move |_, _, _| {
                if let Some(wave) = weak.upgrade() {
                    if let Some(fraction) = wave.drag.take() {
                        let length = wave
                            .media
                            .track
                            .get()
                            .as_ref()
                            .map(|t| t.length)
                            .unwrap_or(0.0);
                        wave.media.seek(fraction * length);
                    }
                    wave.area.queue_draw();
                }
            });
        }
        wave.area.add_controller(drag);

        let weak = Rc::downgrade(&wave);
        wave.area.connect_map(move |_| {
            if let Some(wave) = weak.upgrade() {
                wave.run();
            }
        });
        let weak = Rc::downgrade(&wave);
        wave.area.connect_unmap(move |_| {
            if let Some(wave) = weak.upgrade() {
                wave.halt();
            }
        });
        let weak = Rc::downgrade(&wave);
        media.track.subscribe(move |_| {
            if let Some(wave) = weak.upgrade() {
                wave.run();
            }
        });
        let weak = Rc::downgrade(&wave);
        media.position.subscribe(move |_| {
            if let Some(wave) = weak.upgrade() {
                wave.area.queue_draw();
            }
        });
        wave
    }

    fn scrub(&self, x: f64) {
        let width = self.area.width().max(1) as f64;
        self.drag.set(Some((x / width).clamp(0.0, 1.0)));
        self.area.queue_draw();
    }

    fn playing(&self) -> bool {
        self.media.track.get().as_ref().is_some_and(|t| t.playing)
    }

    fn halt(&self) {
        if let Some(id) = self.tick.borrow_mut().take() {
            id.remove();
        }
    }

    /// Ticks while mapped and either playing or still settling flat.
    fn run(self: &Rc<Self>) {
        if self.tick.borrow().is_some() || !self.area.is_mapped() {
            return;
        }
        if !self.playing() && self.amplitude.get() <= 0.0 {
            self.area.queue_draw();
            return;
        }
        let weak = Rc::downgrade(self);
        let last = Cell::new(None::<i64>);
        let id = self.area.add_tick_callback(move |area, clock| {
            let Some(wave) = weak.upgrade() else {
                return glib::ControlFlow::Break;
            };
            let now = clock.frame_time();
            let dt = last
                .replace(Some(now))
                .map_or(0.0, |prev| (now - prev) as f64 / 1_000_000.0);
            let target = if wave.playing() { 1.0 } else { 0.0 };
            let amp = wave.amplitude.get();
            let next = amp + (target - amp) * (1.0 - (-dt * 8.0).exp());
            wave.amplitude.set(if (next - target).abs() < 0.01 {
                target
            } else {
                next
            });
            wave.phase.set((wave.phase.get() + dt * TAU * 0.8) % TAU);
            area.queue_draw();
            if target == 0.0 && wave.amplitude.get() == 0.0 {
                wave.tick.borrow_mut().take();
                return glib::ControlFlow::Break;
            }
            glib::ControlFlow::Continue
        });
        *self.tick.borrow_mut() = Some(id);
    }

    fn draw(&self, area: &gtk::DrawingArea, cr: &gtk::cairo::Context, w: f64, h: f64) {
        let length = self
            .media
            .track
            .get()
            .as_ref()
            .map(|t| t.length)
            .unwrap_or(0.0);
        let fraction = self
            .drag
            .get()
            .unwrap_or_else(|| {
                if length > 0.0 {
                    self.media.now() / length
                } else {
                    0.0
                }
            })
            .clamp(0.0, 1.0);
        let color = area.color();
        let (r, g, b) = (
            color.red() as f64,
            color.green() as f64,
            color.blue() as f64,
        );
        let mid = h / 2.0;
        let edge = 2.0;
        let at = edge + fraction * (w - edge * 2.0);

        cr.set_line_cap(gtk::cairo::LineCap::Round);
        cr.set_line_width(3.0);

        if at + GAP < w - edge {
            cr.set_source_rgba(r, g, b, 0.25);
            cr.move_to(at + GAP, mid);
            cr.line_to(w - edge, mid);
            let _ = cr.stroke();
        }

        let end = at - GAP;
        if end > edge {
            let amp = AMPLITUDE * self.amplitude.get();
            let phase = self.phase.get();
            cr.set_source_rgba(r, g, b, 1.0);
            let mut x = edge;
            cr.move_to(x, mid);
            while x < end {
                x = (x + 1.5).min(end);
                let taper = ((x - edge) / 10.0).min((end - x) / 10.0).clamp(0.0, 1.0);
                cr.line_to(x, mid + amp * taper * (x * TAU / WAVELENGTH - phase).sin());
            }
            let _ = cr.stroke();
        }

        cr.set_source_rgba(r, g, b, 1.0);
        cr.set_line_width(4.0);
        cr.move_to(at, mid - 6.0);
        cr.line_to(at, mid + 6.0);
        let _ = cr.stroke();
    }
}
