//! The latest commits of a github user, on the desktop.

use gtk::pango;
use gtk::prelude::*;

use crate::i18n::t;
use crate::icons;
use crate::services::github::{ago, Github};
use crate::widgets::shape::{Kind, Shape};

pub fn build(user: &str) -> gtk::Box {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
    root.add_css_class("desktop-widget");
    root.add_css_class("desktop-github");
    root.set_size_request(360, -1);

    let head = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    let badge = Shape::new(Kind::SoftBurst, 40, true, Some(&icons::icon("commit")));
    badge.add_css_class("desktop-github-badge");
    head.append(&badge);
    let titles = gtk::Box::new(gtk::Orientation::Vertical, 0);
    titles.set_valign(gtk::Align::Center);
    let title = gtk::Label::new(Some(t("commits")));
    title.add_css_class("desktop-title");
    title.set_xalign(0.0);
    let who = gtk::Label::new(Some(&format!("@{user}")));
    who.add_css_class("dim");
    who.set_xalign(0.0);
    titles.append(&title);
    titles.append(&who);
    head.append(&titles);
    root.append(&head);

    let list = gtk::Box::new(gtk::Orientation::Vertical, 4);
    root.append(&list);

    let github = Github::new(user);
    let keep = github.clone();
    github.commits.subscribe(move |commits| {
        let _ = &keep;
        crate::widgets::clear(&list);
        let Some(commits) = commits else {
            let waiting = gtk::Label::new(Some(t("buscando commits…")));
            waiting.add_css_class("dim");
            list.append(&waiting);
            return;
        };
        for c in commits.iter().take(5) {
            let row = gtk::Box::new(gtk::Orientation::Vertical, 1);
            let meta = gtk::Label::new(Some(&format!("{} · {}", c.repo, ago(&c.when))));
            meta.add_css_class("commit-meta");
            meta.set_xalign(0.0);
            meta.set_ellipsize(pango::EllipsizeMode::End);
            let message = gtk::Label::new(Some(&c.message));
            message.add_css_class("commit-message");
            message.set_xalign(0.0);
            message.set_ellipsize(pango::EllipsizeMode::End);
            message.set_max_width_chars(1);
            message.set_hexpand(true);
            row.append(&meta);
            row.append(&message);
            let button = gtk::Button::new();
            button.add_css_class("commit-row");
            button.set_child(Some(&row));
            button.set_tooltip_text(Some(&c.sha));
            let url = c.url.clone();
            button.connect_clicked(move |_| crate::exec::spawn("xdg-open", &[&url]));
            list.append(&button);
        }
    });
    root
}
