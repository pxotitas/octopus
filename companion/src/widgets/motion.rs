use std::cell::Cell;

use gtk::glib;
use gtk::graphene;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

/// Where a panel comes from: it fades in while sliding from this side and
/// growing out of that corner. Drawing only, so the window never resizes
/// while it animates.
#[derive(Clone, Copy, Default)]
pub enum Origin {
    #[default]
    Left,
    LeftBottom,
    Bottom,
    Right,
    /// Slides fully out past the bottom edge, for surfaces docked to it.
    BottomEdge,
    /// Fades with a short rise that stays inside the window's margin, so
    /// nothing is cut at the window's edge.
    Fade,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Motion {
        pub progress: Cell<f64>,
        pub from: Cell<Origin>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Motion {
        const NAME: &'static str = "OctopusMotion";
        type Type = super::Motion;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_layout_manager_type::<gtk::BinLayout>();
        }
    }

    impl ObjectImpl for Motion {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Motion {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let Some(child) = widget.first_child() else {
                return;
            };
            let p = self.progress.get() as f32;
            if p <= 0.001 {
                return;
            }
            if p >= 0.999 {
                widget.snapshot_child(&child, snapshot);
                return;
            }
            let h = widget.height() as f32;
            let rest = 1.0 - p;
            snapshot.save();
            snapshot.push_opacity(p.min(1.0) as f64);
            // Translation only, in whole pixels: scaling text forces GSK to
            // rasterize new glyph sizes every frame.
            let (dx, dy) = match self.from.get() {
                Origin::Left | Origin::LeftBottom => (-28.0 * rest, 0.0),
                Origin::Right => (28.0 * rest, 0.0),
                Origin::Bottom => (0.0, 28.0 * rest),
                Origin::BottomEdge => (0.0, h * rest),
                Origin::Fade => (0.0, 10.0 * rest),
            };
            snapshot.translate(&graphene::Point::new(dx.round(), dy.round()));
            widget.snapshot_child(&child, snapshot);
            snapshot.pop();
            snapshot.restore();
        }
    }
}

glib::wrapper! {
    pub struct Motion(ObjectSubclass<imp::Motion>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Motion {
    pub fn new(child: &impl IsA<gtk::Widget>, from: Origin) -> Self {
        let motion: Self = glib::Object::new();
        motion.imp().from.set(from);
        child.set_parent(&motion);
        motion
    }

    pub fn set_origin(&self, from: Origin) {
        self.imp().from.set(from);
    }

    pub fn set_progress(&self, progress: f64) {
        self.imp().progress.set(progress);
        self.queue_draw();
    }
}
