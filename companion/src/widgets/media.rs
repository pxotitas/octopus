use std::rc::Rc;

use gtk::glib;
use gtk::pango;
use gtk::prelude::*;

use crate::icons;
use crate::services::media::Media;
use crate::widgets::art;

pub enum Size {
    Card,
    Hero,
}

/// Cover, title, artist, transport and a seekable progress bar. Hidden
/// (with a placeholder) when nothing is playing.
pub fn player(media: &Rc<Media>, size: Size) -> gtk::Stack {
    let hero = matches!(size, Size::Hero);
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_vhomogeneous(false);
    stack.set_interpolate_size(true);
    stack.add_css_class("media");
    stack.add_css_class(if hero { "media-hero" } else { "media-card" });

    let idle = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    idle.add_css_class("media-idle");
    idle.append(&icons::icon("music_off"));
    idle.append(&gtk::Label::new(Some("nada tocando")));
    stack.add_named(&idle, Some("idle"));

    let root = gtk::Box::new(
        if hero {
            gtk::Orientation::Vertical
        } else {
            gtk::Orientation::Horizontal
        },
        if hero { 16 } else { 14 },
    );
    let (cover, picture, _) = art::frame("media-art", if hero { 240 } else { 88 }, "music_note");
    if hero {
        cover.set_halign(gtk::Align::Center);
    }
    root.append(&cover);

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

    let progress = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
    progress.set_draw_value(false);
    progress.add_css_class("media-progress");
    side.append(&progress);

    let controls = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    controls.set_halign(if hero {
        gtk::Align::Center
    } else {
        gtk::Align::Start
    });
    controls.add_css_class("media-controls");
    let previous = icons::button("skip_previous", "anterior");
    let play = icons::button("play_arrow", "tocar");
    play.add_css_class("media-play");
    let next = icons::button("skip_next", "próxima");
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
        let m = media.clone();
        progress.connect_change_value(move |_, _, value| {
            m.seek(value);
            glib::Propagation::Proceed
        });
    }

    {
        let progress = progress.clone();
        let stack = stack.clone();
        media.track.subscribe(move |track| {
            let Some(t) = track else {
                stack.set_visible_child_name("idle");
                return;
            };
            stack.set_visible_child_name("player");
            title.set_label(&t.title);
            artist.set_label(&t.artist);
            artist.set_visible(!t.artist.is_empty());
            icons::set(&play, if t.playing { "pause" } else { "play_arrow" });
            if t.playing {
                play.add_css_class("playing");
            } else {
                play.remove_css_class("playing");
            }
            art::set_file(&picture, t.art.as_deref());
            progress.set_range(0.0, t.length.max(1.0));
            progress.set_visible(t.length > 0.0);
        });
    }
    media
        .position
        .subscribe(move |seconds| progress.set_value(*seconds));

    stack
}
