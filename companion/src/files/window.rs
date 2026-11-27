//! A window: the toolbar (history, the path, search, views), the sidebar,
//! tabs, and the status line with the zoom.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use super::pane::{Hooks, Pane};
use super::sidebar::{Place, Sidebar};
use super::{display_name, ops, quicklook, state, tags};
use crate::i18n::{t, tf};
use crate::icons;

pub struct Window {
    pub window: gtk::ApplicationWindow,
    notebook: gtk::Notebook,
    panes: RefCell<Vec<Rc<Pane>>>,
    sidebar: RefCell<Option<Rc<Sidebar>>>,
    crumbs: gtk::Box,
    path_stack: gtk::Stack,
    path_entry: gtk::Entry,
    back: gtk::Button,
    forward: gtk::Button,
    search: gtk::SearchEntry,
    view_icons: gtk::ToggleButton,
    view_list: gtk::ToggleButton,
    status: gtk::Label,
    free: gtk::Label,
    zoom: gtk::Scale,
    search_gen: std::cell::Cell<u64>,
    git: RefCell<Option<Rc<super::git::Bar>>>,
}

thread_local! {
    /// The windows open, kept alive until they close.
    static OPEN: RefCell<Vec<Rc<Window>>> = RefCell::default();
}

use super::terminal;

impl Window {
    pub fn open(app: &gtk::Application, start: Option<gio::File>) -> Rc<Self> {
        let st = state::get();
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .title(t("arquivos"))
            .default_width(st.width)
            .default_height(st.height)
            .build();
        window.add_css_class("files-window");

        // the toolbar
        let header = gtk::HeaderBar::new();
        header.add_css_class("files-header");
        let back = icons::button("arrow_back", t("voltar"));
        let forward = icons::button("arrow_forward", t("avançar"));
        back.set_action_name(Some("win.back"));
        forward.set_action_name(Some("win.forward"));
        header.pack_start(&back);
        header.pack_start(&forward);

        let crumbs = gtk::Box::new(gtk::Orientation::Horizontal, 2);
        crumbs.add_css_class("files-crumbs");
        let path_entry = gtk::Entry::new();
        path_entry.add_css_class("files-path-entry");
        path_entry.set_hexpand(true);
        let path_stack = gtk::Stack::new();
        path_stack.set_transition_type(gtk::StackTransitionType::Crossfade);
        let crumbs_scroll = gtk::ScrolledWindow::builder().child(&crumbs).vscrollbar_policy(gtk::PolicyType::Never).hscrollbar_policy(gtk::PolicyType::External).build();
        path_stack.add_named(&crumbs_scroll, Some("crumbs"));
        path_stack.add_named(&path_entry, Some("entry"));
        path_stack.set_hexpand(true);
        path_stack.set_size_request(360, -1);
        header.set_title_widget(Some(&path_stack));

        let menu_button = gtk::MenuButton::new();
        menu_button.set_icon_name("open-menu-symbolic");
        menu_button.set_child(Some(&icons::icon("more_vert")));
        menu_button.add_css_class("files-tool");
        let app_menu = gio::Menu::new();
        app_menu.append(Some(t("nova janela")), Some("win.new-window"));
        app_menu.append(Some(t("clonar repositório…")), Some("win.clone"));
        app_menu.append(Some(t("conectar ao github")), Some("win.github"));
        app_menu.append(Some(t("nova aba")), Some("win.new-tab"));
        app_menu.append(Some(t("mostrar ocultos")), Some("win.hidden"));
        app_menu.append(Some(t("usar como gerenciador padrão")), Some("win.make-default"));
        menu_button.set_menu_model(Some(&app_menu));
        header.pack_end(&menu_button);

        let views = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        views.add_css_class("files-views");
        let view_icons = gtk::ToggleButton::new();
        view_icons.set_child(Some(&icons::icon("grid_view")));
        view_icons.set_tooltip_text(Some(t("ícones")));
        let view_list = gtk::ToggleButton::new();
        view_list.set_child(Some(&icons::icon("view_list")));
        view_list.set_tooltip_text(Some(t("lista")));
        view_list.set_group(Some(&view_icons));
        views.append(&view_icons);
        views.append(&view_list);
        header.pack_end(&views);

        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some(t("buscar")));
        search.add_css_class("files-search");
        header.pack_end(&search);
        window.set_titlebar(Some(&header));

        // the body
        let notebook = gtk::Notebook::new();
        notebook.add_css_class("files-tabs");
        notebook.set_scrollable(true);
        notebook.set_show_border(false);
        notebook.set_vexpand(true);

        let progress = gtk::ProgressBar::new();
        progress.add_css_class("files-progress");
        let progress_label = gtk::Label::new(None);
        progress_label.add_css_class("files-progress-label");
        progress_label.set_xalign(0.0);
        let progress_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        progress_box.append(&progress_label);
        progress_box.append(&progress);
        let progress_reveal = gtk::Revealer::builder().child(&progress_box).transition_type(gtk::RevealerTransitionType::SlideDown).build();
        progress_box.add_css_class("files-progress-box");

        let error = gtk::Label::new(None);
        error.add_css_class("files-error");
        error.set_wrap(true);
        let error_reveal = gtk::Revealer::builder().child(&error).transition_type(gtk::RevealerTransitionType::SlideUp).build();

        let status = gtk::Label::new(None);
        status.add_css_class("files-status");
        status.set_xalign(0.0);
        status.set_hexpand(true);
        let free = gtk::Label::new(None);
        free.add_css_class("files-status");
        let zoom = gtk::Scale::with_range(gtk::Orientation::Horizontal, 32.0, 192.0, 8.0);
        zoom.set_value(st.zoom as f64);
        zoom.set_size_request(120, -1);
        zoom.add_css_class("files-zoom");
        let foot = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        foot.add_css_class("files-foot");
        foot.append(&status);
        foot.append(&free);
        foot.append(&zoom);

        let fab = gtk::MenuButton::new();
        fab.add_css_class("files-fab");
        fab.set_child(Some(&icons::icon("add")));
        fab.set_tooltip_text(Some(t("novo")));
        fab.set_halign(gtk::Align::End);
        fab.set_valign(gtk::Align::End);
        fab.set_direction(gtk::ArrowType::Up);
        let fab_menu = gio::Menu::new();
        fab_menu.append(Some(t("nova pasta")), Some("win.new-folder"));
        fab_menu.append(Some(t("colar")), Some("win.paste"));
        fab_menu.append(Some(t("clonar repositório…")), Some("win.clone"));
        fab_menu.append(Some(t("conectar a servidor")), Some("win.connect"));
        fab.set_menu_model(Some(&fab_menu));
        let stage = gtk::Overlay::new();
        stage.set_child(Some(&notebook));
        stage.add_overlay(&fab);
        stage.set_vexpand(true);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
        content.add_css_class("files-content");
        content.append(&progress_reveal);
        content.append(&stage);
        content.append(&error_reveal);
        content.append(&foot);

        let paned = gtk::Paned::new(gtk::Orientation::Horizontal);
        paned.set_end_child(Some(&content));
        paned.set_position(st.sidebar);
        paned.set_shrink_start_child(false);
        paned.set_resize_start_child(false);
        window.set_child(Some(&paned));

        let this = Rc::new(Self {
            window: window.clone(),
            notebook,
            panes: RefCell::default(),
            sidebar: RefCell::default(),
            crumbs,
            path_stack,
            path_entry,
            back,
            forward,
            search,
            view_icons,
            view_list,
            status,
            free,
            zoom,
            search_gen: std::cell::Cell::new(0),
            git: RefCell::default(),
        });
        let git = super::git::Bar::new(this.complain(), this.opener());
        content.insert_child_after(&git.root, Some(&progress_reveal));
        *this.git.borrow_mut() = Some(git);

        let weak = Rc::downgrade(&this);
        let picked: Rc<dyn Fn(Place)> = Rc::new(move |place| {
            if let Some(w) = weak.upgrade() {
                w.place(place);
            }
        });
        let sidebar = Sidebar::new(picked, this.dropper());
        paned.set_start_child(Some(&sidebar.root));
        *this.sidebar.borrow_mut() = Some(sidebar);

        // progress and errors from operations
        let ops = ops::get();
        {
            let (r, bar, label) = (progress_reveal.clone(), progress.clone(), progress_label.clone());
            ops.progress.subscribe(move |p| {
                r.set_reveal_child(p.is_some());
                if let Some((text, fraction)) = p {
                    label.set_label(text);
                    bar.set_fraction(*fraction);
                }
            });
            let (r, label) = (error_reveal.clone(), error.clone());
            ops.error.subscribe(move |e| {
                if let Some(text) = e {
                    label.set_label(text);
                    r.set_reveal_child(true);
                    let r = r.clone();
                    glib::timeout_add_local_once(std::time::Duration::from_secs(5), move || r.set_reveal_child(false));
                }
            });
        }

        this.wire(&paned);
        this.actions();
        let home = gio::File::for_path(glib::home_dir());
        this.add_tab(&start.unwrap_or(home));
        OPEN.with(|o| o.borrow_mut().push(this.clone()));
        let weak = Rc::downgrade(&this);
        window.connect_destroy(move |_| {
            if let Some(w) = weak.upgrade() {
                OPEN.with(|o| o.borrow_mut().retain(|x| !Rc::ptr_eq(x, &w)));
            }
        });
        window.present();
        this.current().focus();
        this
    }

    fn complain(self: &Rc<Self>) -> Rc<dyn Fn(String)> {
        Rc::new(|e: String| ops::get().error.set(Some(e)))
    }

    /// Opens a folder here, or a file in its app.
    fn opener(self: &Rc<Self>) -> Rc<dyn Fn(gio::File)> {
        let weak = Rc::downgrade(self);
        Rc::new(move |file: gio::File| {
            let Some(w) = weak.upgrade() else { return };
            let is_dir = file.query_file_type(gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE) == gio::FileType::Directory;
            if is_dir {
                w.current().go(&file);
                w.window.present();
            } else if let Some(parent) = file.parent() {
                let _ = gio::AppInfo::launch_default_for_uri(&file.uri(), gdk::Display::default().map(|d| d.app_launch_context()).as_ref());
                let _ = parent;
            }
        })
    }

    /// Selects a file once its folder has loaded.
    pub fn select_soon(self: &Rc<Self>, file: gio::File) {
        let pane = self.current();
        glib::timeout_add_local_once(std::time::Duration::from_millis(400), move || pane.select_file(&file));
    }

    fn dropper(self: &Rc<Self>) -> super::cells::OnDrop {
        Rc::new(|files: Vec<gio::File>, into: gio::File, copy: bool| {
            // another disk copies, like Finder; ctrl copies anywhere
            let same_fs = files.first().is_some_and(|f| {
                let fs = |x: &gio::File| x.query_info("id::filesystem", gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE).ok().and_then(|i| i.attribute_string("id::filesystem"));
                fs(f) == fs(&into)
            });
            let moving = !copy && same_fs;
            glib::spawn_future_local(ops::get().transfer(files, into, moving));
        })
    }

    fn current(&self) -> Rc<Pane> {
        let index = self.notebook.current_page().unwrap_or(0) as usize;
        let panes = self.panes.borrow();
        panes.get(index).or(panes.first()).cloned().expect("a window always has a tab")
    }

    fn add_tab(self: &Rc<Self>, start: &gio::File) {
        let weak = Rc::downgrade(self);
        let moved = {
            let weak = weak.clone();
            move || {
                if let Some(w) = weak.upgrade() {
                    w.refresh();
                }
            }
        };
        let changed = {
            let weak = weak.clone();
            move || {
                if let Some(w) = weak.upgrade() {
                    w.update_status();
                }
            }
        };
        let open = {
            let weak = weak.clone();
            move |file: gio::File, info: gio::FileInfo| {
                if let Some(w) = weak.upgrade() {
                    w.open_file(&file, &info);
                }
            }
        };
        let menu = {
            let weak = weak.clone();
            move |widget: &gtk::Widget, x: f64, y: f64, on_item: bool| {
                if let Some(w) = weak.upgrade() {
                    w.popup(widget, x, y, on_item);
                }
            }
        };
        let pane = Pane::new(
            start,
            Hooks { moved: Box::new(moved), changed: Box::new(changed), open: Box::new(open), menu: Box::new(menu), dropped: self.dropper() },
        );
        let tab = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        let label = gtk::Label::new(Some(&pane.title()));
        label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        label.set_max_width_chars(18);
        let close = icons::button("close", t("fechar aba"));
        close.add_css_class("files-tab-close");
        tab.append(&label);
        tab.append(&close);
        let page = self.notebook.append_page(&pane.root, Some(&tab));
        self.notebook.set_tab_reorderable(&pane.root, true);
        self.panes.borrow_mut().push(pane.clone());
        let (weak, root) = (Rc::downgrade(self), pane.root.clone());
        close.connect_clicked(move |_| {
            if let Some(w) = weak.upgrade() {
                if let Some(i) = w.notebook.page_num(&root) {
                    w.close_tab(i as usize);
                }
            }
        });
        self.notebook.set_current_page(Some(page));
        self.notebook.set_show_tabs(self.notebook.n_pages() > 1);
        self.refresh();
    }

    fn close_tab(&self, index: usize) {
        if self.panes.borrow().len() <= 1 {
            self.window.close();
            return;
        }
        let pane = self.panes.borrow_mut().remove(index);
        self.notebook.remove_page(Some(index as u32));
        drop(pane);
        self.notebook.set_show_tabs(self.notebook.n_pages() > 1);
        self.refresh();
    }

    /// Everything that follows the folder shown.
    fn refresh(&self) {
        let pane = self.current();
        let location = pane.location();
        self.window.set_title(Some(&pane.title()));
        self.back.set_sensitive(pane.can_back());
        self.forward.set_sensitive(pane.can_forward());
        if let Some(tab) = self.notebook.tab_label(&pane.root).and_downcast::<gtk::Box>() {
            if let Some(label) = tab.first_child().and_downcast::<gtk::Label>() {
                label.set_label(&pane.title());
            }
        }
        if let Some(s) = self.sidebar.borrow().as_ref() {
            s.highlight(&location);
        }
        if let Some(git) = self.git.borrow().as_ref() {
            if pane.showing_results() {
                git.root.set_reveal_child(false);
            } else {
                git.update(&location);
            }
        }
        self.view_icons.set_active(pane.view() == "icons");
        self.view_list.set_active(pane.view() == "list");
        self.zoom.set_visible(pane.view() == "icons");
        self.build_crumbs(&pane);
        self.path_entry.set_text(&location.parse_name());
        self.update_status();
        let free = self.free.clone();
        free.set_label("");
        location.query_filesystem_info_async("filesystem::free", glib::Priority::LOW, gio::Cancellable::NONE, move |r| {
            if let Ok(info) = r {
                let bytes = info.attribute_uint64("filesystem::free");
                if bytes > 0 {
                    free.set_label(&tf("{} livres", glib::format_size(bytes)));
                }
            }
        });
    }

    fn build_crumbs(self: &Self, pane: &Rc<Pane>) {
        crate::widgets::clear(&self.crumbs);
        if pane.showing_results() {
            let label = gtk::Label::new(Some(&pane.title()));
            label.add_css_class("files-crumb-title");
            self.crumbs.append(&label);
            return;
        }
        let mut chain = vec![pane.location()];
        while let Some(parent) = chain.last().and_then(|f| f.parent()) {
            let home = gio::File::for_path(glib::home_dir());
            if chain.last().is_some_and(|f| f.equal(&home)) {
                break;
            }
            chain.push(parent);
        }
        chain.reverse();
        let count = chain.len();
        for (i, file) in chain.into_iter().enumerate() {
            if i > 0 {
                let sep = icons::icon("chevron_right");
                sep.add_css_class("files-crumb-sep");
                self.crumbs.append(&sep);
            }
            let button = gtk::Button::new();
            button.add_css_class("files-crumb");
            let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            if i == 0 {
                content.append(&icons::icon(super::symbol_of(&file)));
            }
            content.append(&gtk::Label::new(Some(&display_name(&file))));
            button.set_child(Some(&content));
            if i + 1 == count {
                button.add_css_class("current");
            }
            let p = pane.clone();
            button.connect_clicked(move |_| p.go(&file));
            self.crumbs.append(&button);
        }
    }

    fn update_status(&self) {
        let pane = self.current();
        let selected = pane.selected_infos();
        let text = if selected.is_empty() {
            tf("{} itens", pane.count())
        } else {
            let bytes: i64 = selected.iter().filter(|i| i.file_type() != gio::FileType::Directory).map(|i| i.size()).sum();
            let mut s = format!("{} {}", tf("{} de", selected.len()), tf("{} selecionados", pane.count()));
            if bytes > 0 {
                s.push_str(&format!(" · {}", glib::format_size(bytes as u64)));
            }
            s
        };
        self.status.set_label(&text);
    }

    fn open_file(self: &Rc<Self>, file: &gio::File, info: &gio::FileInfo) {
        let target = info.attribute_string("standard::target-uri").map(|u| gio::File::for_uri(&u));
        if info.file_type() == gio::FileType::Directory || info.file_type() == gio::FileType::Mountable || target.is_some() {
            self.current().go(target.as_ref().unwrap_or(file));
            return;
        }
        let context = gdk::Display::default().map(|d| d.app_launch_context());
        if let Err(err) = gio::AppInfo::launch_default_for_uri(&file.uri(), context.as_ref()) {
            ops::get().error.set(Some(format!("{}: {err}", display_name(file))));
        }
    }

    fn place(self: &Rc<Self>, place: Place) {
        match place {
            Place::Folder(f) => self.current().go(&f),
            Place::Tag(tag) => {
                let name = super::tags::COLORS.iter().find(|c| c.0 == tag).map(|c| t(c.1)).unwrap_or(tag);
                self.current().show_results(&format!("{} {name}", t("tag")), tags::tagged(tag));
            }
            Place::Network => super::network::dialog(self.window.upcast_ref(), self.opener(), self.complain()),
            Place::Drive => super::drive::open(self.window.upcast_ref(), self.opener(), self.complain()),
            Place::Connect => self.connect_server(),
        }
    }

    /// sftp://, ftp://… mounted through gvfs, then opened.
    fn connect_server(self: &Rc<Self>) {
        let dialog = gtk::Window::builder().transient_for(&self.window).modal(true).title(t("conectar a servidor")).default_width(420).build();
        dialog.add_css_class("files-dialog");
        let column = gtk::Box::new(gtk::Orientation::Vertical, 12);
        column.add_css_class("files-dialog-root");
        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some("sftp://user@host/path"));
        let hint = gtk::Label::new(Some(t("sftp e ftp funcionam; smb precisa do gvfs-smb")));
        hint.add_css_class("dim");
        hint.set_xalign(0.0);
        let go = gtk::Button::with_label(t("conectar"));
        go.add_css_class("suggested");
        column.append(&entry);
        column.append(&hint);
        column.append(&go);
        dialog.set_child(Some(&column));
        let (weak, d) = (Rc::downgrade(self), dialog.clone());
        let connect = Rc::new(move |text: String| {
            let file = gio::File::for_uri(text.trim());
            let op = gtk::MountOperation::new(Some(&d));
            let (weak, d2, f2) = (weak.clone(), d.clone(), file.clone());
            file.mount_enclosing_volume(gio::MountMountFlags::NONE, Some(&op), gio::Cancellable::NONE, move |r| {
                let already = matches!(&r, Err(e) if e.matches(gio::IOErrorEnum::AlreadyMounted));
                if r.is_ok() || already {
                    if let Some(w) = weak.upgrade() {
                        w.current().go(&f2);
                    }
                    d2.close();
                } else if let Err(e) = r {
                    ops::get().error.set(Some(e.to_string()));
                }
            });
        });
        let (c, e) = (connect.clone(), entry.clone());
        go.connect_clicked(move |_| c(e.text().to_string()));
        entry.connect_activate(move |e| connect(e.text().to_string()));
        dialog.present();
    }

    fn popup(self: &Rc<Self>, widget: &gtk::Widget, x: f64, y: f64, on_item: bool) {
        let pane = self.current();
        let in_trash = pane.location().uri_scheme().as_deref() == Some("trash");
        let model = if on_item { super::menu::for_selection(&pane.selected(), &pane.selected_infos(), in_trash) } else { super::menu::for_folder(in_trash) };
        let popover = gtk::PopoverMenu::from_model(Some(&model));
        popover.set_parent(widget);
        popover.set_has_arrow(false);
        popover.set_halign(gtk::Align::Start);
        popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
        popover.connect_closed(|p| {
            let p = p.clone();
            glib::idle_add_local_once(move || p.unparent());
        });
        popover.popup();
    }

    fn rename(self: &Rc<Self>) {
        let Some(file) = self.current().selected().into_iter().next() else { return };
        let name = display_name(&file);
        let dialog = gtk::Window::builder().transient_for(&self.window).modal(true).title(t("renomear")).default_width(380).build();
        dialog.add_css_class("files-dialog");
        let entry = gtk::Entry::new();
        entry.set_text(&name);
        entry.add_css_class("files-dialog-root");
        dialog.set_child(Some(&entry));
        let stem = name.rfind('.').filter(|i| *i > 0).unwrap_or(name.chars().count());
        entry.select_region(0, stem as i32);
        let (d, pane) = (dialog.clone(), self.current());
        entry.connect_activate(move |e| {
            let new = e.text().to_string();
            let (file, pane, d) = (file.clone(), pane.clone(), d.clone());
            glib::spawn_future_local(async move {
                if !new.is_empty() && new != display_name(&file) {
                    if let Some(renamed) = ops::get().rename(file, new).await {
                        glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || pane.select_file(&renamed));
                    }
                }
                d.close();
            });
        });
        let keys = gtk::EventControllerKey::new();
        let d = dialog.clone();
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                d.close();
                return glib::Propagation::Stop;
            }
            glib::Propagation::Proceed
        });
        dialog.add_controller(keys);
        dialog.present();
    }

    /// Finds by name under the folder, a few thousand at most, off the
    /// main thread; results stream in.
    fn deep_search(self: &Rc<Self>, text: &str) {
        let pane = self.current();
        let base = if pane.showing_results() { gio::File::for_path(glib::home_dir()) } else { pane.location() };
        let Some(root) = base.path() else { return };
        let needle = text.to_lowercase();
        let generation = self.search_gen.get() + 1;
        self.search_gen.set(generation);
        pane.show_results(&tf("“{}”", text), Vec::new());
        let (tx, rx) = async_channel::unbounded::<Vec<std::path::PathBuf>>();
        let fd = glib::find_program_in_path("fd");
        std::thread::spawn(move || {
            if let Some(fd) = fd {
                use std::io::BufRead;
                let child = std::process::Command::new(fd)
                    .args(["-i", "--absolute-path", "--max-results", "3000", "--exclude", ".git", "--exclude", "node_modules", "--exclude", "target", "--fixed-strings", "--", &needle])
                    .arg(&root)
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::null())
                    .spawn();
                if let Ok(mut child) = child {
                    let mut batch = Vec::new();
                    if let Some(out) = child.stdout.take() {
                        for line in std::io::BufReader::new(out).lines().map_while(Result::ok) {
                            batch.push(std::path::PathBuf::from(line));
                            if batch.len() >= 60 && tx.send_blocking(std::mem::take(&mut batch)).is_err() {
                                let _ = child.kill();
                                return;
                            }
                        }
                    }
                    let _ = tx.send_blocking(batch);
                    let _ = child.wait();
                    return;
                }
            }
            let mut found = 0;
            let mut batch = Vec::new();
            let mut stack = vec![root];
            while let Some(dir) = stack.pop() {
                let Ok(rd) = std::fs::read_dir(&dir) else { continue };
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().to_lowercase();
                    if name.starts_with('.') {
                        continue;
                    }
                    let path = e.path();
                    if e.file_type().is_ok_and(|t| t.is_dir()) {
                        stack.push(path.clone());
                    }
                    if name.contains(&needle) {
                        batch.push(path);
                        found += 1;
                        if batch.len() >= 40 && tx.send_blocking(std::mem::take(&mut batch)).is_err() {
                            return;
                        }
                    }
                    if found >= 3000 {
                        let _ = tx.send_blocking(batch);
                        return;
                    }
                }
            }
            let _ = tx.send_blocking(batch);
        });
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            while let Ok(batch) = rx.recv().await {
                let Some(w) = weak.upgrade() else { return };
                if w.search_gen.get() != generation || !pane.showing_results() {
                    return;
                }
                pane.append_results(batch.into_iter().map(gio::File::for_path).collect());
            }
        });
    }

    fn wire(self: &Rc<Self>, paned: &gtk::Paned) {
        let weak = Rc::downgrade(self);
        self.notebook.connect_switch_page(move |_, _, _| {
            if let Some(w) = weak.upgrade() {
                glib::idle_add_local_once(move || w.refresh());
            }
        });
        for (button, view) in [(self.view_icons.clone(), "icons"), (self.view_list.clone(), "list")] {
            let weak = Rc::downgrade(self);
            button.connect_toggled(move |b| {
                if b.is_active() {
                    if let Some(w) = weak.upgrade() {
                        w.current().set_view(view);
                        w.zoom.set_visible(view == "icons");
                        state::edit(|s| s.view = view.into());
                    }
                }
            });
        }
        let weak = Rc::downgrade(self);
        self.zoom.connect_value_changed(move |z| {
            if let Some(w) = weak.upgrade() {
                let v = z.value() as i32;
                w.current().set_zoom(v);
                state::edit(|s| s.zoom = v);
            }
        });
        let weak = Rc::downgrade(self);
        self.search.connect_search_changed(move |s| {
            if let Some(w) = weak.upgrade() {
                let pane = w.current();
                if !pane.showing_results() {
                    pane.set_query(&s.text());
                }
            }
        });
        let weak = Rc::downgrade(self);
        self.search.connect_activate(move |s| {
            if let Some(w) = weak.upgrade() {
                let text = s.text().to_string();
                if !text.trim().is_empty() {
                    w.deep_search(text.trim());
                }
            }
        });
        let weak = Rc::downgrade(self);
        self.search.connect_stop_search(move |s| {
            s.set_text("");
            if let Some(w) = weak.upgrade() {
                let pane = w.current();
                if pane.showing_results() {
                    pane.back();
                }
                pane.focus();
            }
        });
        let weak = Rc::downgrade(self);
        self.path_entry.connect_activate(move |e| {
            let Some(w) = weak.upgrade() else { return };
            let text = e.text().to_string();
            let expanded = if let Some(rest) = text.strip_prefix('~') { format!("{}{rest}", glib::home_dir().display()) } else { text };
            let file = gio::File::for_parse_name(&expanded);
            w.path_stack.set_visible_child_name("crumbs");
            w.current().go(&file);
            w.current().focus();
        });
        let keys = gtk::EventControllerKey::new();
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, _| {
            if key == gdk::Key::Escape {
                if let Some(w) = weak.upgrade() {
                    w.path_stack.set_visible_child_name("crumbs");
                    w.current().focus();
                }
            }
            glib::Propagation::Proceed
        });
        self.path_entry.add_controller(keys);

        // space: quick look, unless typing
        let keys = gtk::EventControllerKey::new();
        keys.set_propagation_phase(gtk::PropagationPhase::Capture);
        let weak = Rc::downgrade(self);
        keys.connect_key_pressed(move |_, key, _, state| {
            let Some(w) = weak.upgrade() else { return glib::Propagation::Proceed };
            let typing = gtk::prelude::GtkWindowExt::focus(&w.window).is_some_and(|f| f.is::<gtk::Text>() || f.is::<gtk::Entry>() || f.ancestor(gtk::Entry::static_type()).is_some());
            if matches!(key, gdk::Key::space | gdk::Key::Escape) && quicklook::close() {
                return glib::Propagation::Stop;
            }
            if key == gdk::Key::space && !typing && state.is_empty() {
                w.quicklook();
                return glib::Propagation::Stop;
            }
            // typing a letter in the folder jumps to the search
            if !typing && !state.intersects(gdk::ModifierType::CONTROL_MASK | gdk::ModifierType::ALT_MASK) {
                if let Some(c) = key.to_unicode().filter(|c| c.is_alphanumeric()) {
                    w.search.grab_focus();
                    w.search.set_text(&c.to_string());
                    w.search.set_position(-1);
                    return glib::Propagation::Stop;
                }
            }
            glib::Propagation::Proceed
        });
        self.window.add_controller(keys);

        let p = paned.clone();
        self.window.connect_close_request(move |w| {
            let (width, height) = (w.width(), w.height());
            let side = p.position();
            state::edit(|s| {
                s.width = width;
                s.height = height;
                s.sidebar = side;
            });
            glib::Propagation::Proceed
        });
    }

    fn quicklook(&self) {
        let pane = self.current();
        let selected = pane.selected();
        let shown = pane.shown();
        let start = selected.first().and_then(|s| shown.iter().position(|(f, _)| f.equal(s))).unwrap_or(0);
        let items = if selected.len() > 1 { shown.into_iter().filter(|(f, _)| selected.iter().any(|s| s.equal(f))).collect() } else { shown };
        let start = if selected.len() > 1 { 0 } else { start };
        quicklook::show(self.window.upcast_ref(), items, start);
    }

    fn actions(self: &Rc<Self>) {
        let simple = |name: &str, f: Box<dyn Fn(&Rc<Self>)>| {
            let action = gio::SimpleAction::new(name, None);
            let weak = Rc::downgrade(self);
            action.connect_activate(move |_, _| {
                if let Some(w) = weak.upgrade() {
                    f(&w);
                }
            });
            self.window.add_action(&action);
        };
        let with_string = |name: &str, f: Box<dyn Fn(&Rc<Self>, String)>| {
            let action = gio::SimpleAction::new(name, Some(glib::VariantTy::STRING));
            let weak = Rc::downgrade(self);
            action.connect_activate(move |_, v| {
                if let (Some(w), Some(s)) = (weak.upgrade(), v.and_then(|v| v.get::<String>())) {
                    f(&w, s);
                }
            });
            self.window.add_action(&action);
        };
        let run = |f: std::pin::Pin<Box<dyn std::future::Future<Output = ()>>>| {
            glib::spawn_future_local(f);
        };

        simple("back", Box::new(|w| w.current().back()));
        simple("forward", Box::new(|w| w.current().forward()));
        simple("up", Box::new(|w| w.current().up()));
        simple("reload", Box::new(|w| {
            let p = w.current();
            let here = p.location();
            p.go(&gio::File::for_path("/"));
            p.back();
            p.go(&here);
            p.back();
        }));
        simple("location", Box::new(|w| {
            w.path_stack.set_visible_child_name("entry");
            w.path_entry.grab_focus();
            w.path_entry.select_region(0, -1);
        }));
        simple("search", Box::new(|w| {
            w.search.grab_focus();
        }));
        simple("open", Box::new(|w| {
            let p = w.current();
            for (file, info) in p.selected().into_iter().zip(p.selected_infos()) {
                w.open_file(&file, &info);
            }
        }));
        with_string("open-with", Box::new(|w, id| {
            let Some(app) = gio_unix::DesktopAppInfo::new(&id) else { return };
            let files = w.current().selected();
            let context = gdk::Display::default().map(|d| d.app_launch_context());
            if let Err(e) = app.launch(&files, context.as_ref()) {
                ops::get().error.set(Some(e.to_string()));
            }
        }));
        simple("open-tab", Box::new(|w| {
            for f in w.current().selected() {
                w.add_tab(&f);
            }
        }));
        simple("quicklook", Box::new(|w| w.quicklook()));
        simple("info", Box::new(|w| {
            let p = w.current();
            let files = p.selected();
            if files.is_empty() {
                super::info::show(w.window.upcast_ref(), &p.location());
            }
            for f in files.iter().take(4) {
                super::info::show(w.window.upcast_ref(), f);
            }
        }));
        simple("rename", Box::new(|w| w.rename()));
        simple("trash", Box::new(move |w| {
            let files = w.current().selected();
            if !files.is_empty() {
                run(Box::pin(ops::get().trash(files)));
            }
        }));
        simple("delete", Box::new(move |w| {
            let files = w.current().selected();
            if files.is_empty() {
                return;
            }
            let dialog = gtk::AlertDialog::builder()
                .message(tf("apagar {} itens pra sempre?", files.len()))
                .detail(t("não dá pra desfazer"))
                .buttons([t("cancelar"), t("apagar")])
                .cancel_button(0)
                .default_button(0)
                .build();
            dialog.choose(Some(&w.window), gio::Cancellable::NONE, move |r| {
                if r == Ok(1) {
                    glib::spawn_future_local(ops::get().remove(files));
                }
            });
        }));
        simple("restore", Box::new(|w| {
            for file in w.current().selected() {
                if let Ok(info) = file.query_info("trash::orig-path", gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE) {
                    if let Some(orig) = info.attribute_byte_string("trash::orig-path") {
                        let back = gio::File::for_path(orig.as_str());
                        if let Err(e) = file.move_(&back, gio::FileCopyFlags::NONE, gio::Cancellable::NONE, None) {
                            ops::get().error.set(Some(e.to_string()));
                        }
                    }
                }
            }
        }));
        simple("empty-trash", Box::new(move |_| run(Box::pin(ops::get().empty_trash()))));
        simple("copy", Box::new(|w| ops::get().to_clipboard(&w.window, &w.current().selected(), false)));
        simple("cut", Box::new(|w| ops::get().to_clipboard(&w.window, &w.current().selected(), true)));
        simple("paste", Box::new(move |w| {
            let p = w.current();
            if !p.showing_results() {
                run(Box::pin(ops::get().paste(w.window.clone().upcast(), p.location())));
            }
        }));
        simple("duplicate", Box::new(move |w| {
            let p = w.current();
            let files = p.selected();
            if !files.is_empty() && !p.showing_results() {
                run(Box::pin(ops::get().transfer(files, p.location(), false)));
            }
        }));
        simple("copy-path", Box::new(|w| {
            let paths: Vec<String> = w.current().selected().iter().map(|f| f.parse_name().to_string()).collect();
            w.window.clipboard().set_text(&paths.join("\n"));
        }));
        simple("select-all", Box::new(|w| w.current().select_all()));
        simple("new-folder", Box::new(|w| {
            let p = w.current();
            if p.showing_results() {
                return;
            }
            let w = w.clone();
            glib::spawn_future_local(async move {
                if let Some(folder) = ops::get().new_folder(p.location()).await {
                    glib::timeout_add_local_once(std::time::Duration::from_millis(250), move || {
                        p.selection.unselect_all();
                        p.select_file(&folder);
                        w.rename();
                    });
                }
            });
        }));
        simple("compress", Box::new(move |w| {
            let files = w.current().selected();
            if !files.is_empty() {
                run(Box::pin(ops::get().compress(files)));
            }
        }));
        simple("extract", Box::new(move |w| {
            if let Some(f) = w.current().selected().into_iter().next() {
                run(Box::pin(ops::get().extract(f)));
            }
        }));
        simple("undo", Box::new(move |_| run(Box::pin(ops::get().undo()))));
        simple("new-tab", Box::new(|w| {
            let here = w.current().location();
            w.add_tab(&here);
        }));
        simple("close-tab", Box::new(|w| {
            let i = w.notebook.current_page().unwrap_or(0) as usize;
            w.close_tab(i);
        }));
        simple("new-window", Box::new(|w| {
            if let Some(app) = w.window.application() {
                Window::open(&app, Some(w.current().location()));
            }
        }));
        simple("zoom-in", Box::new(|w| w.zoom.set_value(w.zoom.value() + 16.0)));
        simple("zoom-out", Box::new(|w| w.zoom.set_value(w.zoom.value() - 16.0)));
        with_string("view", Box::new(|w, view| {
            if view == "list" {
                w.view_list.set_active(true);
            } else {
                w.view_icons.set_active(true);
            }
        }));
        with_string("sort", Box::new(|w, key| {
            let p = w.current();
            let (_, descending) = p.sort();
            p.set_sort(&key, descending);
            state::edit(|s| s.sort = key);
        }));
        with_string("tag", Box::new(|w, tag| {
            for f in w.current().selected() {
                tags::toggle(&f, &tag);
            }
            // the listing doesn't watch extended attributes: show the change
            let p = w.current();
            if !p.showing_results() {
                let here = p.location();
                p.set_query("\u{0}");
                p.set_query("");
                let _ = here;
            }
        }));

        let hidden = gio::SimpleAction::new_stateful("hidden", None, &state::get().hidden.to_variant());
        let weak = Rc::downgrade(self);
        hidden.connect_activate(move |a, _| {
            let on = !a.state().and_then(|s| s.get::<bool>()).unwrap_or(false);
            a.set_state(&on.to_variant());
            if let Some(w) = weak.upgrade() {
                for p in w.panes.borrow().iter() {
                    p.set_hidden(on);
                }
            }
            state::edit(|s| s.hidden = on);
        });
        self.window.add_action(&hidden);
        let descending = gio::SimpleAction::new_stateful("descending", None, &state::get().descending.to_variant());
        let weak = Rc::downgrade(self);
        descending.connect_activate(move |a, _| {
            let on = !a.state().and_then(|s| s.get::<bool>()).unwrap_or(false);
            a.set_state(&on.to_variant());
            if let Some(w) = weak.upgrade() {
                let p = w.current();
                let (key, _) = p.sort();
                p.set_sort(&key, on);
            }
            state::edit(|s| s.descending = on);
        });
        self.window.add_action(&descending);

        simple("bookmark", Box::new(|w| {
            let p = w.current();
            let target = p.selected().into_iter().next().unwrap_or_else(|| p.location());
            super::sidebar::toggle_bookmark(&target);
        }));
        simple("terminal", Box::new(|w| {
            let p = w.current();
            let folder = p.selected_infos().first().filter(|i| i.file_type() == gio::FileType::Directory).and_then(|_| p.selected().into_iter().next()).unwrap_or_else(|| p.location());
            if let (Some(term), Some(path)) = (terminal(), folder.path()) {
                let _ = std::process::Command::new(term).current_dir(path).spawn();
            }
        }));
        simple("share", Box::new(|w| super::share::dialog(w.window.upcast_ref(), w.current().selected())));
        simple("clone", Box::new(|w| {
            let p = w.current();
            let into = if p.showing_results() { gio::File::for_path(glib::home_dir()) } else { p.location() };
            super::git::clone_dialog(w.window.upcast_ref(), into, w.complain(), w.opener());
        }));
        simple("github", Box::new(|_| super::git::connect_github()));
        simple("connect", Box::new(|w| w.connect_server()));
        simple("make-default", Box::new(|_| {
            let ok = std::process::Command::new("xdg-mime").args(["default", "octopus-files.desktop", "inode/directory"]).status().is_ok_and(|s| s.success());
            if !ok {
                ops::get().error.set(Some(t("não deu pra definir como padrão").to_owned()));
            }
        }));
    }
}
