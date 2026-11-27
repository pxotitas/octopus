use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::services::notifications::Image;

/// Picture clipped by its CSS radius; `fallback` is a Material icon shown
/// when there is no image.
pub fn frame(class: &str, size: i32, fallback: &str) -> (gtk::Overlay, gtk::Picture, gtk::Label) {
    let overlay = gtk::Overlay::new();
    overlay.add_css_class(class);
    overlay.set_size_request(size, size);
    overlay.set_overflow(gtk::Overflow::Hidden);
    overlay.set_valign(gtk::Align::Start);
    let placeholder = crate::icons::icon(fallback);
    placeholder.add_css_class("art-fallback");
    overlay.set_child(Some(&placeholder));
    let picture = gtk::Picture::new();
    picture.set_content_fit(gtk::ContentFit::Cover);
    picture.set_can_shrink(true);
    overlay.add_overlay(&picture);
    (overlay, picture, placeholder)
}

/// Covers come as big as 3000px; shown at most at 240.
pub fn set_file(picture: &gtk::Picture, path: Option<&str>) {
    let path = path.map(|p| std::path::PathBuf::from(p.strip_prefix("file://").unwrap_or(p)));
    crate::widgets::thumb::set(picture, path.as_deref(), 240, false);
}

pub fn texture(image: &Image) -> Option<gdk::Texture> {
    match image {
        Image::Data {
            width,
            height,
            stride,
            alpha,
            bytes,
        } => {
            let format = if *alpha {
                gdk::MemoryFormat::R8g8b8a8
            } else {
                gdk::MemoryFormat::R8g8b8
            };
            let needed = (*stride as usize) * (*height as usize - 1)
                + (*width as usize) * if *alpha { 4 } else { 3 };
            if *width <= 0 || *height <= 0 || bytes.len() < needed {
                return None;
            }
            let bytes = glib::Bytes::from(bytes.as_slice());
            Some(
                gdk::MemoryTexture::new(*width, *height, format, &bytes, *stride as usize).upcast(),
            )
        }
        Image::Path(path) => {
            let path = path.strip_prefix("file://").unwrap_or(path);
            gdk::Texture::from_filename(path).ok()
        }
    }
}

/// An app icon: an absolute path, or a name in the icon theme.
pub fn app_icon(name: &str, size: i32) -> gtk::Image {
    let image = if name.starts_with('/') || name.starts_with("file://") {
        gtk::Image::from_file(name.strip_prefix("file://").unwrap_or(name))
    } else if name.is_empty() {
        gtk::Image::from_icon_name("dialog-information-symbolic")
    } else {
        gtk::Image::from_icon_name(name)
    };
    image.set_pixel_size(size);
    image
}
