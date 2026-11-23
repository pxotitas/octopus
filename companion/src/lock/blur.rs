use gtk::gdk;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;
use gtk::prelude::*;

/// A soft, dim, desaturated copy of an image: decoded small, box-blurred
/// three times, and left for the GPU to scale up, which blurs it further.
pub fn blurred(
    path: &str,
    width: i32,
    radius: usize,
    dim: f32,
    saturation: f32,
) -> Option<gdk::Texture> {
    let pixbuf = Pixbuf::from_file_at_scale(path, width, -1, true)
        .ok()?
        .add_alpha(false, 0, 0, 0)
        .ok()?;
    let (w, h, stride) = (
        pixbuf.width() as usize,
        pixbuf.height() as usize,
        pixbuf.rowstride() as usize,
    );
    let mut pixels = pixbuf.read_pixel_bytes().to_vec();

    for _ in 0..3 {
        box_pass(&mut pixels, w, h, stride, radius, true);
        box_pass(&mut pixels, w, h, stride, radius, false);
    }
    for y in 0..h {
        for x in 0..w {
            let i = y * stride + x * 4;
            let (r, g, b) = (pixels[i] as f32, pixels[i + 1] as f32, pixels[i + 2] as f32);
            let grey = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            for (c, v) in [r, g, b].into_iter().enumerate() {
                pixels[i + c] = ((grey + (v - grey) * saturation) * dim).clamp(0.0, 255.0) as u8;
            }
        }
    }
    let bytes = glib::Bytes::from_owned(pixels);
    Some(
        gdk::MemoryTexture::new(
            w as i32,
            h as i32,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            stride,
        )
        .upcast(),
    )
}

fn box_pass(pixels: &mut [u8], w: usize, h: usize, stride: usize, radius: usize, horizontal: bool) {
    let (lines, length) = if horizontal { (h, w) } else { (w, h) };
    let index = |line: usize, at: usize| {
        if horizontal {
            line * stride + at * 4
        } else {
            at * stride + line * 4
        }
    };
    let mut row = vec![[0i32; 3]; length];
    for line in 0..lines {
        for (at, slot) in row.iter_mut().enumerate() {
            let i = index(line, at);
            *slot = [pixels[i] as i32, pixels[i + 1] as i32, pixels[i + 2] as i32];
        }
        let window = (2 * radius + 1) as i32;
        let mut sum = [0i32; 3];
        for k in 0..=2 * radius {
            let at = k.saturating_sub(radius).min(length - 1);
            for c in 0..3 {
                sum[c] += row[at][c];
            }
        }
        for at in 0..length {
            let i = index(line, at);
            for c in 0..3 {
                pixels[i + c] = (sum[c] / window).clamp(0, 255) as u8;
            }
            let out = at.saturating_sub(radius);
            let inc = (at + radius + 1).min(length - 1);
            for c in 0..3 {
                sum[c] = sum[c] + row[inc][c] - row[out][c];
            }
        }
    }
}
