//! Small drawings for the weather panel: a day's low-to-high bar, the sun's
//! arc between sunrise and sunset, and the moon's phase. All in CSS `color`.

use std::cell::Cell;
use std::f32::consts::PI;

use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};

fn faded(color: &gdk::RGBA, alpha: f32) -> gdk::RGBA {
    gdk::RGBA::new(color.red(), color.green(), color.blue(), color.alpha() * alpha)
}

fn stroke(width: f32) -> gsk::Stroke {
    let stroke = gsk::Stroke::new(width);
    stroke.set_line_cap(gsk::LineCap::Round);
    stroke
}

/// What a drawing shows: fractions of its width or of a turn.
#[derive(Default, Clone, Copy, PartialEq)]
pub enum Figure {
    #[default]
    None,
    /// A track with a bar from `low` to `high` (0..1) and an optional dot.
    Range { low: f32, high: f32, now: Option<f32> },
    /// The sun `progress` (0..1, or outside when it's down) along its arc.
    Sun { progress: f32 },
    /// The moon `age`: 0 new, 0.5 full.
    Moon { age: f32 },
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Sky {
        pub figure: Cell<Figure>,
        pub width: Cell<i32>,
        pub height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Sky {
        const NAME: &'static str = "OctopusSky";
        type Type = super::Sky;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("sky");
        }
    }

    impl ObjectImpl for Sky {}

    impl WidgetImpl for Sky {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let size = if orientation == gtk::Orientation::Horizontal {
                self.width.get()
            } else {
                self.height.get()
            };
            (size, size, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let (w, h) = (widget.width() as f32, widget.height() as f32);
            if w <= 0.0 || h <= 0.0 {
                return;
            }
            let ink = widget.color();
            match self.figure.get() {
                Figure::None => {}
                Figure::Range { low, high, now } => range(snapshot, w, h, low, high, now, &ink),
                Figure::Sun { progress } => sun(snapshot, w, h, progress, &ink),
                Figure::Moon { age } => moon(snapshot, w.min(h), age, &ink),
            }
        }
    }

    fn range(snapshot: &gtk::Snapshot, w: f32, h: f32, low: f32, high: f32, now: Option<f32>, ink: &gdk::RGBA) {
        let thick = h.min(6.0);
        let y = h / 2.0;
        let pad = thick / 2.0;
        let x = |f: f32| pad + f.clamp(0.0, 1.0) * (w - thick);
        let track = gsk::PathBuilder::new();
        track.move_to(pad, y);
        track.line_to(w - pad, y);
        snapshot.append_stroke(&track.to_path(), &stroke(thick), &faded(ink, 0.18));
        let bar = gsk::PathBuilder::new();
        bar.move_to(x(low), y);
        bar.line_to(x(high).max(x(low) + 0.1), y);
        snapshot.append_stroke(&bar.to_path(), &stroke(thick), ink);
        if let Some(now) = now {
            let dot = gsk::PathBuilder::new();
            dot.add_circle(&graphene::Point::new(x(now), y), thick * 0.9);
            snapshot.append_fill(&dot.to_path(), gsk::FillRule::Winding, &faded(ink, 0.45));
        }
    }

    /// A half ellipse from sunrise (left) to sunset (right), the walked part
    /// solid, with the sun on it.
    fn sun(snapshot: &gtk::Snapshot, w: f32, h: f32, progress: f32, ink: &gdk::RGBA) {
        let thick = 4.0;
        let (cx, base) = (w / 2.0, h - 6.0);
        let (rx, ry) = (w / 2.0 - 10.0, h - 18.0);
        let at = |t: f32| {
            let a = PI * (1.0 - t);
            graphene::Point::new(cx + rx * a.cos(), base - ry * a.sin())
        };
        let arc = |from: f32, to: f32| {
            let path = gsk::PathBuilder::new();
            let steps = 64;
            for i in 0..=steps {
                let p = at(from + (to - from) * i as f32 / steps as f32);
                if i == 0 {
                    path.move_to(p.x(), p.y());
                } else {
                    path.line_to(p.x(), p.y());
                }
            }
            path.to_path()
        };
        let horizon = gsk::PathBuilder::new();
        horizon.move_to(2.0, base);
        horizon.line_to(w - 2.0, base);
        snapshot.append_stroke(&horizon.to_path(), &stroke(2.0), &faded(ink, 0.25));
        snapshot.append_stroke(&arc(0.0, 1.0), &stroke(thick), &faded(ink, 0.2));
        if (0.0..=1.0).contains(&progress) {
            if progress > 0.0 {
                snapshot.append_stroke(&arc(0.0, progress), &stroke(thick), ink);
            }
            let dot = gsk::PathBuilder::new();
            dot.add_circle(&at(progress), 8.0);
            snapshot.append_fill(&dot.to_path(), gsk::FillRule::Winding, ink);
        }
    }

    /// The moon as it looks, whatever the theme: a dark disc, its lit part
    /// pale, outlined in `ink`.
    fn moon(snapshot: &gtk::Snapshot, side: f32, age: f32, ink: &gdk::RGBA) {
        let (c, r) = (side / 2.0, side / 2.0 - 1.5);
        let disc = gsk::PathBuilder::new();
        disc.add_circle(&graphene::Point::new(c, c), r);
        let disc = disc.to_path();
        snapshot.append_fill(&disc, gsk::FillRule::Winding, &gdk::RGBA::new(0.16, 0.16, 0.13, 1.0));
        snapshot.append_stroke(&disc, &stroke(1.5), &faded(ink, 0.6));
        let pale = gdk::RGBA::new(0.97, 0.95, 0.86, 1.0);

        // Illuminated fraction and which side is lit.
        let phase = (age.rem_euclid(1.0)) * 2.0 * PI;
        let lit = (1.0 - phase.cos()) / 2.0;
        if lit < 0.02 {
            return;
        }
        let waxing = age.rem_euclid(1.0) < 0.5;
        let side_sign = if waxing { 1.0 } else { -1.0 };
        let term = r * (1.0 - 2.0 * lit);
        let path = gsk::PathBuilder::new();
        let steps = 48;
        for i in 0..=steps {
            let t = -PI / 2.0 + PI * i as f32 / steps as f32;
            let (x, y) = (c + side_sign * r * t.cos(), c + r * t.sin());
            if i == 0 {
                path.move_to(x, y);
            } else {
                path.line_to(x, y);
            }
        }
        for i in 0..=steps {
            let t = PI / 2.0 - PI * i as f32 / steps as f32;
            path.line_to(c + side_sign * term * t.cos(), c + r * t.sin());
        }
        path.close();
        snapshot.append_fill(&path.to_path(), gsk::FillRule::Winding, &pale);
    }
}

glib::wrapper! {
    pub struct Sky(ObjectSubclass<imp::Sky>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Sky {
    pub fn new(figure: Figure, width: i32, height: i32) -> Self {
        let sky: Self = glib::Object::new();
        let imp = sky.imp();
        imp.figure.set(figure);
        imp.width.set(width);
        imp.height.set(height);
        sky.set_valign(gtk::Align::Center);
        sky
    }
}

/// How much of the moon is lit, 0 to 1, for an age from 0 (new) to 1.
pub fn lit(age: f64) -> f64 {
    (1.0 - (age.rem_euclid(1.0) * std::f64::consts::TAU).cos()) / 2.0
}

/// Minutes since midnight for "hh:mm".
pub fn minutes(clock: &str) -> Option<i32> {
    let (h, m) = clock.split_once(':')?;
    Some(h.parse::<i32>().ok()? * 60 + m.parse::<i32>().ok()?)
}
