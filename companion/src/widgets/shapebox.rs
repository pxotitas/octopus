use std::cell::Cell;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, graphene};

use crate::widgets::shape::{self, Kind};

/// The outline a card is cut to, after the M3 expressive shapes.
#[derive(Clone, Copy, PartialEq)]
pub enum Outline {
    Rounded(f32),
    /// A fully rounded end on each side.
    Pill,
    /// A round top over a square-ish base.
    Arch,
    /// A round top over a wavy bottom edge.
    Ghost,
    /// A rounded rectangle whose whole edge waves, like a cookie.
    Scallop,
    /// A polar shape stretched to the card.
    Blob(Kind),
}

impl Default for Outline {
    fn default() -> Self {
        Outline::Rounded(20.0)
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct ShapeBox {
        pub outline: Cell<Outline>,
        pub inset: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ShapeBox {
        const NAME: &'static str = "OctopusShapeBox";
        type Type = super::ShapeBox;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("shapebox");
        }
    }

    impl ObjectImpl for ShapeBox {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for ShapeBox {
        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            let inset = self.inset.get() * 2;
            match self.obj().first_child() {
                Some(child) => {
                    let for_size = if for_size >= 0 { (for_size - inset).max(0) } else { -1 };
                    let (min, nat, _, _) = child.measure(orientation, for_size);
                    (min + inset, nat + inset, -1, -1)
                }
                None => (inset, inset, -1, -1),
            }
        }

        fn request_mode(&self) -> gtk::SizeRequestMode {
            self.obj()
                .first_child()
                .map(|c| c.request_mode())
                .unwrap_or(gtk::SizeRequestMode::ConstantSize)
        }

        fn size_allocate(&self, width: i32, height: i32, _baseline: i32) {
            let inset = self.inset.get();
            if let Some(child) = self.obj().first_child() {
                let transform = gsk::Transform::new()
                    .translate(&graphene::Point::new(inset as f32, inset as f32));
                child.allocate(
                    (width - inset * 2).max(0),
                    (height - inset * 2).max(0),
                    -1,
                    Some(transform),
                );
            }
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let (w, h) = (widget.width() as f32, widget.height() as f32);
            if w > 0.0 && h > 0.0 {
                let path = super::outline_path(self.outline.get(), w, h);
                snapshot.append_fill(&path, gsk::FillRule::Winding, &widget.color());
            }
            if let Some(child) = widget.first_child() {
                widget.snapshot_child(&child, snapshot);
            }
        }
    }
}

glib::wrapper! {
    /// A container painted in its CSS `color` and cut to an M3 outline;
    /// `inset` keeps the child clear of the edge.
    pub struct ShapeBox(ObjectSubclass<imp::ShapeBox>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl ShapeBox {
    pub fn new(outline: Outline, inset: i32, child: &impl IsA<gtk::Widget>) -> Self {
        let card: Self = glib::Object::new();
        card.imp().outline.set(outline);
        card.imp().inset.set(inset);
        child.set_parent(&card);
        card
    }

    pub fn set_outline(&self, outline: Outline) {
        if self.imp().outline.replace(outline) != outline {
            self.queue_draw();
        }
    }
}

fn outline_path(outline: Outline, w: f32, h: f32) -> gsk::Path {
    let rect = graphene::Rect::new(0.0, 0.0, w, h);
    let corners = |tl: f32, tr: f32, br: f32, bl: f32| {
        let size = |r: f32| graphene::Size::new(r, r);
        gsk::RoundedRect::new(rect, size(tl), size(tr), size(br), size(bl))
    };
    let builder = gsk::PathBuilder::new();
    match outline {
        Outline::Rounded(r) => builder.add_rounded_rect(&corners(r, r, r, r)),
        Outline::Pill => {
            let r = w.min(h) / 2.0;
            builder.add_rounded_rect(&corners(r, r, r, r));
        }
        Outline::Arch => {
            let top = (w / 2.0).min(h * 0.6);
            builder.add_rounded_rect(&corners(top, top, 18.0, 18.0));
        }
        Outline::Ghost => ghost(&builder, w, h),
        Outline::Scallop => scallop(&builder, w, h),
        Outline::Blob(kind) => return shape::path(kind, w, h, 0.0),
    }
    builder.to_path()
}

/// Round top corners, then a bottom edge of gentle waves.
fn ghost(builder: &gsk::PathBuilder, w: f32, h: f32) {
    let r = (w.min(h) * 0.35).min(40.0);
    let amp = 5.0_f32;
    let base = h - amp;
    let waves = (w / 70.0).round().max(2.0);
    builder.move_to(0.0, base);
    builder.line_to(0.0, r);
    builder.svg_arc_to(r, r, 0.0, false, true, r, 0.0);
    builder.line_to(w - r, 0.0);
    builder.svg_arc_to(r, r, 0.0, false, true, w, r);
    builder.line_to(w, base);
    const STEPS: usize = 96;
    for i in 1..=STEPS {
        let t = i as f32 / STEPS as f32;
        let x = w * (1.0 - t);
        let y = base + amp * (t * waves * TAU).cos() * -1.0 + 0.0;
        builder.line_to(x, y.min(h));
    }
    builder.close();
}

/// A rounded rectangle walked edge by edge, pushed in and out along the way.
fn scallop(builder: &gsk::PathBuilder, w: f32, h: f32) {
    let amp = 3.5_f32;
    let r = (w.min(h) / 2.0 - amp).min(30.0).max(4.0);
    let (x0, y0, x1, y1) = (amp, amp, w - amp, h - amp);
    let (sw, sh) = (x1 - x0 - 2.0 * r, y1 - y0 - 2.0 * r);
    let arc = FRAC_PI_2 * r;
    let length = 2.0 * (sw + sh) + 4.0 * arc;
    let waves = (length / 42.0).round().max(8.0);

    // A point and its outward normal at distance `s` along the edge,
    // starting at the top edge's left end and going clockwise.
    let at = |mut s: f32| -> (f32, f32, f32, f32) {
        let corner = |cx: f32, cy: f32, start: f32, s: f32| {
            let a = start + s / r;
            (cx + r * a.cos(), cy + r * a.sin(), a.cos(), a.sin())
        };
        if s < sw {
            return (x0 + r + s, y0, 0.0, -1.0);
        }
        s -= sw;
        if s < arc {
            return corner(x1 - r, y0 + r, -FRAC_PI_2, s);
        }
        s -= arc;
        if s < sh {
            return (x1, y0 + r + s, 1.0, 0.0);
        }
        s -= sh;
        if s < arc {
            return corner(x1 - r, y1 - r, 0.0, s);
        }
        s -= arc;
        if s < sw {
            return (x1 - r - s, y1, 0.0, 1.0);
        }
        s -= sw;
        if s < arc {
            return corner(x0 + r, y1 - r, FRAC_PI_2, s);
        }
        s -= arc;
        if s < sh {
            return (x0, y1 - r - s, -1.0, 0.0);
        }
        s -= sh;
        corner(x0 + r, y0 + r, PI, s.min(arc))
    };

    let steps = (length / 2.0).ceil() as usize;
    for i in 0..steps {
        let s = length * i as f32 / steps as f32;
        let (x, y, nx, ny) = at(s);
        let push = amp * (s / length * waves * TAU).cos();
        let (px, py) = (x + nx * push, y + ny * push);
        if i == 0 {
            builder.move_to(px, py);
        } else {
            builder.line_to(px, py);
        }
    }
    builder.close();
}
