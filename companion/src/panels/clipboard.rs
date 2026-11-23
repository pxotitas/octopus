use std::io::Write;
use std::process::{Command, Stdio};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib, pango};

use crate::exec;
use crate::icons;
use crate::widgets;

const LIMIT: usize = 150;

/// cliphist's history: type to filter, Enter copies, Delete forgets. Images
/// show as thumbnails.
pub struct Clipboard {
    pub root: gtk::Box,
    search: gtk::SearchEntry,
    list: gtk::ListBox,
    close: Rc<dyn Fn()>,
}

impl Clipboard {
    pub fn new(close: Rc<dyn Fn()>) -> Rc<Self> {
        let root = gtk::Box::new(gtk::Orientation::Vertical, 10);
        root.add_css_class("surface");
        root.add_css_class("clipboard");

        let wipe = icons::button("delete_sweep", "apagar tudo");
        root.append(&widgets::heading(
            "área de transferência",
            Some(wipe.upcast_ref()),
        ));

        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some("buscar"));
        root.append(&search);

        let list = gtk::ListBox::new();
        list.add_css_class("clip-list");
        list.set_selection_mode(gtk::SelectionMode::Browse);
        let scroller = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .min_content_height(420)
            .max_content_height(420)
            .child(&list)
            .build();
        root.append(&scroller);

        let panel = Rc::new(Self {
            root,
            search: search.clone(),
            list: list.clone(),
            close,
        });

        {
            let search = search.clone();
            list.set_filter_func(move |row| {
                let query = search.text().to_lowercase();
                query.is_empty()
                    || unsafe {
                        row.data::<String>("search")
                            .is_some_and(|t| t.as_ref().contains(&query))
                    }
            });
        }
        {
            let list = list.clone();
            search.connect_search_changed(move |_| {
                list.invalidate_filter();
                if let Some(first) = first_visible(&list) {
                    list.select_row(Some(&first));
                }
            });
        }
        let weak = Rc::downgrade(&panel);
        search.connect_activate(move |_| {
            if let Some(p) = weak.upgrade() {
                if let Some(row) = p.list.selected_row().or_else(|| first_visible(&p.list)) {
                    p.copy(&row);
                }
            }
        });
        let weak = Rc::downgrade(&panel);
        list.connect_row_activated(move |_, row| {
            if let Some(p) = weak.upgrade() {
                p.copy(row);
            }
        });

        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(&panel);
        keys.connect_key_pressed(move |_, key, _, _| {
            let Some(p) = weak.upgrade() else {
                return glib::Propagation::Proceed;
            };
            match key {
                gdk::Key::Down | gdk::Key::Up => {
                    p.step(if key == gdk::Key::Down { 1 } else { -1 });
                    glib::Propagation::Stop
                }
                gdk::Key::Delete => {
                    if let Some(row) = p.list.selected_row() {
                        p.forget(&row);
                    }
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed,
            }
        });
        panel.root.add_controller(keys);

        let weak = Rc::downgrade(&panel);
        wipe.connect_clicked(move |_| {
            exec::spawn("cliphist", &["wipe"]);
            if let Some(p) = weak.upgrade() {
                widgets_clear(&p.list);
            }
        });
        panel
    }

    pub fn load(self: &Rc<Self>) {
        self.search.set_text("");
        self.search.grab_focus();
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let listing = exec::output("cliphist", &["list"])
                .await
                .unwrap_or_default();
            let Some(p) = weak.upgrade() else { return };
            widgets_clear(&p.list);
            for line in listing.lines().take(LIMIT) {
                p.list.append(&row(line));
            }
            if let Some(first) = p.list.row_at_index(0) {
                p.list.select_row(Some(&first));
            }
        });
    }

    fn step(&self, by: i32) {
        let mut index = self.list.selected_row().map_or(-1, |r| r.index());
        loop {
            index += by;
            let Some(row) = self.list.row_at_index(index) else {
                return;
            };
            if row.is_child_visible() {
                self.list.select_row(Some(&row));
                row.grab_focus();
                if let Some(text) = self.search.delegate().and_downcast::<gtk::Text>() {
                    text.grab_focus_without_selecting();
                }
                return;
            }
        }
    }

    fn copy(&self, row: &gtk::ListBoxRow) {
        let line = line_of(row);
        gio::spawn_blocking(move || {
            if let Some(bytes) = decode(&line) {
                if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(&bytes);
                    }
                    let _ = child.wait();
                }
            }
        });
        (self.close)();
    }

    fn forget(&self, row: &gtk::ListBoxRow) {
        let line = line_of(row);
        let index = row.index();
        self.list.remove(row);
        if let Some(next) = self
            .list
            .row_at_index(index)
            .or_else(|| self.list.row_at_index(index - 1))
        {
            self.list.select_row(Some(&next));
        }
        gio::spawn_blocking(move || pipe("cliphist", &["delete"], line.as_bytes()));
    }
}

fn widgets_clear(list: &gtk::ListBox) {
    list.remove_all();
}

fn first_visible(list: &gtk::ListBox) -> Option<gtk::ListBoxRow> {
    let mut i = 0;
    while let Some(row) = list.row_at_index(i) {
        if row.is_child_visible() {
            return Some(row);
        }
        i += 1;
    }
    None
}

fn line_of(row: &gtk::ListBoxRow) -> String {
    unsafe {
        row.data::<String>("line")
            .map(|p| p.as_ref().clone())
            .unwrap_or_default()
    }
}

fn pipe(program: &str, args: &[&str], input: &[u8]) -> Option<Vec<u8>> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(input).ok()?;
    let out = child.wait_with_output().ok()?;
    out.status.success().then_some(out.stdout)
}

fn decode(line: &str) -> Option<Vec<u8>> {
    pipe("cliphist", &["decode"], line.as_bytes())
}

fn row(line: &str) -> gtk::ListBoxRow {
    let (id, content) = line.split_once('\t').unwrap_or((line, ""));
    let row = gtk::ListBoxRow::new();
    row.add_css_class("clip-row");
    unsafe { row.set_data("line", line.to_owned()) };

    let image = content.starts_with("[[ binary data")
        && ["png", "jpg", "jpeg", "webp", "gif", "bmp"]
            .iter()
            .any(|k| content.contains(k));
    if image {
        unsafe { row.set_data("search", "imagem".to_owned()) };
        let picture = gtk::Picture::new();
        picture.set_content_fit(gtk::ContentFit::Contain);
        picture.set_size_request(-1, 120);
        picture.set_halign(gtk::Align::Start);
        picture.add_css_class("clip-image");
        row.set_child(Some(&picture));
        let thumbs = exec::cache_dir().join("clipboard");
        let path = thumbs.join(format!("{id}.img"));
        let line = line.to_owned();
        glib::spawn_future_local(async move {
            let file = path.clone();
            let ready = gio::spawn_blocking(move || {
                if !file.exists() {
                    let _ = std::fs::create_dir_all(&thumbs);
                    let bytes = decode(&line)?;
                    std::fs::write(&file, bytes).ok()?;
                }
                Some(())
            })
            .await
            .ok()
            .flatten();
            if ready.is_some() {
                picture.set_file(Some(&gio::File::for_path(&path)));
            }
        });
    } else {
        unsafe { row.set_data("search", content.to_lowercase()) };
        let label = gtk::Label::new(Some(content.trim()));
        label.set_xalign(0.0);
        label.set_lines(2);
        label.set_wrap(true);
        label.set_wrap_mode(pango::WrapMode::WordChar);
        label.set_ellipsize(pango::EllipsizeMode::End);
        row.set_child(Some(&label));
    }
    row
}
