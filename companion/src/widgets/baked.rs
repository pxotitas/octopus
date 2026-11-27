//! Drawings that change rarely, rendered once into a texture and shown as a
//! picture after that: one image on the GPU instead of hundreds of shapes
//! rebuilt every frame.

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{gdk, graphene};

#[derive(Default)]
pub struct Baked {
    last: RefCell<Option<(u64, gdk::Texture)>>,
}

impl Baked {
    /// Draws `draw` into `snapshot`, from the texture when `key` (whatever
    /// the drawing depends on, size included) is the same as last time.
    pub fn snapshot(&self, widget: &impl IsA<gtk::Widget>, snapshot: &gtk::Snapshot, key: u64, draw: impl FnOnce(&gtk::Snapshot)) {
        let widget = widget.as_ref();
        let (w, h) = (widget.width() as f32, widget.height() as f32);
        let bounds = graphene::Rect::new(0.0, 0.0, w, h);
        if let Some((k, texture)) = self.last.borrow().as_ref() {
            if *k == key {
                snapshot.append_texture(texture, &bounds);
                return;
            }
        }
        let Some(renderer) = widget.native().and_then(|n| n.renderer()) else {
            draw(snapshot);
            return;
        };
        let scale = widget.scale_factor().max(1) as f32;
        let inner = gtk::Snapshot::new();
        inner.scale(scale, scale);
        draw(&inner);
        let Some(node) = inner.to_node() else { return };
        let texture = renderer.render_texture(&node, Some(&graphene::Rect::new(0.0, 0.0, w * scale, h * scale)));
        snapshot.append_texture(&texture, &bounds);
        *self.last.borrow_mut() = Some((key, texture));
    }

    pub fn forget(&self) {
        self.last.borrow_mut().take();
    }
}
