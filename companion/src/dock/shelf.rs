//! The dock's row of icons, with Velatura's magnification.

use std::cell::Cell;
use std::f64::consts::PI;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, graphene, gsk};

/// How many slots the swell reaches on each side of the pointer.
const REACH: f64 = 2.6;

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct Shelf {
        /// Pointer position along the row at rest.
        pub pointer: Cell<f64>,
        /// 0 at rest, 1 fully magnifying; eased by the dock.
        pub swell: Cell<f64>,
        /// Largest scale an icon reaches (1 turns magnification off).
        pub magnify: Cell<f64>,
        pub spacing: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Shelf {
        const NAME: &'static str = "OctopusShelf";
        type Type = super::Shelf;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for Shelf {
        fn dispose(&self) {
            while let Some(child) = self.obj().first_child() {
                child.unparent();
            }
        }
    }

    impl WidgetImpl for Shelf {
        fn measure(&self, orientation: gtk::Orientation, _for_size: i32) -> (i32, i32, i32, i32) {
            let widget = self.obj();
            let mut total = 0;
            let mut count = 0;
            let mut child = widget.first_child();
            while let Some(c) = child {
                if c.is_visible() {
                    let (_, nat, _, _) = c.measure(orientation, -1);
                    if orientation == gtk::Orientation::Horizontal {
                        total += nat;
                        count += 1;
                    } else {
                        total = total.max(nat);
                    }
                }
                child = c.next_sibling();
            }
            if orientation == gtk::Orientation::Horizontal && count > 1 {
                total += self.spacing.get() * (count - 1);
            }
            (total, total, -1, -1)
        }

        /// Each icon grows by a cosine bell centred on the pointer and pushes
        /// its neighbours apart; the row keeps its two ends where they rest,
        /// and everything is measured against the row at rest, so a growing
        /// icon never moves what it is measured from.
        fn size_allocate(&self, width: i32, height: i32, _baseline: i32) {
            let widget = self.obj();
            let spacing = self.spacing.get() as f64;
            let mut children = Vec::new();
            let mut child = widget.first_child();
            while let Some(c) = child {
                if c.is_visible() {
                    let (_, w, _, _) = c.measure(gtk::Orientation::Horizontal, -1);
                    children.push((c.clone(), w as f64));
                }
                child = c.next_sibling();
            }
            let mut lefts = Vec::with_capacity(children.len());
            let mut x = 0.0;
            for (_, w) in &children {
                lefts.push(x);
                x += w + spacing;
            }
            let slot = children.first().map(|(_, w)| w + spacing).unwrap_or(1.0);
            let top = (self.magnify.get() - 1.0).max(0.0) * self.swell.get();
            let gains: Vec<f64> = children
                .iter()
                .zip(&lefts)
                .map(|((_, w), left)| {
                    let away = (self.pointer.get() - (left + w / 2.0)).abs() / (REACH * slot);
                    if away >= 1.0 {
                        0.0
                    } else {
                        top * (1.0 + (PI * away).cos()) / 2.0
                    }
                })
                .collect();
            let mut sums = vec![0.0];
            for ((_, w), gain) in children.iter().zip(&gains) {
                let last = *sums.last().unwrap_or(&0.0);
                sums.push(last + gain * w);
            }
            let whole = *sums.last().unwrap_or(&0.0);
            let length = (width as f64).max(1.0);
            for (i, ((c, w), gain)) in children.iter().zip(&gains).enumerate() {
                let shift = sums[i] - whole * (lefts[i] / length).clamp(0.0, 1.0);
                let scale = 1.0 + gain;
                let (cw, h) = (*w as f32, height as f32);
                let transform = gsk::Transform::new()
                    .translate(&graphene::Point::new(
                        (lefts[i] + shift) as f32 + cw / 2.0,
                        h,
                    ))
                    .scale(scale as f32, scale as f32)
                    .translate(&graphene::Point::new(-cw / 2.0, -h));
                c.allocate(*w as i32, height, -1, Some(transform));
            }
        }
    }
}

glib::wrapper! {
    pub struct Shelf(ObjectSubclass<imp::Shelf>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Shelf {
    pub fn new(spacing: i32, magnify: f64) -> Self {
        let shelf: Self = glib::Object::new();
        shelf.imp().spacing.set(spacing);
        shelf.imp().magnify.set(magnify);
        shelf.set_overflow(gtk::Overflow::Visible);
        shelf
    }

    pub fn append(&self, child: &impl IsA<gtk::Widget>) {
        child.set_parent(self);
    }

    pub fn clear(&self) {
        while let Some(child) = self.first_child() {
            child.unparent();
        }
    }

    pub fn set_pointer(&self, x: f64) {
        self.imp().pointer.set(x);
        self.queue_allocate();
    }

    pub fn set_swell(&self, swell: f64) {
        self.imp().swell.set(swell);
        self.queue_allocate();
    }

    pub fn magnifies(&self) -> bool {
        self.imp().magnify.get() > 1.0
    }
}
