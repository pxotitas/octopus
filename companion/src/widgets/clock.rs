use std::cell::{Cell, RefCell};
use std::f32::consts::{FRAC_PI_2, TAU};

use chrono::Timelike;
use gtk::gsk;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib, graphene};

use crate::widgets::shape::{self, Kind};

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Clock {
        pub size: Cell<i32>,
        pub no_seconds: Cell<bool>,
        pub timer: RefCell<Option<glib::SourceId>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Clock {
        const NAME: &'static str = "OctopusClock";
        type Type = super::Clock;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.set_css_name("clock");
        }
    }

    impl ObjectImpl for Clock {
        fn constructed(&self) {
            self.parent_constructed();
            let widget = self.obj();
            widget.connect_map(|clock| clock.start());
            widget.connect_unmap(|clock| clock.stop());
        }

        fn dispose(&self) {
            self.obj().stop();
        }
    }

    impl WidgetImpl for Clock {
        fn measure(&self, _orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let size = self.size.get();
            (size, size, -1, -1)
        }

        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let widget = self.obj();
            let side = widget.width().min(widget.height()) as f32;
            if side <= 0.0 {
                return;
            }
            let ink = widget.color();
            let face = super::with_alpha(&ink, 0.16);
            let (c, r) = (side / 2.0, side / 2.0);
            let now = chrono::Local::now();

            snapshot.append_fill(&shape::path(Kind::Cookie12, side, side, 0.0), gsk::FillRule::Winding, &face);

            let dots = gsk::PathBuilder::new();
            for i in 0..12 {
                let a = i as f32 / 12.0 * TAU - FRAC_PI_2;
                let dot = if i % 3 == 0 { 2.6 } else { 1.6 };
                dots.add_circle(&point(c, a, r * 0.64), dot * side / 120.0);
            }
            snapshot.append_fill(&dots.to_path(), gsk::FillRule::Winding, &super::with_alpha(&ink, 0.55));

            let second = now.second() as f32 + now.nanosecond() as f32 / 1e9;
            if !self.no_seconds.get() {
                snake(snapshot, c, r * 0.8, second / 60.0, side, &super::with_alpha(&ink, 0.55));
            }

            let minute = now.minute() as f32 + second / 60.0;
            let hour = (now.hour() % 12) as f32 + minute / 60.0;
            hand(snapshot, c, hour / 12.0, r * 0.38, side * 0.085, &ink);
            hand(snapshot, c, minute / 60.0, r * 0.58, side * 0.05, &ink);

            let pin = gsk::PathBuilder::new();
            pin.add_circle(&graphene::Point::new(c, c), side * 0.035);
            snapshot.append_fill(&pin.to_path(), gsk::FillRule::Winding, &ink);
        }
    }

    fn point(c: f32, angle: f32, radius: f32) -> graphene::Point {
        graphene::Point::new(c + radius * angle.cos(), c + radius * angle.sin())
    }

    /// A rounded hand from the centre, `turn` of the way round.
    fn hand(snapshot: &gtk::Snapshot, c: f32, turn: f32, length: f32, width: f32, ink: &gdk::RGBA) {
        let a = turn * TAU - FRAC_PI_2;
        let path = gsk::PathBuilder::new();
        path.move_to(c, c);
        let tip = point(c, a, length);
        path.line_to(tip.x(), tip.y());
        let stroke = gsk::Stroke::new(width);
        stroke.set_line_cap(gsk::LineCap::Round);
        snapshot.append_stroke(&path.to_path(), &stroke, ink);
    }

    /// The seconds as a wavy line growing round the rim, like the players'
    /// timelines.
    fn snake(snapshot: &gtk::Snapshot, c: f32, radius: f32, progress: f32, side: f32, ink: &gdk::RGBA) {
        if progress <= 0.002 {
            return;
        }
        let path = gsk::PathBuilder::new();
        let waves = 24.0;
        let amp = side * 0.011;
        let steps = (progress * 360.0).ceil() as usize + 1;
        for i in 0..steps {
            let t = progress * i as f32 / (steps - 1) as f32;
            let a = t * TAU - FRAC_PI_2;
            let rr = radius + amp * (t * waves * TAU).sin();
            let p = point(c, a, rr);
            if i == 0 {
                path.move_to(p.x(), p.y());
            } else {
                path.line_to(p.x(), p.y());
            }
        }
        let stroke = gsk::Stroke::new(side * 0.022);
        stroke.set_line_cap(gsk::LineCap::Round);
        stroke.set_line_join(gsk::LineJoin::Round);
        snapshot.append_stroke(&path.to_path(), &stroke, ink);
    }
}

glib::wrapper! {
    /// An analog clock on an M3 cookie: dots for the hours, rounded hands,
    /// and the seconds as a wave round the rim. Drawn in its CSS `color`.
    pub struct Clock(ObjectSubclass<imp::Clock>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Clock {
    pub fn new(size: i32) -> Self {
        let clock: Self = glib::Object::new();
        clock.imp().size.set(size);
        clock.set_halign(gtk::Align::Center);
        clock.set_valign(gtk::Align::Center);
        clock
    }

    pub fn set_seconds(&self, on: bool) {
        self.imp().no_seconds.set(!on);
        self.queue_draw();
    }

    pub fn set_size(&self, size: i32) {
        self.imp().size.set(size);
        self.queue_resize();
    }

    /// Redraws about 4 times a second, only while on screen.
    fn start(&self) {
        let imp = self.imp();
        if imp.timer.borrow().is_some() {
            return;
        }
        let weak = self.downgrade();
        let id = glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
            match weak.upgrade() {
                Some(clock) => {
                    clock.queue_draw();
                    glib::ControlFlow::Continue
                }
                None => glib::ControlFlow::Break,
            }
        });
        *imp.timer.borrow_mut() = Some(id);
    }

    fn stop(&self) {
        if let Some(id) = self.imp().timer.borrow_mut().take() {
            id.remove();
        }
    }
}

fn with_alpha(color: &gdk::RGBA, alpha: f32) -> gdk::RGBA {
    gdk::RGBA::new(color.red(), color.green(), color.blue(), color.alpha() * alpha)
}
