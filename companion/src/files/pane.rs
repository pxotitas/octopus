//! One tab: a folder (or a set of results) shown as icons or as a list,
//! with its history, its sorting and filtering, and the selection.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use super::cells::{self, OnDrop};
use super::{file_of, state, ATTRIBUTES};
use crate::i18n::t;

pub struct Hooks {
    /// The folder changed.
    pub moved: Box<dyn Fn()>,
    /// The selection or the listing changed.
    pub changed: Box<dyn Fn()>,
    /// A file was opened (double click, enter).
    pub open: Box<dyn Fn(gio::File, gio::FileInfo)>,
    /// A right click: where, and whether it was on an item.
    pub menu: Box<dyn Fn(&gtk::Widget, f64, f64, bool)>,
    pub dropped: OnDrop,
}

pub struct Pane {
    pub root: gtk::Overlay,
    stack: gtk::Stack,
    grid: gtk::GridView,
    list: gtk::ColumnView,
    empty: gtk::Box,
    dir: gtk::DirectoryList,
    results: gio::ListStore,
    filter_model: gtk::FilterListModel,
    filter: gtk::CustomFilter,
    sorter: gtk::CustomSorter,
    pub selection: gtk::MultiSelection,
    location: RefCell<gio::File>,
    /// Set when the tab shows results (a search, a tag) instead of a folder.
    title: RefCell<Option<String>>,
    back: RefCell<Vec<gio::File>>,
    forward: RefCell<Vec<gio::File>>,
    query: Rc<RefCell<String>>,
    hidden: Rc<Cell<bool>>,
    sort: Rc<RefCell<(String, bool)>>,
    zoom: Rc<Cell<i32>>,
    hooks: Rc<Hooks>,
}

fn natural(a: &str, b: &str) -> std::cmp::Ordering {
    glib::FilenameCollationKey::from(a.to_lowercase()).cmp(&glib::FilenameCollationKey::from(b.to_lowercase()))
}

impl Pane {
    pub fn new(start: &gio::File, hooks: Hooks) -> Rc<Self> {
        let st = state::get();
        let hidden = Rc::new(Cell::new(st.hidden));
        let query: Rc<RefCell<String>> = Rc::default();
        let sort = Rc::new(RefCell::new((st.sort.clone(), st.descending)));
        let zoom = Rc::new(Cell::new(st.zoom));
        let hooks = Rc::new(hooks);

        let dir = gtk::DirectoryList::new(Some(ATTRIBUTES), Some(start));
        dir.set_monitored(true);
        let results = gio::ListStore::new::<gio::FileInfo>();

        let filter = {
            let (hidden, query) = (hidden.clone(), query.clone());
            gtk::CustomFilter::new(move |obj| {
                let Some(info) = obj.downcast_ref::<gio::FileInfo>() else { return false };
                if !hidden.get() && (info.is_hidden() || info.is_backup()) {
                    return false;
                }
                let q = query.borrow();
                q.is_empty() || info.display_name().to_lowercase().contains(&q.to_lowercase())
            })
        };
        let sorter = {
            let sort = sort.clone();
            gtk::CustomSorter::new(move |a, b| {
                let (Some(a), Some(b)) = (a.downcast_ref::<gio::FileInfo>(), b.downcast_ref::<gio::FileInfo>()) else {
                    return gtk::Ordering::Equal;
                };
                let dir_a = a.file_type() == gio::FileType::Directory;
                let dir_b = b.file_type() == gio::FileType::Directory;
                if dir_a != dir_b {
                    return if dir_a { gtk::Ordering::Smaller } else { gtk::Ordering::Larger };
                }
                let (key, descending) = sort.borrow().clone();
                let by_name = natural(&a.display_name(), &b.display_name());
                let order = match key.as_str() {
                    "size" => a.size().cmp(&b.size()).then(by_name),
                    "modified" => {
                        let m = |i: &gio::FileInfo| i.modification_date_time().map(|d| d.to_unix()).unwrap_or(0);
                        m(a).cmp(&m(b)).then(by_name)
                    }
                    "kind" => {
                        let k = |i: &gio::FileInfo| i.content_type().map(|c| c.to_string()).unwrap_or_default();
                        k(a).cmp(&k(b)).then(by_name)
                    }
                    _ => by_name,
                };
                (if descending { order.reverse() } else { order }).into()
            })
        };
        let filter_model = gtk::FilterListModel::new(Some(dir.clone()), Some(filter.clone()));
        let sorted = gtk::SortListModel::new(Some(filter_model.clone()), Some(sorter.clone()));
        let selection = gtk::MultiSelection::new(Some(sorted));

        let dropped = hooks.dropped.clone();
        let grid = gtk::GridView::new(Some(selection.clone()), Some(cells::grid_factory(zoom.clone(), dropped.clone())));
        grid.add_css_class("file-grid");
        grid.set_max_columns(64);
        grid.set_enable_rubberband(true);
        let list = gtk::ColumnView::new(Some(selection.clone()));
        list.add_css_class("file-list");
        list.set_enable_rubberband(true);
        list.set_reorderable(false);
        cells::list_columns(&list, dropped);

        let stack = gtk::Stack::new();
        stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        stack.set_transition_duration(140);
        for (name, view) in [("icons", grid.clone().upcast::<gtk::Widget>()), ("list", list.clone().upcast())] {
            let scroller = gtk::ScrolledWindow::builder().child(&view).hexpand(true).vexpand(true).build();
            stack.add_named(&scroller, Some(name));
        }

        let empty = gtk::Box::new(gtk::Orientation::Vertical, 8);
        empty.add_css_class("files-empty");
        empty.set_halign(gtk::Align::Center);
        empty.set_valign(gtk::Align::Center);
        empty.set_can_target(false);
        let shape = crate::widgets::shape::Shape::new(crate::widgets::shape::Kind::Cookie9, 96, true, Some(&crate::icons::icon("folder_open")));
        shape.add_css_class("files-empty-shape");
        let label = gtk::Label::new(Some(t("pasta vazia")));
        label.add_css_class("files-empty-label");
        empty.append(&shape);
        empty.append(&label);

        let root = gtk::Overlay::new();
        root.add_css_class("files-pane");
        root.set_child(Some(&stack));
        root.add_overlay(&empty);

        let pane = Rc::new(Self {
            root,
            stack,
            grid,
            list,
            empty,
            dir,
            results,
            filter_model,
            filter,
            sorter,
            selection,
            location: RefCell::new(start.clone()),
            title: RefCell::new(None),
            back: RefCell::default(),
            forward: RefCell::default(),
            query,
            hidden,
            sort,
            zoom,
            hooks,
        });
        pane.set_view(&st.view);
        pane.wire();
        pane
    }

    fn wire(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        let activate = move |pos: u32| {
            let Some(p) = weak.upgrade() else { return };
            let Some(info) = p.selection.item(pos).and_downcast::<gio::FileInfo>() else { return };
            if let Some(file) = file_of(&info) {
                (p.hooks.open)(file, info);
            }
        };
        let activate = Rc::new(activate);
        let a = activate.clone();
        self.grid.connect_activate(move |_, pos| a(pos));
        self.list.connect_activate(move |_, pos| activate(pos));

        let weak = Rc::downgrade(self);
        self.selection.connect_selection_changed(move |_, _, _| {
            if let Some(p) = weak.upgrade() {
                (p.hooks.changed)();
            }
        });
        let weak = Rc::downgrade(self);
        self.selection.connect_items_changed(move |_, _, _, _| {
            if let Some(p) = weak.upgrade() {
                p.update_empty();
                (p.hooks.changed)();
            }
        });
        let weak = Rc::downgrade(self);
        self.dir.connect_loading_notify(move |_| {
            if let Some(p) = weak.upgrade() {
                p.update_empty();
            }
        });

        for view in [self.grid.clone().upcast::<gtk::Widget>(), self.list.clone().upcast()] {
            // right click: on an item, it joins (or becomes) the selection
            let click = gtk::GestureClick::new();
            click.set_button(gdk::BUTTON_SECONDARY);
            let weak = Rc::downgrade(self);
            let v = view.clone();
            click.connect_pressed(move |g, _, x, y| {
                let Some(p) = weak.upgrade() else { return };
                g.set_state(gtk::EventSequenceState::Claimed);
                let on_item = match p.position_at(&v, x, y) {
                    Some(pos) => {
                        if !p.selection.is_selected(pos) {
                            p.selection.select_item(pos, true);
                        }
                        true
                    }
                    None => {
                        p.selection.unselect_all();
                        false
                    }
                };
                (p.hooks.menu)(&v, x, y, on_item);
            });
            view.add_controller(click);

            // a press on the empty space clears the selection
            let press = gtk::GestureClick::new();
            press.set_button(gdk::BUTTON_PRIMARY);
            let weak = Rc::downgrade(self);
            let v = view.clone();
            press.connect_pressed(move |_, _, x, y| {
                let Some(p) = weak.upgrade() else { return };
                if p.position_at(&v, x, y).is_none() {
                    p.selection.unselect_all();
                }
            });
            view.add_controller(press);

            // drag out: the selection, or the item under the pointer
            let source = gtk::DragSource::new();
            source.set_actions(gdk::DragAction::COPY | gdk::DragAction::MOVE);
            let weak = Rc::downgrade(self);
            let v = view.clone();
            source.connect_prepare(move |_, x, y| {
                let p = weak.upgrade()?;
                let pos = p.position_at(&v, x, y)?;
                if !p.selection.is_selected(pos) {
                    p.selection.select_item(pos, true);
                }
                let files = p.selected();
                (!files.is_empty()).then(|| gdk::ContentProvider::for_value(&gdk::FileList::from_array(&files).to_value()))
            });
            view.add_controller(source);
        }

        // drops on the empty space go into this folder
        let target = gtk::DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY | gdk::DragAction::MOVE);
        let weak = Rc::downgrade(self);
        target.connect_drop(move |t, value, _, _| {
            let Some(p) = weak.upgrade() else { return false };
            let Ok(list) = value.get::<gdk::FileList>() else { return false };
            if p.title.borrow().is_some() {
                return false;
            }
            let copy = t.current_drop().is_some_and(|d| d.actions() == gdk::DragAction::COPY);
            (p.hooks.dropped)(list.files(), p.location(), copy);
            true
        });
        self.stack.add_controller(target);
    }

    /// The item under a point of a view, climbing from the widget picked
    /// to the tile or row the factory made.
    fn position_at(&self, view: &gtk::Widget, x: f64, y: f64) -> Option<u32> {
        let mut widget = view.pick(x, y, gtk::PickFlags::DEFAULT)?;
        loop {
            if widget.has_css_class("file-tile") || widget.has_css_class("file-row") {
                break;
            }
            if &widget == view {
                return None;
            }
            widget = widget.parent()?;
        }
        let name = widget
            .last_child()
            .and_then(|l| l.first_child())
            .and_downcast::<gtk::Label>()
            .or_else(|| widget.first_child().and_then(|i| i.next_sibling()).and_downcast::<gtk::Label>())?
            .label();
        // a row's name sits right after its icon; a tile's under it
        (0..self.selection.n_items()).find(|&i| self.selection.item(i).and_downcast::<gio::FileInfo>().is_some_and(|f| f.display_name() == name))
    }

    fn update_empty(&self) {
        let empty = self.selection.n_items() == 0 && !self.dir.is_loading();
        self.empty.set_visible(empty);
    }

    pub fn location(&self) -> gio::File {
        self.location.borrow().clone()
    }

    /// The tab's title: the folder's name, or what the results are of.
    pub fn title(&self) -> String {
        self.title.borrow().clone().unwrap_or_else(|| super::display_name(&self.location()))
    }

    pub fn showing_results(&self) -> bool {
        self.title.borrow().is_some()
    }

    pub fn go(&self, file: &gio::File) {
        if !self.showing_results() && self.location().equal(file) {
            return;
        }
        if !self.showing_results() {
            self.back.borrow_mut().push(self.location());
        }
        self.forward.borrow_mut().clear();
        self.show(file);
    }

    fn show(&self, file: &gio::File) {
        *self.title.borrow_mut() = None;
        *self.location.borrow_mut() = file.clone();
        self.query.borrow_mut().clear();
        self.filter_model.set_model(Some(&self.dir));
        self.dir.set_file(Some(file));
        self.filter.changed(gtk::FilterChange::Different);
        self.selection.unselect_all();
        for scroller in [self.grid.parent(), self.list.parent()].into_iter().flatten() {
            if let Some(s) = scroller.downcast_ref::<gtk::ScrolledWindow>() {
                s.vadjustment().set_value(0.0);
            }
        }
        (self.hooks.moved)();
    }

    pub fn back(&self) {
        let Some(previous) = self.back.borrow_mut().pop() else { return };
        self.forward.borrow_mut().push(self.location());
        self.show(&previous);
    }

    pub fn forward(&self) {
        let Some(next) = self.forward.borrow_mut().pop() else { return };
        self.back.borrow_mut().push(self.location());
        self.show(&next);
    }

    pub fn up(&self) {
        if let Some(parent) = self.location().parent() {
            self.go(&parent);
        }
    }

    pub fn can_back(&self) -> bool {
        !self.back.borrow().is_empty()
    }

    pub fn can_forward(&self) -> bool {
        !self.forward.borrow().is_empty()
    }

    /// Shows a list of files instead of a folder (search results, a tag).
    pub fn show_results(self: &Rc<Self>, title: &str, files: Vec<gio::File>) {
        if !self.showing_results() {
            self.back.borrow_mut().push(self.location());
        }
        *self.title.borrow_mut() = Some(title.to_owned());
        self.results.remove_all();
        self.filter_model.set_model(Some(&self.results));
        (self.hooks.moved)();
        self.append_results(files);
    }

    pub fn append_results(self: &Rc<Self>, files: Vec<gio::File>) {
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            for file in files {
                let Ok(info) = file.query_info_future(ATTRIBUTES, gio::FileQueryInfoFlags::NONE, glib::Priority::DEFAULT).await else { continue };
                info.set_attribute_object("standard::file", &file);
                let Some(p) = weak.upgrade() else { return };
                if !p.showing_results() {
                    return;
                }
                p.results.append(&info);
            }
        });
    }

    pub fn selected(&self) -> Vec<gio::File> {
        let set = self.selection.selection();
        (0..set.size() as u32)
            .filter_map(|i| self.selection.item(set.nth(i)).and_downcast::<gio::FileInfo>())
            .filter_map(|info| file_of(&info))
            .collect()
    }

    pub fn selected_infos(&self) -> Vec<gio::FileInfo> {
        let set = self.selection.selection();
        (0..set.size() as u32).filter_map(|i| self.selection.item(set.nth(i)).and_downcast::<gio::FileInfo>()).collect()
    }

    /// Every file shown, in order (for quick look's arrows).
    pub fn shown(&self) -> Vec<(gio::File, gio::FileInfo)> {
        (0..self.selection.n_items())
            .filter_map(|i| self.selection.item(i).and_downcast::<gio::FileInfo>())
            .filter_map(|info| file_of(&info).map(|f| (f, info)))
            .collect()
    }

    pub fn select_file(&self, file: &gio::File) {
        let pos = (0..self.selection.n_items()).find(|&i| {
            self.selection.item(i).and_downcast::<gio::FileInfo>().and_then(|i| file_of(&i)).is_some_and(|f| f.equal(file))
        });
        if let Some(pos) = pos {
            self.selection.select_item(pos, true);
            match self.view().as_str() {
                "list" => self.list.scroll_to(pos, None, gtk::ListScrollFlags::FOCUS, None),
                _ => self.grid.scroll_to(pos, gtk::ListScrollFlags::FOCUS, None),
            }
        }
    }

    pub fn count(&self) -> u32 {
        self.selection.n_items()
    }

    pub fn set_query(&self, text: &str) {
        *self.query.borrow_mut() = text.to_owned();
        self.filter.changed(gtk::FilterChange::Different);
    }

    pub fn view(&self) -> String {
        self.stack.visible_child_name().map(|s| s.to_string()).unwrap_or_default()
    }

    pub fn set_view(&self, view: &str) {
        let view = if view == "list" { "list" } else { "icons" };
        self.stack.set_visible_child_name(view);
        let focus: gtk::Widget = if view == "list" { self.list.clone().upcast() } else { self.grid.clone().upcast() };
        focus.grab_focus();
    }

    pub fn hidden(&self) -> bool {
        self.hidden.get()
    }

    pub fn set_hidden(&self, on: bool) {
        self.hidden.set(on);
        self.filter.changed(gtk::FilterChange::Different);
    }

    pub fn sort(&self) -> (String, bool) {
        self.sort.borrow().clone()
    }

    pub fn set_sort(&self, key: &str, descending: bool) {
        *self.sort.borrow_mut() = (key.to_owned(), descending);
        self.sorter.changed(gtk::SorterChange::Different);
    }

    pub fn set_zoom(&self, size: i32) {
        self.zoom.set(size.clamp(32, 192));
        // rebinding redraws every tile at the new size
        self.grid.set_model(None::<&gtk::SelectionModel>);
        self.grid.set_model(Some(&self.selection));
    }

    pub fn zoom(&self) -> i32 {
        self.zoom.get()
    }

    pub fn focus(&self) {
        self.set_view(&self.view());
    }

    pub fn select_all(&self) {
        self.selection.select_all();
    }
}
