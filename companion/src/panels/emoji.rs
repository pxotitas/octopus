use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use gtk::prelude::*;
use gtk::{gdk, glib};

use crate::exec;
use crate::i18n::t;

/// The emoji picker: search by name or keyword (in the shell's language and
/// in English), Enter or a click types the emoji and leaves it copied.
pub struct Emoji {
    pub root: gtk::Box,
    search: gtk::SearchEntry,
    grid: gtk::GridView,
    selection: gtk::SingleSelection,
    store: gtk::StringList,
    loaded: RefCell<bool>,
    close: Rc<dyn Fn()>,
}

fn field(line: &str, index: usize) -> &str {
    line.split('\t').nth(index).unwrap_or("")
}

impl Emoji {
    pub fn new(close: Rc<dyn Fn()>) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
        root.add_css_class("surface");
        root.add_css_class("emoji");

        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some(t("buscar emoji")));
        root.append(&search);

        let store = gtk::StringList::new(&[]);
        let query: Rc<RefCell<String>> = Rc::default();
        let filter = {
            let query = query.clone();
            gtk::CustomFilter::new(move |item| {
                let q = query.borrow();
                if q.is_empty() {
                    return true;
                }
                item.downcast_ref::<gtk::StringObject>().is_some_and(|s| {
                    let line = s.string();
                    field(&line, 1).to_lowercase().contains(q.as_str())
                        || field(&line, 2).contains(q.as_str())
                })
            })
        };
        let filtered = gtk::FilterListModel::new(Some(store.clone()), Some(filter.clone()));
        let selection = gtk::SingleSelection::new(Some(filtered));

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_setup(|_, item| {
            let label = gtk::Label::new(None);
            label.add_css_class("emoji-glyph");
            item.downcast_ref::<gtk::ListItem>()
                .unwrap()
                .set_child(Some(&label));
        });
        factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let Some(line) = item.item().and_downcast::<gtk::StringObject>() else {
                return;
            };
            if let Some(label) = item.child().and_downcast::<gtk::Label>() {
                let line = line.string();
                label.set_label(field(&line, 0));
                label.set_tooltip_text(Some(field(&line, 1)));
            }
        });

        let grid = gtk::GridView::new(Some(selection.clone()), Some(factory));
        grid.set_max_columns(9);
        grid.set_min_columns(9);
        grid.set_single_click_activate(true);
        grid.add_css_class("emoji-grid");
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_height(360)
            .max_content_height(360)
            .child(&grid)
            .build();
        root.append(&scroller);

        let name = gtk::Label::new(None);
        name.add_css_class("dim");
        name.set_xalign(0.0);
        root.append(&name);

        let panel = Rc::new(Self {
            root,
            search: search.clone(),
            grid: grid.clone(),
            selection: selection.clone(),
            store,
            loaded: RefCell::new(false),
            close,
        });

        {
            let (query, filter, selection) = (query.clone(), filter.clone(), selection.clone());
            search.connect_search_changed(move |entry| {
                *query.borrow_mut() = entry.text().to_lowercase();
                filter.changed(gtk::FilterChange::Different);
                selection.set_selected(0);
            });
        }
        selection.connect_selected_item_notify(move |sel| {
            let text = sel
                .selected_item()
                .and_downcast::<gtk::StringObject>()
                .map(|s| field(&s.string(), 1).to_owned());
            name.set_label(&text.unwrap_or_default());
        });

        let weak = Rc::downgrade(&panel);
        search.connect_activate(move |_| {
            if let Some(p) = weak.upgrade() {
                p.pick(p.selection.selected());
            }
        });
        let weak = Rc::downgrade(&panel);
        grid.connect_activate(move |_, position| {
            if let Some(p) = weak.upgrade() {
                p.pick(position);
            }
        });

        // Arrows move through the grid while the search keeps the keyboard.
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(&panel);
        keys.connect_key_pressed(move |_, key, _, _| {
            let Some(p) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            let step: i64 = match key {
                gdk::Key::Down => 9,
                gdk::Key::Up => -9,
                gdk::Key::Tab => 1,
                gdk::Key::ISO_Left_Tab => -1,
                _ => return glib::Propagation::Proceed,
            };
            let count = p.selection.n_items() as i64;
            if count > 0 {
                let current = p.selection.selected() as i64;
                let next = (current + step).clamp(0, count - 1) as u32;
                p.selection.set_selected(next);
                p.grid.scroll_to(next, gtk::ListScrollFlags::NONE, None);
            }
            glib::Propagation::Stop
        });
        panel.root.add_controller(keys);
        panel
    }

    pub fn unload(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || {
            if let Some(p) = weak.upgrade().filter(|p| !p.root.is_mapped()) {
                p.store.splice(0, p.store.n_items(), &[] as &[&str]);
                p.loaded.replace(false);
            }
        });
    }

    pub fn open(self: &Rc<Self>) {
        self.search.set_text("");
        self.search.grab_focus();
        if self.loaded.replace(true) {
            return;
        }
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let language = if crate::i18n::english() { "en" } else { "pt" };
            let cache = exec::cache_dir().join(format!("emoji-{language}.tsv"));
            let text = match std::fs::read_to_string(&cache) {
                Ok(text) if !text.is_empty() => text,
                _ => {
                    let script = exec::script("emoji.py");
                    let text = exec::output("python3", &[&script, language, "--tsv"])
                        .await
                        .unwrap_or_default();
                    let _ = std::fs::create_dir_all(exec::cache_dir());
                    let _ = std::fs::write(&cache, &text);
                    text
                }
            };
            let Some(p) = weak.upgrade() else { return };
            let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
            p.store.splice(0, p.store.n_items(), &lines);
            p.selection.set_selected(0);
        });
    }

    fn pick(&self, position: u32) {
        let Some(line) = self
            .selection
            .model()
            .and_then(|m| m.item(position))
            .and_downcast::<gtk::StringObject>()
        else {
            return;
        };
        let glyph = field(&line.string(), 0).to_owned();
        (self.close)();
        exec::spawn("wl-copy", &[&glyph]);
        glib::timeout_add_local_once(Duration::from_millis(180), move || {
            exec::spawn("wtype", &[&glyph]);
        });
    }
}
