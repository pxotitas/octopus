use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, graphene};

use crate::widgets::anim::{self, Tween};

/// M3 expressive shapes, drawn as a polar wave: `lobes` bumps of `depth`
/// around a circle. Changing shape morphs through the circle.
#[derive(Clone, Copy, PartialEq, Default)]
pub enum Kind {
    #[default]
    Circle,
    Clover4,
    Clover8,
    Cookie4,
    Cookie6,
    Cookie7,
    Cookie9,
    Cookie12,
    Pentagon,
    Sunny,
}

/// The shapes that read as shapes (the circle is the morph's midpoint).
pub const VARIETY: [Kind; 9] = [
    Kind::Clover4,
    Kind::Clover8,
    Kind::Cookie4,
    Kind::Cookie6,
    Kind::Cookie7,
    Kind::Cookie9,
    Kind::Cookie12,
    Kind::Pentagon,
    Kind::Sunny,
];

/// A random shape other than `not`.
pub fn random_kind(not: &[Kind]) -> Kind {
    let choices: Vec<Kind> = VARIETY.into_iter().filter(|k| !not.contains(k)).collect();
    if choices.is_empty() {
        return Kind::Cookie9;
    }
    choices[glib::random_int_range(0, choices.len() as i32) as usize]
}

impl Kind {
    fn params(self) -> (f64, f64) {
        match self {
            Kind::Circle => (0.0, 0.0),
            Kind::Clover4 => (4.0, 0.16),
            Kind::Clover8 => (8.0, 0.11),
            Kind::Cookie4 => (4.0, 0.07),
            Kind::Cookie6 => (6.0, 0.085),
            Kind::Cookie7 => (7.0, 0.07),
            Kind::Cookie9 => (9.0, 0.055),
            Kind::Cookie12 => (12.0, 0.035),
            Kind::Pentagon => (5.0, 0.09),
            Kind::Sunny => (8.0, 0.04),
        }
    }
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Shape {
        pub kind: Cell<Kind>,
        /// 0 draws a circle, 1 the full shape.
        pub depth: Cell<f64>,
        pub angle: Cell<f64>,
        pub fill: Cell<bool>,
        pub spin: RefCell<Option<gtk::TickCallbackId>>,
        pub tween: RefCell<Option<Rc<Tween>>>,
        pub turn: RefCell<Option<Rc<Tween>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Shape {
        const NAME: &'static str = "OctopusShape";
        type Type = super::Shape;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
            klass.set_css_name("shape");
        }
    }

    impl ObjectImpl for Shape {
        fn dispose(&self) {
            if let Some(id) = self.spin.borrow_mut().take() {
                id.remove();
            }
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Shape {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let (w, h) = (widget.width() as f32, widget.height() as f32);
            if w <= 0.0 || h <= 0.0 {
                return;
            }
            let (lobes, depth) = self.kind.get().params();
            let path = outline(w, h, lobes, depth * self.depth.get(), self.angle.get());
            if self.fill.get() {
                snapshot.append_fill(&path, gsk::FillRule::Winding, &widget.color());
            }
            if let Some(child) = widget.first_child() {
                snapshot.push_fill(&path, gsk::FillRule::Winding);
                widget.snapshot_child(&child, snapshot);
                snapshot.pop();
            }
        }
    }

    fn outline(w: f32, h: f32, lobes: f64, depth: f64, angle: f64) -> gsk::Path {
        let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
        let radius = cx.min(cy);
        let builder = gsk::PathBuilder::new();
        if depth <= 0.0005 {
            builder.add_circle(&graphene::Point::new(cx as f32, cy as f32), radius as f32);
            return builder.to_path();
        }
        const STEPS: usize = 144;
        for i in 0..STEPS {
            let t = i as f64 / STEPS as f64 * std::f64::consts::TAU;
            let r = radius * (1.0 + depth * (lobes * (t + angle)).cos()) / (1.0 + depth);
            let (x, y) = ((cx + r * t.cos()) as f32, (cy + r * t.sin()) as f32);
            if i == 0 {
                builder.move_to(x, y);
            } else {
                builder.line_to(x, y);
            }
        }
        builder.close();
        builder.to_path()
    }
}

glib::wrapper! {
    pub struct Shape(ObjectSubclass<imp::Shape>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Shape {
    /// `fill` paints the shape in the widget's CSS `color`; the child is
    /// always clipped to it.
    pub fn new(kind: Kind, size: i32, fill: bool, child: Option<&impl IsA<gtk::Widget>>) -> Self {
        let shape: Self = glib::Object::new();
        let imp = shape.imp();
        imp.kind.set(kind);
        imp.depth.set(1.0);
        imp.fill.set(fill);
        shape.set_size_request(size, size);
        shape.set_halign(gtk::Align::Center);
        shape.set_valign(gtk::Align::Center);
        if let Some(child) = child {
            child.set_parent(&shape);
        }
        shape
    }

    /// Morphs through the circle into `kind`.
    pub fn morph(&self, kind: Kind) {
        let imp = self.imp();
        if imp.kind.get() == kind {
            return;
        }
        let tween = self.tween();
        let weak = self.downgrade();
        let from = imp.kind.get();
        tween.set(0.0);
        tween.animate(
            self,
            1.0,
            320,
            anim::STANDARD,
            move |t| {
                let Some(s) = weak.upgrade() else { return };
                let imp = s.imp();
                let (current, depth) = if from == Kind::Circle || t >= 0.5 {
                    (
                        kind,
                        if from == Kind::Circle {
                            t
                        } else {
                            (t - 0.5) * 2.0
                        },
                    )
                } else {
                    (from, 1.0 - t * 2.0)
                };
                imp.kind.set(current);
                imp.depth.set(depth);
                s.queue_draw();
            },
            || {},
        );
    }

    pub fn kind(&self) -> Kind {
        self.imp().kind.get()
    }

    /// Mahoraga's wheel: clicks round by a random number of notches, either
    /// way, and becomes another shape.
    pub fn adapt(&self, avoid: &[Kind]) -> Kind {
        let mut not = avoid.to_vec();
        not.push(self.kind());
        let kind = random_kind(&not);
        self.morph(kind);
        let notches = glib::random_int_range(1, 4) as f64;
        let direction = if glib::random_int_range(0, 2) == 0 {
            -1.0
        } else {
            1.0
        };
        let ms = glib::random_int_range(420, 700) as u32;
        self.turn(direction * notches * std::f64::consts::TAU / 16.0, ms);
        kind
    }

    /// Turns by `radians` and settles past the mark, like a ratchet.
    pub fn turn(&self, radians: f64, ms: u32) {
        let imp = self.imp();
        let tween = imp
            .turn
            .borrow_mut()
            .get_or_insert_with(|| Tween::new(0.0))
            .clone();
        tween.set(imp.angle.get());
        let weak = self.downgrade();
        tween.animate(
            self,
            imp.angle.get() + radians,
            ms,
            anim::OVERSHOOT,
            move |a| {
                if let Some(s) = weak.upgrade() {
                    s.imp().angle.set(a);
                    s.queue_draw();
                }
            },
            || {},
        );
    }

    fn tween(&self) -> Rc<Tween> {
        self.imp()
            .tween
            .borrow_mut()
            .get_or_insert_with(|| Tween::new(0.0))
            .clone()
    }

    /// A slow turn while something plays; stops as soon as it is off screen.
    pub fn set_spinning(&self, on: bool) {
        let imp = self.imp();
        if !on {
            if let Some(id) = imp.spin.borrow_mut().take() {
                id.remove();
            }
            return;
        }
        if imp.spin.borrow().is_some() {
            return;
        }
        let last = Cell::new(None::<i64>);
        let id = self.add_tick_callback(move |shape, clock| {
            let now = clock.frame_time();
            if let Some(prev) = last.replace(Some(now)) {
                let dt = (now - prev) as f64 / 1_000_000.0;
                let imp = shape.imp();
                imp.angle
                    .set((imp.angle.get() + dt * 0.35) % std::f64::consts::TAU);
                shape.queue_draw();
            }
            glib::ControlFlow::Continue
        });
        *imp.spin.borrow_mut() = Some(id);
    }
}
