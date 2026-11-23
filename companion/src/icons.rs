use gtk::prelude::*;

/// A Material Symbols Rounded glyph, drawn from its ligature name.
pub fn icon(name: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(name));
    label.add_css_class("icon");
    label
}

pub fn button(name: &str, tooltip: &str) -> gtk::Button {
    let button = gtk::Button::new();
    button.set_child(Some(&icon(name)));
    button.add_css_class("icon-button");
    if !tooltip.is_empty() {
        button.set_tooltip_text(Some(tooltip));
    }
    button
}

pub fn set(button: &gtk::Button, name: &str) {
    if let Some(label) = button.child().and_downcast::<gtk::Label>() {
        label.set_label(name);
    }
}

pub fn volume(percent: u32, muted: bool) -> &'static str {
    match (muted, percent) {
        (true, _) | (_, 0) => "volume_off",
        (_, 1..=33) => "volume_mute",
        (_, 34..=66) => "volume_down",
        _ => "volume_up",
    }
}

pub fn wifi(signal: u32) -> &'static str {
    match signal {
        0..=24 => "network_wifi_1_bar",
        25..=49 => "network_wifi_2_bar",
        50..=74 => "network_wifi_3_bar",
        _ => "signal_wifi_4_bar",
    }
}
