//! The pieces every settings page is made of: M3 list groups, rows with an
//! icon and a control, segmented buttons, sliders.

use std::rc::Rc;

use gtk::glib;
use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;

/// A page: title, subtitle, then its groups.
pub fn page(title: &str, subtitle: &str) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 6);
    column.add_css_class("settings-page");
    let heading = gtk::Label::new(Some(title));
    heading.add_css_class("settings-title");
    heading.set_xalign(0.0);
    let sub = gtk::Label::new(Some(subtitle));
    sub.add_css_class("settings-subtitle");
    sub.set_xalign(0.0);
    sub.set_wrap(true);
    column.append(&heading);
    column.append(&sub);
    column
}

pub fn scroll(child: &impl IsA<gtk::Widget>) -> gtk::ScrolledWindow {
    gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(child)
        .build()
}

/// A small heading over a group.
pub fn label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("settings-section");
    label.set_xalign(0.0);
    label
}

/// Items in one rounded group, M3 list style: the first and last round off.
pub fn group(items: &[&gtk::Widget]) -> gtk::Box {
    let column = gtk::Box::new(gtk::Orientation::Vertical, 2);
    column.add_css_class("settings-group");
    for item in items {
        column.append(*item);
    }
    column
}

fn lead(icon: &str, title: &str, subtitle: &str) -> gtk::Box {
    let line = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    let glyph = icons::icon(icon);
    glyph.add_css_class("settings-icon");
    glyph.set_valign(gtk::Align::Center);
    line.append(&glyph);
    let texts = gtk::Box::new(gtk::Orientation::Vertical, 2);
    texts.set_valign(gtk::Align::Center);
    texts.set_hexpand(true);
    let name = gtk::Label::new(Some(title));
    name.add_css_class("settings-item-title");
    name.set_xalign(0.0);
    texts.append(&name);
    if !subtitle.is_empty() {
        let sub = gtk::Label::new(Some(subtitle));
        sub.add_css_class("settings-item-sub");
        sub.set_xalign(0.0);
        sub.set_wrap(true);
        texts.append(&sub);
    }
    line.append(&texts);
    line
}

/// Icon and text with the control at the end of the line.
pub fn inline(icon: &str, title: &str, subtitle: &str, control: &impl IsA<gtk::Widget>) -> gtk::Widget {
    let line = lead(icon, title, subtitle);
    line.add_css_class("settings-item");
    control.as_ref().set_valign(gtk::Align::Center);
    line.append(control);
    line.upcast()
}

/// Icon and text with a wide control under them.
pub fn stacked(icon: &str, title: &str, subtitle: &str, control: &impl IsA<gtk::Widget>) -> gtk::Widget {
    let item = gtk::Box::new(gtk::Orientation::Vertical, 12);
    item.add_css_class("settings-item");
    item.append(&lead(icon, title, subtitle));
    item.append(control);
    item.upcast()
}

/// M3 segmented buttons.
pub fn segmented(options: &[&str], selected: usize, on_change: impl Fn(usize) + 'static) -> gtk::Box {
    let group = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    group.add_css_class("segmented");
    group.set_hexpand(true);
    group.set_homogeneous(true);
    let on_change = Rc::new(on_change);
    let mut first: Option<gtk::ToggleButton> = None;
    for (i, option) in options.iter().enumerate() {
        let button = gtk::ToggleButton::with_label(option);
        button.set_group(first.as_ref());
        button.set_active(i == selected);
        let on_change = on_change.clone();
        button.connect_toggled(move |b| {
            if b.is_active() {
                on_change(i);
            }
        });
        first.get_or_insert(button.clone());
        group.append(&button);
    }
    group
}

/// Segmented buttons over (value, label) choices, translated.
pub fn choices(options: &[(&'static str, &'static str)], current: &str, on_change: impl Fn(&'static str) + 'static) -> gtk::Box {
    let labels: Vec<&str> = options.iter().map(|(_, l)| t(l)).collect();
    let selected = options.iter().position(|(v, _)| *v == current).unwrap_or(0);
    let values: Vec<&'static str> = options.iter().map(|(v, _)| *v).collect();
    segmented(&labels, selected, move |i| on_change(values[i]))
}

pub fn switch(active: bool, on_change: impl Fn(bool) + 'static) -> gtk::Switch {
    let switch = gtk::Switch::new();
    switch.set_active(active);
    switch.connect_active_notify(move |s| on_change(s.is_active()));
    switch
}

/// A slider with its value printed beside it.
pub fn slider(min: f64, max: f64, step: f64, value: f64, show: impl Fn(f64) -> String + 'static, on_change: impl Fn(f64) + 'static) -> gtk::Box {
    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, min, max, step);
    scale.set_value(value);
    scale.set_hexpand(true);
    scale.set_draw_value(false);
    // the wheel scrolls the page; it only changes a slider by dragging
    let wheel = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
    wheel.set_propagation_phase(gtk::PropagationPhase::Capture);
    wheel.connect_scroll(|controller, _, dy| {
        let page = controller.widget().and_then(|w| w.ancestor(gtk::ScrolledWindow::static_type())).and_downcast::<gtk::ScrolledWindow>();
        if let Some(page) = page {
            let adj = page.vadjustment();
            let unit = if controller.unit() == gtk::gdk::ScrollUnit::Surface { 1.0 } else { 50.0 };
            adj.set_value(adj.value() + dy * unit);
        }
        glib::Propagation::Stop
    });
    scale.add_controller(wheel);
    let shown = gtk::Label::new(Some(&show(value)));
    shown.add_css_class("percent");
    let holder = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    holder.append(&scale);
    holder.append(&shown);
    scale.connect_value_changed(move |s| {
        let v = (s.value() / step).round() * step;
        shown.set_label(&show(v));
        on_change(v);
    });
    holder
}

pub fn entry(text: &str, placeholder: &str, on_change: impl Fn(String) + 'static) -> gtk::Entry {
    let entry = gtk::Entry::new();
    entry.set_text(text);
    entry.set_placeholder_text(Some(placeholder));
    entry.connect_changed(move |e| on_change(e.text().to_string()));
    entry
}

pub fn hint(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("settings-hint");
    label.set_xalign(0.0);
    label.set_wrap(true);
    label
}

/// A tonal button with an icon.
pub fn action(icon: &str, text: &str, on_click: impl Fn() + 'static) -> gtk::Button {
    let button = gtk::Button::new();
    button.add_css_class("settings-action");
    let content = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    content.append(&icons::icon(icon));
    content.append(&gtk::Label::new(Some(text)));
    button.set_child(Some(&content));
    button.connect_clicked(move |_| on_click());
    button
}
