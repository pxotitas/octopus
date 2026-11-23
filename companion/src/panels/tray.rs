use crate::i18n::t;
use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, glib, pango};

use crate::icons;
use crate::services::tray::{Item, MenuNode, Tray};
use crate::widgets;

/// The tray: every app's icon; a left click activates it, a right click
/// opens its menu here instead of the app's own D-Bus menu.
pub struct TrayPanel {
    pub root: gtk::Box,
    stack: gtk::Stack,
    menu: gtk::Box,
    title: gtk::Label,
    trail: RefCell<Vec<(String, Vec<MenuNode>)>>,
    current: RefCell<Option<Item>>,
    tray: Rc<Tray>,
    close: Rc<dyn Fn()>,
}

impl TrayPanel {
    pub fn new(tray: &Rc<Tray>, close: Rc<dyn Fn()>) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.add_css_class("surface");
        root.add_css_class("menu");
        root.add_css_class("tray-menu");

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::SlideLeftRight);
        stack.set_transition_duration(220);
        stack.set_vhomogeneous(false);
        stack.set_interpolate_size(true);

        let items_page = gtk::Box::new(gtk::Orientation::Vertical, 10);
        items_page.append(&widgets::heading(t("bandeja"), None));
        let grid = gtk::FlowBox::new();
        grid.set_selection_mode(gtk::SelectionMode::None);
        grid.set_max_children_per_line(5);
        grid.set_min_children_per_line(1);
        grid.set_column_spacing(6);
        grid.set_row_spacing(6);
        items_page.append(&grid);
        let empty = gtk::Label::new(Some(t("nenhum app na bandeja")));
        empty.add_css_class("dim");
        items_page.append(&empty);
        stack.add_named(&items_page, Some("items"));

        let menu_page = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let back = icons::button("arrow_back", t("voltar"));
        let title = gtk::Label::new(None);
        title.add_css_class("tray-title");
        title.set_xalign(0.0);
        title.set_ellipsize(pango::EllipsizeMode::End);
        header.append(&back);
        header.append(&title);
        menu_page.append(&header);
        let menu = gtk::Box::new(gtk::Orientation::Vertical, 2);
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .propagate_natural_height(true)
            .max_content_height(420)
            .child(&menu)
            .build();
        menu_page.append(&scroller);
        stack.add_named(&menu_page, Some("menu"));
        root.append(&stack);

        let panel = Rc::new(Self {
            root,
            stack,
            menu,
            title,
            trail: RefCell::new(Vec::new()),
            current: RefCell::new(None),
            tray: tray.clone(),
            close,
        });

        let weak = Rc::downgrade(&panel);
        back.connect_clicked(move |_| {
            if let Some(p) = weak.upgrade() {
                p.back();
            }
        });

        let weak = Rc::downgrade(&panel);
        tray.items.subscribe(move |items| {
            let Some(panel) = weak.upgrade() else { return };
            while let Some(child) = grid.first_child() {
                grid.remove(&child);
            }
            for item in items.iter().filter(|i| !i.passive) {
                grid.insert(&panel.button(item), -1);
            }
            empty.set_visible(items.iter().all(|i| i.passive));
        });
        panel
    }

    /// Back to the icons whenever the panel opens.
    pub fn reset(&self) {
        self.trail.borrow_mut().clear();
        self.stack.set_visible_child_name("items");
    }

    fn button(self: &Rc<Self>, item: &Item) -> gtk::Button {
        let button = gtk::Button::new();
        button.add_css_class("tray-item");
        button.set_tooltip_text(Some(&item.title));
        button.set_child(Some(&icon(item)));

        let click = gtk::GestureClick::new();
        click.set_button(0);
        let weak = Rc::downgrade(self);
        let item = item.clone();
        click.connect_released(move |gesture, _, _, _| {
            let Some(panel) = weak.upgrade() else { return };
            gesture.set_state(gtk::EventSequenceState::Claimed);
            match gesture.current_button() {
                gdk::BUTTON_SECONDARY => panel.open_menu(&item),
                gdk::BUTTON_MIDDLE => {
                    panel.tray.secondary(&item);
                    (panel.close)();
                }
                _ => {
                    let item = item.clone();
                    let weak = Rc::downgrade(&panel);
                    glib::spawn_future_local(async move {
                        let Some(panel) = weak.upgrade() else { return };
                        if panel.tray.activate(&item).await {
                            (panel.close)();
                        } else {
                            panel.open_menu(&item);
                        }
                    });
                }
            }
        });
        button.add_controller(click);
        button
    }

    fn open_menu(self: &Rc<Self>, item: &Item) {
        *self.current.borrow_mut() = Some(item.clone());
        let weak = Rc::downgrade(self);
        let item = item.clone();
        glib::spawn_future_local(async move {
            let Some(panel) = weak.upgrade() else { return };
            let Some(root) = panel.tray.menu(&item).await else {
                return;
            };
            panel.trail.borrow_mut().clear();
            panel.show_level(item.title.clone(), root.children);
        });
    }

    fn show_level(self: &Rc<Self>, title: String, nodes: Vec<MenuNode>) {
        self.title.set_label(&title.to_lowercase());
        widgets::clear(&self.menu);
        for node in &nodes {
            self.menu.append(&self.row(node));
        }
        self.trail.borrow_mut().push((title, nodes));
        self.stack.set_visible_child_name("menu");
    }

    fn back(self: &Rc<Self>) {
        let mut trail = self.trail.borrow_mut();
        trail.pop();
        let previous = trail.pop();
        drop(trail);
        match previous {
            Some((title, nodes)) => self.show_level(title, nodes),
            None => self.stack.set_visible_child_name("items"),
        }
    }

    fn row(self: &Rc<Self>, node: &MenuNode) -> gtk::Widget {
        if node.separator {
            let line = gtk::Separator::new(gtk::Orientation::Horizontal);
            line.add_css_class("tray-separator");
            return line.upcast();
        }
        let button = gtk::Button::new();
        button.add_css_class("menu-row");
        button.set_sensitive(node.enabled);
        let content = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        let lead = match &node.toggle {
            Some((kind, on)) if kind == "radio" => Some(if *on {
                "radio_button_checked"
            } else {
                "radio_button_unchecked"
            }),
            Some((_, on)) => Some(if *on {
                "check_box"
            } else {
                "check_box_outline_blank"
            }),
            None => None,
        };
        if let Some(glyph) = lead {
            content.append(&icons::icon(glyph));
        }
        let label = gtk::Label::new(Some(&node.label));
        label.set_xalign(0.0);
        label.set_hexpand(true);
        label.set_ellipsize(pango::EllipsizeMode::End);
        content.append(&label);
        if !node.children.is_empty() {
            content.append(&icons::icon("chevron_right"));
        }
        button.set_child(Some(&content));

        let weak = Rc::downgrade(self);
        let node = node.clone();
        button.connect_clicked(move |_| {
            let Some(panel) = weak.upgrade() else { return };
            if !node.children.is_empty() {
                panel.show_level(node.label.clone(), node.children.clone());
                return;
            }
            if let Some(item) = panel.current.borrow().as_ref() {
                panel.tray.click(item, node.id);
            }
            (panel.close)();
        });
        button.upcast()
    }
}

fn icon(item: &Item) -> gtk::Image {
    let image = if let Some((w, h, argb)) = &item.pixmap {
        let mut rgba = argb.clone();
        for px in rgba.chunks_exact_mut(4) {
            let (a, r, g, b) = (px[0], px[1], px[2], px[3]);
            px.copy_from_slice(&[r, g, b, a]);
        }
        let bytes = glib::Bytes::from_owned(rgba);
        let texture = gdk::MemoryTexture::new(
            *w,
            *h,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            (*w as usize) * 4,
        );
        gtk::Image::from_paintable(Some(&texture))
    } else if item.icon_name.starts_with('/') {
        gtk::Image::from_file(&item.icon_name)
    } else {
        if !item.icon_theme_path.is_empty() {
            if let Some(display) = gdk::Display::default() {
                let theme = gtk::IconTheme::for_display(&display);
                if !theme
                    .search_path()
                    .iter()
                    .any(|p| p.to_string_lossy() == item.icon_theme_path)
                {
                    theme.add_search_path(&item.icon_theme_path);
                }
            }
        }
        gtk::Image::from_icon_name(if item.icon_name.is_empty() {
            "application-x-executable"
        } else {
            &item.icon_name
        })
    };
    image.set_pixel_size(24);
    image
}
