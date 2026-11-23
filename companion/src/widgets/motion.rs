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
            let (w, h) = (widget.width() as f32, widget.height() as f32);
            let rest = 1.0 - p;
            snapshot.save();
            snapshot.push_opacity(p.min(1.0) as f64);
            let (dx, dy, ox, oy) = match self.from.get() {
                Origin::Left => (-24.0 * rest, 0.0, 0.0, h / 2.0),
                Origin::LeftBottom => (-24.0 * rest, 0.0, 0.0, h),
                Origin::Right => (24.0 * rest, 0.0, w, 0.0),
                Origin::Bottom => (0.0, 24.0 * rest, w / 2.0, h),
                Origin::BottomEdge => (0.0, h * rest, w / 2.0, h),
            };
            snapshot.translate(&graphene::Point::new(dx, dy));
            if !matches!(self.from.get(), Origin::BottomEdge) {
                let scale = 0.92 + 0.08 * p;
                snapshot.translate(&graphene::Point::new(ox, oy));
                snapshot.scale(scale, scale);
                snapshot.translate(&graphene::Point::new(-ox, -oy));
            }
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

    pub fn set_progress(&self, progress: f64) {
        self.imp().progress.set(progress);
        self.queue_draw();
    }
}
