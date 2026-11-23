use std::rc::Rc;

use gtk::prelude::*;

use crate::icons;

/// A quick-settings tile: the body toggles, the chevron opens the full menu.
pub struct Tile {
    pub root: gtk::Box,
    toggle: gtk::Button,
    icon: gtk::Label,
    subtitle: gtk::Label,
}

impl Tile {
    pub fn new(icon: &str, title: &str, with_menu: bool) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        root.add_css_class("tile");
        root.set_hexpand(true);

        let toggle = gtk::Button::new();
        toggle.add_css_class("tile-toggle");
        toggle.set_hexpand(true);
        let body = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        let icon = icons::icon(icon);
        icon.add_css_class("tile-icon");
        let text = gtk::Box::new(gtk::Orientation::Vertical, 0);
        text.set_valign(gtk::Align::Center);
        let title = gtk::Label::new(Some(title));
        title.add_css_class("tile-title");
        title.set_xalign(0.0);
        let subtitle = gtk::Label::new(None);
        subtitle.add_css_class("tile-subtitle");
        subtitle.set_xalign(0.0);
        subtitle.set_ellipsize(gtk::pango::EllipsizeMode::End);
        subtitle.set_max_width_chars(12);
        text.append(&title);
        text.append(&subtitle);
        body.append(&icon);
        body.append(&text);
        toggle.set_child(Some(&body));
        root.append(&toggle);

        let tile = Rc::new(Self {
            root,
            toggle,
            icon,
            subtitle,
        });
        if with_menu {
            let more = icons::button("chevron_right", "");
            more.add_css_class("tile-more");
            tile.root.append(&more);
        }
        tile
    }

    pub fn on_toggle(&self, f: impl Fn() + 'static) {
        self.toggle.connect_clicked(move |_| f());
    }

    pub fn on_menu(&self, f: impl Fn() + 'static) {
        if let Some(more) = self
            .root
            .last_child()
            .and_downcast::<gtk::Button>()
            .filter(|b| b.has_css_class("tile-more"))
        {
            more.connect_clicked(move |_| f());
        }
    }

    pub fn set(&self, on: bool, icon: &str, subtitle: &str) {
        if on {
            self.root.add_css_class("on");
        } else {
            self.root.remove_css_class("on");
        }
        self.icon.set_label(icon);
        self.subtitle.set_label(subtitle);
    }
}
