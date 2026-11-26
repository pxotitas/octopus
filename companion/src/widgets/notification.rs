use crate::i18n::t;
use std::rc::Rc;

use gtk::pango;
use gtk::prelude::*;

use crate::icons;
use crate::services::notifications::{Notification, Notifications};
use crate::widgets::art;
use crate::widgets::shape::{Kind, Shape};

/// One notification: the sender's picture when there is one (with the app
/// icon as a badge), else the app icon.
pub fn card(
    n: &Rc<Notification>,
    service: &Rc<Notifications>,
    on_done: impl Fn() + 'static,
) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    root.add_css_class("notification");
    if n.critical {
        root.add_css_class("critical");
    }

    root.append(&avatar(n));

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_hexpand(true);

    let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    let app = gtk::Label::new(Some(&format!(
        "{} · {}",
        n.app_name.to_lowercase(),
        n.time.format("%H:%M")
    )));
    app.add_css_class("notification-app");
    app.set_xalign(0.0);
    app.set_hexpand(true);
    app.set_ellipsize(pango::EllipsizeMode::End);
    app.set_max_width_chars(1);
    let close = icons::button("close", t("dispensar"));
    close.add_css_class("notification-close");
    header.append(&app);
    header.append(&close);
    text.append(&header);

    if !n.summary.is_empty() {
        let summary = gtk::Label::new(Some(&n.summary));
        summary.add_css_class("notification-summary");
        summary.set_xalign(0.0);
        summary.set_wrap(true);
        summary.set_wrap_mode(pango::WrapMode::WordChar);
        summary.set_lines(2);
        summary.set_ellipsize(pango::EllipsizeMode::End);
        summary.set_max_width_chars(1);
        text.append(&summary);
    }
    if !n.body.is_empty() {
        let body = gtk::Label::new(None);
        let markup = sanitize(&n.body);
        if pango::parse_markup(&markup, '\0').is_ok() {
            body.set_markup(&markup);
        } else {
            body.set_text(&n.body);
        }
        body.add_css_class("notification-body");
        body.set_xalign(0.0);
        body.set_wrap(true);
        body.set_wrap_mode(pango::WrapMode::WordChar);
        body.set_lines(3);
        body.set_ellipsize(pango::EllipsizeMode::End);
        body.set_max_width_chars(1);
        text.append(&body);
    }

    let actions: Vec<_> = n
        .actions
        .iter()
        .filter(|(key, _)| key != "default")
        .cloned()
        .collect();
    if !actions.is_empty() {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        row.add_css_class("notification-actions");
        for (key, label) in actions {
            let button = gtk::Button::with_label(&label.to_lowercase());
            button.add_css_class("pill");
            let (service, id) = (service.clone(), n.id);
            button.connect_clicked(move |_| service.invoke(id, &key));
            row.append(&button);
        }
        text.append(&row);
    }
    root.append(&text);

    let on_done = Rc::new(on_done);
    {
        let (service, id, on_done) = (service.clone(), n.id, on_done.clone());
        close.connect_clicked(move |_| {
            service.dismiss(id);
            on_done();
        });
    }
    if let Some(action) = n.default_action().map(str::to_owned) {
        root.add_css_class("activatable");
        let click = gtk::GestureClick::new();
        let (service, id) = (service.clone(), n.id);
        click.connect_released(move |gesture, _, _, _| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            service.invoke(id, &action);
            on_done();
        });
        root.add_controller(click);
    }
    root
}

/// The body's markup cut down to what the notification spec allows (bold,
/// italic, underline, links), so no app can blow a card up with a giant
/// `<span size>` or a font of its own. Entities pass through untouched.
fn sanitize(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut rest = body;
    while let Some(start) = rest.find('<') {
        out.push_str(&rest[..start]);
        let Some(len) = rest[start..].find('>') else {
            out.push_str(&gtk::glib::markup_escape_text(&rest[start..]));
            return out;
        };
        let tag = &rest[start + 1..start + len];
        let closing = tag.starts_with('/');
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase();
        if matches!(name.as_str(), "b" | "i" | "u") {
            out.push_str(if closing { "</" } else { "<" });
            out.push_str(&name);
            out.push('>');
        }
        rest = &rest[start + len + 1..];
    }
    out.push_str(rest);
    out
}

fn avatar(n: &Notification) -> gtk::Widget {
    let texture = n.image.as_ref().and_then(art::texture);
    let Some(texture) = texture else {
        let icon = art::app_icon(&n.app_icon, 26);
        let shape = Shape::new(Kind::Cookie6, 48, true, Some(&icon));
        shape.add_css_class("notification-icon");
        shape.set_valign(gtk::Align::Start);
        return shape.upcast();
    };

    let overlay = gtk::Overlay::new();
    overlay.set_valign(gtk::Align::Start);
    let picture = gtk::Image::from_paintable(Some(&texture));
    picture.set_pixel_size(48);
    let avatar = Shape::new(Kind::Circle, 48, false, Some(&picture));
    overlay.set_child(Some(&avatar));
    if !n.app_icon.is_empty() {
        let badge = art::app_icon(&n.app_icon, 16);
        badge.add_css_class("notification-badge");
        badge.set_halign(gtk::Align::End);
        badge.set_valign(gtk::Align::End);
        overlay.add_overlay(&badge);
    }
    overlay.upcast()
}

/// Every notification, newest first, rebuilt when the list changes.
pub fn list(service: &Rc<Notifications>) -> gtk::Stack {
    list_sized(service, 340)
}

/// `list`, scrolling past `max_height`.
pub fn list_sized(service: &Rc<Notifications>, max_height: i32) -> gtk::Stack {
    let stack = gtk::Stack::new();
    stack.set_transition_type(gtk::StackTransitionType::Crossfade);
    stack.set_vexpand(true);

    let empty = gtk::Box::new(gtk::Orientation::Vertical, 8);
    empty.add_css_class("empty");
    empty.set_valign(gtk::Align::Center);
    let icon = Shape::new(
        Kind::Sunny,
        64,
        true,
        Some(&icons::icon("notifications_off")),
    );
    icon.add_css_class("empty-icon");
    let label = gtk::Label::new(Some(t("tudo em dia")));
    empty.append(&icon);
    empty.append(&label);

    let column = gtk::Box::new(gtk::Orientation::Vertical, 8);
    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .propagate_natural_height(true)
        .max_content_height(max_height)
        .child(&column)
        .vexpand(true)
        .build();

    stack.add_named(&empty, Some("empty"));
    stack.add_named(&scroller, Some("list"));

    let weak = Rc::downgrade(service);
    let view = stack.clone();
    service.list.subscribe(move |list| {
        let Some(service) = weak.upgrade() else {
            return;
        };
        while let Some(child) = column.first_child() {
            column.remove(&child);
        }
        for n in list {
            column.append(&card(n, &service, || {}));
        }
        view.set_visible_child_name(if list.is_empty() { "empty" } else { "list" });
    });
    stack
}
