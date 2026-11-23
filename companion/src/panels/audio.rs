use crate::i18n::t;
use std::rc::Rc;

use gtk::pango;
use gtk::prelude::*;

use crate::icons;
use crate::services::audio::{Audio, Device, Kind};
use crate::widgets::{self, slider::Slider};

pub fn build(audio: &Rc<Audio>) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.add_css_class("surface");
    root.add_css_class("menu");
    root.add_css_class("audio-menu");

    for kind in [Kind::Sink, Kind::Source] {
        let (title, icon) = match kind {
            Kind::Sink => (t("saída"), "volume_up"),
            Kind::Source => (t("microfone"), "mic"),
        };
        root.append(&widgets::heading(title, None));

        let slider = {
            let audio = audio.clone();
            Slider::new(icon, t("silenciar"), move |v| audio.set_volume(kind, v))
        };
        {
            let audio = audio.clone();
            slider.on_icon(move || audio.toggle_mute(kind));
        }
        let percent = gtk::Label::new(None);
        percent.add_css_class("percent");
        slider.root.append(&percent);
        root.append(&slider.root);

        let devices = gtk::Box::new(gtk::Orientation::Vertical, 4);
        root.append(&devices);

        let weak = Rc::downgrade(audio);
        audio.state.subscribe(move |state| {
            let Some(audio) = weak.upgrade() else { return };
            let (current, list, default) = match kind {
                Kind::Sink => (state.sink(), &state.sinks, &state.default_sink),
                Kind::Source => (state.source(), &state.sources, &state.default_source),
            };
            if let Some(device) = current {
                let glyph = match kind {
                    Kind::Sink => icons::volume(device.volume, device.muted),
                    Kind::Source if device.muted => "mic_off",
                    Kind::Source => "mic",
                };
                slider.set(device.volume, glyph);
                slider.set_muted(device.muted);
                percent.set_label(&format!("{}%", device.volume));
            }
            widgets::clear(&devices);
            if list.len() > 1 {
                for device in list {
                    devices.append(&row(device, &device.name == default, kind, &audio));
                }
            }
        });
    }

    let mixer = gtk::Button::with_label(t("mixer"));
    mixer.add_css_class("pill");
    mixer.set_halign(gtk::Align::End);
    mixer.connect_clicked(|_| crate::exec::spawn("pavucontrol", &[]));
    root.append(&mixer);

    root
}

fn row(device: &Device, selected: bool, kind: Kind, audio: &Rc<Audio>) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("menu-row");
    if selected {
        button.add_css_class("active");
    }
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    content.append(&icons::icon(if selected {
        "radio_button_checked"
    } else {
        "radio_button_unchecked"
    }));
    let name = gtk::Label::new(Some(&device.description));
    name.set_xalign(0.0);
    name.set_hexpand(true);
    name.set_ellipsize(pango::EllipsizeMode::End);
    content.append(&name);
    button.set_child(Some(&content));
    let (audio, target) = (audio.clone(), device.name.clone());
    button.connect_clicked(move |_| audio.set_default(kind, &target));
    button
}
