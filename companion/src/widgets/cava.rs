use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::services::cava::{Cava, BARS};

/// Rounded bars in the widget's CSS `color`.
pub fn bars(cava: &Rc<Cava>) -> gtk::DrawingArea {
    let area = gtk::DrawingArea::new();
    area.add_css_class("cava");
    area.set_content_height(28);
    area.set_content_width(BARS as i32 * 6);

    let values: Rc<RefCell<Vec<f64>>> = Rc::new(RefCell::new(vec![0.0; BARS]));
    {
        let values = values.clone();
        area.set_draw_func(move |area, cr, width, height| {
            let color = area.color();
            cr.set_source_rgba(
                color.red() as f64,
                color.green() as f64,
                color.blue() as f64,
                color.alpha() as f64,
            );
            let values = values.borrow();
            let slot = width as f64 / values.len() as f64;
            let bar = (slot * 0.55).max(2.0);
            for (i, v) in values.iter().enumerate() {
                let h = (v * height as f64).max(bar);
                let x = i as f64 * slot + (slot - bar) / 2.0;
                let y = (height as f64 - h) / 2.0;
                rounded(cr, x, y, bar, h, bar / 2.0);
                let _ = cr.fill();
            }
        });
    }
    {
        let area = area.clone();
        cava.bars.subscribe(move |fresh| {
            *values.borrow_mut() = fresh.clone();
            area.queue_draw();
        });
    }
    area
}

fn rounded(cr: &gtk::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    use std::f64::consts::PI;
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, PI / 2.0);
    cr.arc(x + r, y + h - r, r, PI / 2.0, PI);
    cr.arc(x + r, y + r, r, PI, 3.0 * PI / 2.0);
    cr.close_path();
}
