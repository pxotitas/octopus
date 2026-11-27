//! The column view, like the Finder's: every folder from home (or the root)
//! down to the one open, side by side. The last column is the pane's own
//! listing, so the selection, menus and drags work as in the other views;
//! a file picked there gets a preview column after it.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use super::{cells, file_of, thumbs, ATTRIBUTES};

const WIDTH: i32 = 230;

pub struct Columns {
    pub root: gtk::ScrolledWindow,
    row: gtk::Box,
    /// The pane's listing, always the last folder column.
    pub last: gtk::ListView,
    preview: gtk::Box,
    ancestors: RefCell<Vec<gtk::Widget>>,
    stick: Rc<std::cell::Cell<bool>>,
    filter: gtk::CustomFilter,
    sorter: gtk::Sorter,
    /// A click on a column: go to this folder, then pick this file in it.
    go: Rc<dyn Fn(gio::File, Option<gio::File>)>,
}

fn is_dir(info: &gio::FileInfo) -> bool {
    info.file_type() == gio::FileType::Directory
}

/// The row: icon, name, a chevron on folders. Same shape as the list
/// view's name cell, so the pane finds the item under the pointer.
fn factory(on_drop: Option<cells::OnDrop>) -> gtk::SignalListItemFactory {
    let f = gtk::SignalListItemFactory::new();
    f.connect_setup(move |_, obj| {
        let Some(item) = obj.downcast_ref::<gtk::ListItem>() else { return };
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.add_css_class("file-row");
        row.add_css_class("column-row");
        let image = gtk::Image::new();
        image.set_pixel_size(18);
        let label = gtk::Label::new(None);
        label.set_ellipsize(gtk::pango::EllipsizeMode::Middle);
        label.set_xalign(0.0);
        label.set_hexpand(true);
        let chevron = crate::icons::icon("chevron_right");
        chevron.add_css_class("column-chevron");
        row.append(&image);
        row.append(&label);
        row.append(&chevron);
        if let Some(on_drop) = &on_drop {
            cells::drop_into(&row, item, on_drop);
        }
        item.set_child(Some(&row));
    });
    f.connect_bind(|_, obj| {
        let Some(item) = obj.downcast_ref::<gtk::ListItem>() else { return };
        let Some(info) = item.item().and_downcast::<gio::FileInfo>() else { return };
        let Some(row) = item.child().and_downcast::<gtk::Box>() else { return };
        let Some(image) = row.first_child().and_downcast::<gtk::Image>() else { return };
        match info.symbolic_icon().or_else(|| info.icon()) {
            Some(icon) => image.set_from_gicon(&icon),
            None => image.set_icon_name(Some("text-x-generic-symbolic")),
        }
        if let Some(label) = image.next_sibling().and_downcast::<gtk::Label>() {
            label.set_label(&info.display_name());
            if let Some(chevron) = label.next_sibling() {
                chevron.set_opacity(if is_dir(&info) { 1.0 } else { 0.0 });
            }
        }
        row.set_opacity(if info.is_hidden() { 0.6 } else { 1.0 });
    });
    f
}

fn column(list: &gtk::ListView) -> gtk::ScrolledWindow {
    list.add_css_class("files-column");
    let scroller = gtk::ScrolledWindow::builder()
        .child(list)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .build();
    scroller.set_size_request(WIDTH, -1);
    scroller.add_css_class("files-column-scroller");
    scroller
}

/// Where the columns start: home when the folder is under it.
fn top(location: &gio::File) -> gio::File {
    let home = gio::File::for_path(glib::home_dir());
    if location.equal(&home) || location.has_prefix(&home) {
        home
    } else {
        gio::File::for_path("/")
    }
}

impl Columns {
    pub fn new(
        selection: &gtk::MultiSelection,
        filter: gtk::CustomFilter,
        sorter: gtk::Sorter,
        on_drop: cells::OnDrop,
        go: Rc<dyn Fn(gio::File, Option<gio::File>)>,
    ) -> Rc<Self> {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        row.add_css_class("files-columns");
        let last = gtk::ListView::new(Some(selection.clone()), Some(factory(Some(on_drop))));
        last.set_enable_rubberband(true);
        let preview = gtk::Box::new(gtk::Orientation::Vertical, 10);
        preview.add_css_class("files-preview");
        preview.set_size_request(WIDTH + 40, -1);
        preview.set_visible(false);
        row.append(&column(&last));
        row.append(&preview);
        let root = gtk::ScrolledWindow::builder()
            .child(&row)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .hexpand(true)
            .vexpand(true)
            .build();
        let columns = Rc::new(Self {
            root,
            row,
            last,
            preview,
            ancestors: RefCell::default(),
            stick: Rc::default(),
            filter,
            sorter,
            go,
        });
        columns.wire(selection);
        let stick = columns.stick.clone();
        columns.root.hadjustment().connect_upper_notify(move |adj| {
            if stick.get() {
                adj.set_value(adj.upper() - adj.page_size());
            }
        });
        let stick = columns.stick.clone();
        columns.root.hadjustment().connect_page_size_notify(move |adj| {
            if stick.get() {
                adj.set_value(adj.upper() - adj.page_size());
            }
        });
        let by_hand = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::HORIZONTAL);
        let stick = columns.stick.clone();
        by_hand.connect_scroll(move |_, _, _| {
            stick.set(false);
            glib::Propagation::Proceed
        });
        columns.root.add_controller(by_hand);
        columns
    }

    fn wire(self: &Rc<Self>, selection: &gtk::MultiSelection) {
        // one folder picked with the pointer opens beside it
        let click = gtk::GestureClick::new();
        let weak = Rc::downgrade(self);
        let sel = selection.clone();
        click.connect_released(move |g, n, _, _| {
            let Some(c) = weak.upgrade() else { return };
            let state = g.current_event_state();
            if n != 1 || state.intersects(gtk::gdk::ModifierType::CONTROL_MASK | gtk::gdk::ModifierType::SHIFT_MASK) {
                return;
            }
            let picked = sel.selection();
            if picked.size() != 1 {
                return;
            }
            let Some(info) = sel.item(picked.nth(0)).and_downcast::<gio::FileInfo>() else { return };
            if is_dir(&info) {
                if let Some(folder) = file_of(&info) {
                    let go = c.go.clone();
                    glib::idle_add_local_once(move || go(folder, None));
                }
            }
        });
        self.last.add_controller(click);

        // right goes into the folder picked, left back to the one before
        let keys = gtk::EventControllerKey::new();
        let weak = Rc::downgrade(self);
        let sel = selection.clone();
        keys.connect_key_pressed(move |_, key, _, _| {
            let Some(c) = weak.upgrade() else { return glib::Propagation::Proceed };
            match key {
                gtk::gdk::Key::Right => {
                    let picked = sel.selection();
                    let info = (picked.size() == 1).then(|| sel.item(picked.nth(0)).and_downcast::<gio::FileInfo>()).flatten();
                    if let Some(folder) = info.filter(is_dir).and_then(|i| file_of(&i)) {
                        (c.go)(folder, None);
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                }
                gtk::gdk::Key::Left => {
                    let here = c.shown_folder();
                    if let Some((here, parent)) = here.and_then(|h| h.parent().map(|p| (h, p))) {
                        if c.ancestors.borrow().is_empty() {
                            return glib::Propagation::Proceed;
                        }
                        (c.go)(parent, Some(here));
                        return glib::Propagation::Stop;
                    }
                    glib::Propagation::Proceed
                }
                _ => glib::Propagation::Proceed,
            }
        });
        self.last.add_controller(keys);

        let weak = Rc::downgrade(self);
        selection.connect_selection_changed(move |sel, _, _| {
            if let Some(c) = weak.upgrade() {
                c.show_preview(sel);
            }
        });
    }

    fn shown_folder(&self) -> Option<gio::File> {
        unsafe { self.root.data::<gio::File>("folder").map(|f| f.as_ref().clone()) }
    }

    /// One column for each folder above `location`, each with the next one
    /// on the way down picked. Results (no folder) show just the listing.
    pub fn show(self: &Rc<Self>, location: Option<&gio::File>) {
        for old in self.ancestors.borrow_mut().drain(..) {
            self.row.remove(&old);
        }
        self.preview.set_visible(false);
        let Some(location) = location else {
            unsafe { self.root.steal_data::<gio::File>("folder") };
            return;
        };
        unsafe { self.root.set_data("folder", location.clone()) };
        let start = top(location);
        let mut chain = vec![location.clone()];
        let mut at = location.clone();
        while !at.equal(&start) {
            match at.parent() {
                Some(parent) => {
                    chain.push(parent.clone());
                    at = parent;
                }
                None => break,
            }
        }
        chain.reverse();
        // chain: start .. location; every folder but the last gets a column
        let mut before: Option<gtk::Widget> = None;
        for pair in chain.windows(2) {
            let (folder, next) = (&pair[0], &pair[1]);
            let widget: gtk::Widget = self.ancestor(folder, next).upcast();
            self.row.insert_child_after(&widget, before.as_ref());
            before = Some(widget.clone());
            self.ancestors.borrow_mut().push(widget);
        }
        self.scroll_to_end();
    }

    fn ancestor(self: &Rc<Self>, folder: &gio::File, next: &gio::File) -> gtk::ScrolledWindow {
        let dir = gtk::DirectoryList::new(Some(ATTRIBUTES), Some(folder));
        let filtered = gtk::FilterListModel::new(Some(dir.clone()), Some(self.filter.clone()));
        let sorted = gtk::SortListModel::new(Some(filtered), Some(self.sorter.clone()));
        let selection = gtk::SingleSelection::new(Some(sorted));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);
        let list = gtk::ListView::new(Some(selection.clone()), Some(factory(None)));
        list.add_css_class("ancestor");

        // keeps the way down picked while the listing fills and sorts
        let wanted = next.clone();
        let holds = move |sel: &gtk::SingleSelection| {
            let picked = sel.selected_item().and_downcast::<gio::FileInfo>().and_then(|f| file_of(&f));
            if picked.is_some_and(|f| f.equal(&wanted)) {
                return;
            }
            let found = (0..sel.n_items()).find(|&i| {
                sel.item(i).and_downcast::<gio::FileInfo>().and_then(|f| file_of(&f)).is_some_and(|f| f.equal(&wanted))
            });
            if let Some(i) = found {
                sel.set_selected(i);
            }
        };
        selection.connect_items_changed(move |sel, _, _, _| holds(sel));

        let weak = Rc::downgrade(self);
        let (base, chosen) = (folder.clone(), next.clone());
        // only a click goes somewhere: the listing moving the pick around
        // while it loads doesn't
        let click = gtk::GestureClick::new();
        let sel = selection.clone();
        click.connect_released(move |_, _, _, _| {
            let Some(c) = weak.upgrade() else { return };
            let Some(info) = sel.selected_item().and_downcast::<gio::FileInfo>() else { return };
            let Some(file) = file_of(&info) else { return };
            let (base, chosen) = (base.clone(), chosen.clone());
            // the columns are rebuilt: not from inside this list's own signal
            glib::idle_add_local_once(move || {
                if file.equal(&chosen) {
                    (c.go)(chosen, None);
                } else if is_dir(&info) {
                    (c.go)(file, None);
                } else {
                    (c.go)(base, Some(file));
                }
            });
        });
        list.add_controller(click);
        column(&list)
    }

    /// To the right edge, and kept there as columns come and take their
    /// width, until the pointer scrolls it.
    fn scroll_to_end(&self) {
        self.stick.set(true);
        let adj = self.root.hadjustment();
        adj.set_value(adj.upper() - adj.page_size());
    }

    /// A single file picked: its picture (or icon) big, and what it is.
    fn show_preview(&self, sel: &gtk::MultiSelection) {
        while let Some(child) = self.preview.first_child() {
            self.preview.remove(&child);
        }
        let picked = sel.selection();
        let info = (picked.size() == 1).then(|| sel.item(picked.nth(0)).and_downcast::<gio::FileInfo>()).flatten();
        let Some(info) = info.filter(|i| !is_dir(i)) else {
            self.preview.set_visible(false);
            return;
        };
        let image = gtk::Image::new();
        image.set_pixel_size(160);
        image.add_css_class("preview-icon");
        match info.icon() {
            Some(icon) => image.set_from_gicon(&icon),
            None => image.set_icon_name(Some("text-x-generic")),
        }
        if let Some(file) = file_of(&info) {
            let ct = info.content_type().map(|c| c.to_string()).unwrap_or_default();
            if thumbs::wanted(&ct) {
                let weak = image.downgrade();
                thumbs::request(&file, &info, move |texture| {
                    if let (Some(image), Some(texture)) = (weak.upgrade(), texture) {
                        image.set_paintable(Some(&texture));
                        image.add_css_class("thumb");
                    }
                });
            }
        }
        let name = gtk::Label::new(Some(&info.display_name()));
        name.add_css_class("preview-name");
        name.set_wrap(true);
        name.set_wrap_mode(gtk::pango::WrapMode::WordChar);
        name.set_justify(gtk::Justification::Center);
        let kind = gtk::Label::new(Some(&format!("{} · {}", cells::kind(&info), cells::size(&info))));
        kind.add_css_class("dim");
        kind.set_wrap(true);
        let when = gtk::Label::new(Some(&cells::modified(&info)));
        when.add_css_class("dim");
        self.preview.append(&image);
        self.preview.append(&name);
        self.preview.append(&kind);
        self.preview.append(&when);
        self.preview.set_visible(true);
        self.scroll_to_end();
    }
}
