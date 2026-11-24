use crate::i18n::t;
use std::rc::Rc;

use gtk::pango;
use gtk::prelude::*;

use crate::icons;
use crate::services::cava::Cava;
use crate::services::media::Media;
use crate::widgets::art;
use crate::widgets::shape::{Kind, Shape};
use crate::widgets::wave::Wave;

pub enum Size {
    Card,
    Hero,
}

/// Cover, title, artist, transport and the wave timeline. A placeholder
/// takes its place when nothing is playing. With `cava`, a thin visualizer
/// runs along the card's bottom edge while music plays.
pub fn player(media: &Rc<Media>, size: Size, cava: Option<&Rc<Cava>>) -> gtk::Overlay {
    let hero = matches!(size, Size::Hero);
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_vhomogeneous(false);
    stack.set_interpolate_size(true);

    let idle = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    idle.add_css_class("media-idle");
    idle.append(&icons::icon("music_off"));
    idle.append(&gtk::Label::new(Some(t("nada tocando"))));
    stack.add_named(&idle, Some("idle"));

    let root = gtk::Box::new(
        if hero {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        },
        if hero { 16 } else { 14 },
    );
    let (cover, picture, _) = art::frame(
        if hero { "media-art-hero" } else { "media-art" },
        if hero { 240 } else { 84 },
        "music_note",
    );
    let spinner = if hero {
        cover.remove_css_class("media-art-hero");
        let shape = Shape::new(
            crate::widgets::shape::random_kind(&[]),
            240,
            true,
            Some(&cover),
        );
        shape.add_css_class("media-art-hero");
        root.append(&shape);
        Some(shape)
    } else {
        root.append(&cover);
        None
    };

    let side = gtk::Box::new(gtk::Orientation::Vertical, 4);
    side.set_hexpand(true);
    side.set_valign(gtk::Align::Center);
    let title = gtk::Label::new(None);
    title.add_css_class("media-title");
    title.set_ellipsize(pango::EllipsizeMode::End);
    let artist = gtk::Label::new(None);
    artist.add_css_class("media-artist");
    artist.set_ellipsize(pango::EllipsizeMode::End);
    for label in [&title, &artist] {
        label.set_xalign(if hero { 0.5 } else { 0.0 });
        side.append(label);
    }

    let wave = Wave::new(media);
    side.append(&wave.area);

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.set_halign(if hero {
        gtk::Align::Center
    } else {
        gtk::Align::Start
    });
    controls.add_css_class("media-controls");
    let previous = icons::button("skip_previous", t("anterior"));
    let play_icon = icons::icon("play_arrow");
    let play_shape = Shape::new(Kind::Circle, 40, true, Some(&play_icon));
    play_shape.add_css_class("media-play-shape");
    let play = gtk::Button::new();
    play.set_child(Some(&play_shape));
    play.add_css_class("media-play");
    play.set_tooltip_text(Some(t("tocar")));
    let next = icons::button("skip_next", t("próxima"));
    controls.append(&previous);
    controls.append(&play);
    controls.append(&next);
    side.append(&controls);
    root.append(&side);
    stack.add_named(&root, Some("player"));

    {
        let m = media.clone();
        previous.connect_clicked(move |_| m.previous());
        let m = media.clone();
        play.connect_clicked(move |_| m.play_pause());
        let m = media.clone();
        next.connect_clicked(move |_| m.next());
    }

    let overlay = gtk::Overlay::new();
    overlay.add_css_class("media");
    overlay.add_css_class(if hero { "media-hero" } else { "media-card" });
    overlay.set_child(Some(&stack));

    let bars = cava.map(|cava| {
        let bars = crate::widgets::cava::bars(cava);
        bars.add_css_class("media-cava");
        bars.set_content_height(10);
        bars.set_valign(gtk::Align::End);
        bars.set_hexpand(true);
        bars.set_can_target(false);
        overlay.add_overlay(&bars);
        bars
    });

    {
        let stack = stack.clone();
        let wave = wave.clone();
        let song = std::cell::RefCell::new(String::new());
        media.track.subscribe(move |track| {
            let _ = &wave;
            let Some(t) = track else {
                stack.set_visible_child_name("idle");
                if let Some(bars) = &bars {
                    bars.set_visible(false);
                }
                return;
            };
            stack.set_visible_child_name("player");
            title.set_label(&t.title);
            artist.set_label(&t.artist);
            artist.set_visible(!t.artist.is_empty());
            play_icon.set_label(if t.playing { "pause" } else { "play_arrow" });
            play_shape.morph(if t.playing {
                Kind::Cookie9
            } else {
                Kind::Circle
            });
            if let Some(shape) = &spinner {
                shape.set_spinning(t.playing);
                let now = format!("{}\n{}\n{}", t.player, t.artist, t.title);
                if *song.borrow() != now {
                    if !song.borrow().is_empty() {
                        shape.morph(crate::widgets::shape::random_kind(&[shape.kind()]));
                    }
                    *song.borrow_mut() = now;
                }
            }
            if let Some(bars) = &bars {
                bars.set_visible(t.playing);
            }
            art::set_file(&picture, t.art.as_deref());
            wave.area.set_visible(t.length > 0.0);
        });
    }

    overlay
}
