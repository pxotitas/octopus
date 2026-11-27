//! Git, the basics: a bar over any folder inside a repository (the branch,
//! what changed, commit, pull, push), cloning by url or from your GitHub
//! repositories, and connecting GitHub through `gh`.

use std::path::{Path, PathBuf};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};

use crate::i18n::{t, tf};
use crate::icons;

/// Runs git (or gh) without ever stopping to ask for a password.
async fn run(dir: PathBuf, program: &'static str, args: Vec<String>) -> Result<String, String> {
    gio::spawn_blocking(move || {
        let out = std::process::Command::new(program)
            .args(&args)
            .current_dir(&dir)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| e.to_string())?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            let err = String::from_utf8_lossy(&out.stderr).trim().to_owned();
            Err(if err.is_empty() { String::from_utf8_lossy(&out.stdout).trim().to_owned() } else { err })
        }
    })
    .await
    .unwrap_or_else(|_| Err(String::new()))
}

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

#[derive(Default, Clone)]
struct Status {
    branch: String,
    upstream: bool,
    ahead: u32,
    behind: u32,
    /// (two-letter code, path)
    changes: Vec<(String, String)>,
}

fn parse_status(text: &str) -> Status {
    let mut s = Status::default();
    for line in text.lines() {
        if let Some(head) = line.strip_prefix("## ") {
            let (names, counts) = head.split_once(" [").unwrap_or((head, ""));
            let (branch, upstream) = names.split_once("...").unwrap_or((names, ""));
            s.branch = branch.trim_start_matches("No commits yet on ").to_owned();
            s.upstream = !upstream.is_empty();
            for part in counts.trim_end_matches(']').split(", ") {
                if let Some(n) = part.strip_prefix("ahead ") {
                    s.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = part.strip_prefix("behind ") {
                    s.behind = n.parse().unwrap_or(0);
                }
            }
        } else if line.len() > 3 {
            s.changes.push((line[..2].to_owned(), line[3..].trim_matches('"').to_owned()));
        }
    }
    s
}

fn describe(code: &str) -> (&'static str, &'static str) {
    match code.trim() {
        "??" => ("add_circle", "novo"),
        c if c.contains('D') => ("remove_circle", "apagado"),
        c if c.contains('A') => ("add_circle", "adicionado"),
        c if c.contains('R') => ("drive_file_move", "renomeado"),
        _ => ("edit", "modificado"),
    }
}

/// The bar shown over folders inside a repository.
pub struct Bar {
    pub root: gtk::Revealer,
    root_dir: std::cell::RefCell<Option<PathBuf>>,
    branch: gtk::MenuButton,
    summary: gtk::Label,
    changes: gtk::MenuButton,
    error: Rc<dyn Fn(String)>,
    open: Rc<dyn Fn(gio::File)>,
    generation: std::cell::Cell<u64>,
}

impl Bar {
    pub fn new(error: Rc<dyn Fn(String)>, open: Rc<dyn Fn(gio::File)>) -> Rc<Self> {
        let card = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        card.add_css_class("git-bar");
        let badge = crate::widgets::shape::Shape::new(crate::widgets::shape::Kind::Cookie6, 36, true, Some(&icons::icon("commit")));
        badge.add_css_class("git-badge");
        card.append(&badge);
        let branch = gtk::MenuButton::new();
        branch.add_css_class("git-branch");
        card.append(&branch);
        let summary = gtk::Label::new(None);
        summary.add_css_class("git-summary");
        summary.set_hexpand(true);
        summary.set_xalign(0.0);
        card.append(&summary);
        let changes = gtk::MenuButton::new();
        changes.add_css_class("git-tool");
        changes.set_child(Some(&icons::icon("difference")));
        changes.set_tooltip_text(Some(t("alterações")));
        card.append(&changes);
        let root = gtk::Revealer::builder().child(&card).transition_type(gtk::RevealerTransitionType::SlideDown).build();
        let bar = Rc::new(Self {
            root,
            root_dir: Default::default(),
            branch,
            summary,
            changes,
            error,
            open,
            generation: std::cell::Cell::new(0),
        });
        for (icon, tip, action) in [("arrow_downward", "puxar", 0), ("arrow_upward", "enviar", 1), ("check", "commit…", 2)] {
            let button = icons::button(icon, t(tip));
            button.add_css_class(if action == 2 { "git-commit" } else { "git-tool" });
            if action == 2 {
                let content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                content.append(&icons::icon(icon));
                content.append(&gtk::Label::new(Some(t("commit"))));
                button.set_child(Some(&content));
            }
            let weak = Rc::downgrade(&bar);
            button.connect_clicked(move |b| {
                let Some(bar) = weak.upgrade() else { return };
                match action {
                    0 => bar.sync(&["pull", "--ff-only"]),
                    1 => bar.push(),
                    _ => bar.commit_dialog(b),
                }
            });
            card.append(&button);
        }
        bar
    }

    /// Looks for a repository around the folder; shows the bar if there is one.
    pub fn update(self: &Rc<Self>, location: &gio::File) {
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let Some(dir) = location.path() else {
            self.root.set_reveal_child(false);
            return;
        };
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let top = run(dir, "git", args(&["rev-parse", "--show-toplevel"])).await.ok().map(|s| PathBuf::from(s.trim()));
            let Some(bar) = weak.upgrade() else { return };
            if bar.generation.get() != generation {
                return;
            }
            *bar.root_dir.borrow_mut() = top.clone();
            match top {
                Some(_) => bar.refresh(),
                None => bar.root.set_reveal_child(false),
            }
        });
    }

    fn dir(&self) -> Option<PathBuf> {
        self.root_dir.borrow().clone()
    }

    pub fn refresh(self: &Rc<Self>) {
        let Some(dir) = self.dir() else { return };
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Ok(text) = run(dir.clone(), "git", args(&["status", "--porcelain=v1", "-b", "-uall"])).await else { return };
            let branches = run(dir.clone(), "git", args(&["branch", "--format=%(refname:short)"])).await.unwrap_or_default();
            let Some(bar) = weak.upgrade() else { return };
            let status = parse_status(&text);
            bar.show(&dir, &status, branches.lines().map(str::to_owned).collect());
        });
    }

    fn show(self: &Rc<Self>, dir: &Path, status: &Status, branches: Vec<String>) {
        let repo = dir.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let label = gtk::Box::new(gtk::Orientation::Horizontal, 6);
        label.append(&icons::icon("call_split"));
        label.append(&gtk::Label::new(Some(&status.branch)));
        self.branch.set_child(Some(&label));
        self.branch.set_tooltip_text(Some(&repo));

        let mut parts = vec![repo];
        parts.push(if status.changes.is_empty() { t("tudo salvo").to_owned() } else { tf("{} alterações", status.changes.len()) });
        if status.ahead > 0 {
            parts.push(format!("↑{}", status.ahead));
        }
        if status.behind > 0 {
            parts.push(format!("↓{}", status.behind));
        }
        if !status.upstream {
            parts.push(t("só local").to_owned());
        }
        self.summary.set_label(&parts.join(" · "));

        // the branch menu: switch, or make a new one
        let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
        list.add_css_class("git-pop");
        for name in branches {
            let b = gtk::Button::with_label(&name);
            b.add_css_class("files-pop-item");
            if name == status.branch {
                b.add_css_class("current");
            }
            let weak = Rc::downgrade(self);
            b.connect_clicked(move |b| {
                if let Some(bar) = weak.upgrade() {
                    if let Some(p) = b.ancestor(gtk::Popover::static_type()).and_downcast::<gtk::Popover>() {
                        p.popdown();
                    }
                    bar.sync(&["switch", &name]);
                }
            });
            list.append(&b);
        }
        let entry = gtk::Entry::new();
        entry.set_placeholder_text(Some(t("nova branch")));
        let weak = Rc::downgrade(self);
        entry.connect_activate(move |e| {
            let name = e.text().trim().replace(' ', "-");
            if let (Some(bar), false) = (weak.upgrade(), name.is_empty()) {
                if let Some(p) = e.ancestor(gtk::Popover::static_type()).and_downcast::<gtk::Popover>() {
                    p.popdown();
                }
                bar.sync(&["switch", "-c", &name]);
            }
        });
        list.append(&entry);
        let pop = gtk::Popover::new();
        pop.set_child(Some(&list));
        self.branch.set_popover(Some(&pop));

        // what changed: a click opens the file
        let changes = gtk::Box::new(gtk::Orientation::Vertical, 2);
        changes.add_css_class("git-pop");
        if status.changes.is_empty() {
            changes.append(&gtk::Label::new(Some(t("nada mudou desde o último commit"))));
        }
        for (code, path) in status.changes.iter().take(60) {
            let (icon, what) = describe(code);
            let b = gtk::Button::new();
            b.add_css_class("files-pop-item");
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row.append(&icons::icon(icon));
            let l = gtk::Label::new(Some(path));
            l.set_xalign(0.0);
            l.set_hexpand(true);
            l.set_ellipsize(gtk::pango::EllipsizeMode::Start);
            l.set_max_width_chars(40);
            row.append(&l);
            let tag = gtk::Label::new(Some(t(what)));
            tag.add_css_class("dim");
            row.append(&tag);
            b.set_child(Some(&row));
            let (open, file) = (self.open.clone(), gio::File::for_path(dir.join(path)));
            b.connect_clicked(move |_| open(file.clone()));
            changes.append(&b);
        }
        let scroller = gtk::ScrolledWindow::builder().child(&changes).propagate_natural_height(true).max_content_height(420).hscrollbar_policy(gtk::PolicyType::Never).build();
        let pop = gtk::Popover::new();
        pop.set_child(Some(&scroller));
        self.changes.set_popover(Some(&pop));
        self.root.set_reveal_child(true);
    }

    /// pull, switch: run, then show what's true now.
    fn sync(self: &Rc<Self>, command: &[&str]) {
        let Some(dir) = self.dir() else { return };
        let (weak, a) = (Rc::downgrade(self), args(command));
        self.summary.set_label(t("um momento…"));
        glib::spawn_future_local(async move {
            let result = run(dir, "git", a).await;
            let Some(bar) = weak.upgrade() else { return };
            if let Err(e) = result {
                (bar.error)(e);
            }
            bar.refresh();
        });
    }

    fn push(self: &Rc<Self>) {
        let Some(dir) = self.dir() else { return };
        let weak = Rc::downgrade(self);
        self.summary.set_label(t("enviando…"));
        glib::spawn_future_local(async move {
            // the first push of a branch sets where it goes
            let has_upstream = run(dir.clone(), "git", args(&["rev-parse", "--abbrev-ref", "@{u}"])).await.is_ok();
            let push = if has_upstream { args(&["push"]) } else { args(&["push", "-u", "origin", "HEAD"]) };
            let result = run(dir, "git", push).await;
            let Some(bar) = weak.upgrade() else { return };
            if let Err(e) = result {
                (bar.error)(if e.contains("Authentication") || e.contains("could not read Username") { t("o github pediu login: conecte pelo menu ⋮ → conectar ao github").to_owned() } else { e });
            }
            bar.refresh();
        });
    }

    fn commit_dialog(self: &Rc<Self>, anchor: &gtk::Button) {
        let Some(dir) = self.dir() else { return };
        let Some(parent) = anchor.root().and_downcast::<gtk::Window>() else { return };
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Ok(text) = run(dir.clone(), "git", args(&["status", "--porcelain=v1", "-uall"])).await else { return };
            let status = parse_status(&format!("## x\n{text}"));
            let Some(bar) = weak.upgrade() else { return };
            if status.changes.is_empty() {
                (bar.error)(t("nada pra commitar").to_owned());
                return;
            }
            let window = gtk::Window::builder().transient_for(&parent).modal(true).title(t("commit")).default_width(480).default_height(520).build();
            window.add_css_class("files-dialog");
            let column = gtk::Box::new(gtk::Orientation::Vertical, 12);
            column.add_css_class("files-dialog-root");
            let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
            let mut checks = Vec::new();
            for (code, path) in &status.changes {
                let (icon, what) = describe(code);
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                let check = gtk::CheckButton::new();
                check.set_active(true);
                row.append(&check);
                row.append(&icons::icon(icon));
                let l = gtk::Label::new(Some(path));
                l.set_xalign(0.0);
                l.set_hexpand(true);
                l.set_ellipsize(gtk::pango::EllipsizeMode::Start);
                row.append(&l);
                let tag = gtk::Label::new(Some(t(what)));
                tag.add_css_class("dim");
                row.append(&tag);
                list.append(&row);
                checks.push((check, path.clone()));
            }
            let scroller = gtk::ScrolledWindow::builder().child(&list).vexpand(true).hscrollbar_policy(gtk::PolicyType::Never).build();
            column.append(&scroller);
            let message = gtk::Entry::new();
            message.set_placeholder_text(Some(t("o que mudou")));
            column.append(&message);
            let foot = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            let also_push = gtk::CheckButton::with_label(t("e enviar"));
            also_push.set_hexpand(true);
            foot.append(&also_push);
            let go = gtk::Button::with_label(t("commit"));
            go.add_css_class("suggested");
            foot.append(&go);
            column.append(&foot);
            window.set_child(Some(&column));
            window.present();
            message.grab_focus();

            let checks = Rc::new(checks);
            let submit = {
                let (w, message, also_push, bar, dir) = (window.clone(), message.clone(), also_push.clone(), bar.clone(), dir.clone());
                Rc::new(move || {
                    let text = message.text().trim().to_owned();
                    if text.is_empty() {
                        message.add_css_class("error");
                        return;
                    }
                    let paths: Vec<String> = checks.iter().filter(|(c, _)| c.is_active()).map(|(_, p)| p.clone()).collect();
                    if paths.is_empty() {
                        return;
                    }
                    let (bar, dir, push, w) = (bar.clone(), dir.clone(), also_push.is_active(), w.clone());
                    w.set_sensitive(false);
                    glib::spawn_future_local(async move {
                        let mut add = args(&["add", "-A", "--"]);
                        add.extend(paths.iter().map(|p| p.split(" -> ").last().unwrap_or(p).to_owned()));
                        let result = match run(dir.clone(), "git", add).await {
                            Ok(_) => run(dir.clone(), "git", vec!["commit".into(), "-m".into(), text]).await,
                            Err(e) => Err(e),
                        };
                        w.close();
                        match result {
                            Ok(_) if push => bar.push(),
                            Ok(_) => bar.refresh(),
                            Err(e) => (bar.error)(e),
                        }
                    });
                })
            };
            let s = submit.clone();
            message.connect_activate(move |_| s());
            go.connect_clicked(move |_| submit());
        });
    }
}

/// Clone by url, or pick from your GitHub repositories; the clone lands in
/// `into` and opens when done.
pub fn clone_dialog(parent: &gtk::Window, into: gio::File, error: Rc<dyn Fn(String)>, open: Rc<dyn Fn(gio::File)>) {
    let Some(base) = into.path() else { return };
    let window = gtk::Window::builder().transient_for(parent).modal(true).title(t("clonar repositório")).default_width(520).default_height(560).build();
    window.add_css_class("files-dialog");
    let column = gtk::Box::new(gtk::Orientation::Vertical, 12);
    column.add_css_class("files-dialog-root");
    let url = gtk::Entry::new();
    url.set_placeholder_text(Some("https://github.com/user/repo"));
    column.append(&url);
    let place = gtk::Label::new(Some(&tf("vai pra {}", into.parse_name())));
    place.add_css_class("dim");
    place.set_xalign(0.0);
    column.append(&place);

    let heading = gtk::Label::new(Some(t("seus repositórios")));
    heading.add_css_class("files-sidebar-heading");
    heading.set_xalign(0.0);
    column.append(&heading);
    let filter = gtk::SearchEntry::new();
    column.append(&filter);
    let list = gtk::Box::new(gtk::Orientation::Vertical, 2);
    let scroller = gtk::ScrolledWindow::builder().child(&list).vexpand(true).hscrollbar_policy(gtk::PolicyType::Never).build();
    column.append(&scroller);
    let note = gtk::Label::new(Some(t("procurando no github…")));
    note.add_css_class("dim");
    note.set_wrap(true);
    list.append(&note);

    let progress = gtk::ProgressBar::new();
    progress.add_css_class("files-progress");
    progress.set_visible(false);
    column.append(&progress);
    let go = gtk::Button::with_label(t("clonar"));
    go.add_css_class("suggested");
    column.append(&go);
    window.set_child(Some(&column));
    window.present();

    // your repositories, when gh is logged in
    {
        let (list, url, note, filter) = (list.clone(), url.clone(), note.clone(), filter.clone());
        glib::spawn_future_local(async move {
            let found = run(glib::home_dir(), "gh", args(&["repo", "list", "--limit", "100", "--json", "nameWithOwner,description,url"])).await;
            match found.ok().and_then(|s| serde_json::from_str::<Vec<serde_json::Value>>(&s).ok()) {
                Some(repos) => {
                    list.remove(&note);
                    let rows: Rc<Vec<(gtk::Button, String)>> = Rc::new(
                        repos
                            .iter()
                            .map(|r| {
                                let name = r["nameWithOwner"].as_str().unwrap_or_default().to_owned();
                                let about = r["description"].as_str().unwrap_or_default().to_owned();
                                let link = r["url"].as_str().unwrap_or_default().to_owned();
                                let b = gtk::Button::new();
                                b.add_css_class("files-pop-item");
                                let col = gtk::Box::new(gtk::Orientation::Vertical, 0);
                                let n = gtk::Label::new(Some(&name));
                                n.set_xalign(0.0);
                                n.add_css_class("git-repo-name");
                                col.append(&n);
                                if !about.is_empty() {
                                    let a = gtk::Label::new(Some(&about));
                                    a.set_xalign(0.0);
                                    a.add_css_class("dim");
                                    a.set_ellipsize(gtk::pango::EllipsizeMode::End);
                                    col.append(&a);
                                }
                                b.set_child(Some(&col));
                                let u = url.clone();
                                b.connect_clicked(move |_| u.set_text(&format!("{link}.git")));
                                list.append(&b);
                                (b, name.to_lowercase())
                            })
                            .collect(),
                    );
                    filter.connect_search_changed(move |f| {
                        let q = f.text().to_lowercase();
                        for (b, name) in rows.iter() {
                            b.set_visible(name.contains(&q));
                        }
                    });
                }
                None => note.set_label(t("o gh não está conectado: cole a url em cima, ou conecte pelo menu ⋮ → conectar ao github")),
            }
        });
    }

    let start = {
        let (window, url, progress, go) = (window.clone(), url.clone(), progress.clone(), go.clone());
        Rc::new(move || {
            let link = url.text().trim().to_owned();
            if link.is_empty() {
                return;
            }
            let name = link.trim_end_matches('/').trim_end_matches(".git").rsplit('/').next().unwrap_or("repo").to_owned();
            let target = base.join(&name);
            go.set_sensitive(false);
            progress.set_visible(true);
            let (tx, rx) = async_channel::unbounded::<Result<f64, String>>();
            let (cwd, t2) = (base.clone(), target.clone());
            std::thread::spawn(move || {
                use std::io::Read;
                let child = std::process::Command::new("git")
                    .args(["clone", "--progress", &link])
                    .arg(&t2)
                    .current_dir(&cwd)
                    .env("GIT_TERMINAL_PROMPT", "0")
                    .stderr(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::null())
                    .spawn();
                let mut child = match child {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx.send_blocking(Err(e.to_string()));
                        return;
                    }
                };
                let mut stderr = child.stderr.take().unwrap();
                let mut buf = [0u8; 512];
                let mut line = String::new();
                let mut all = String::new();
                while let Ok(n) = stderr.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    for c in String::from_utf8_lossy(&buf[..n]).chars() {
                        if c == '\r' || c == '\n' {
                            // "Receiving objects:  45% (…)" weighs most
                            if let Some(pct) = line.split('%').next().and_then(|s| s.rsplit(' ').next()).and_then(|s| s.parse::<f64>().ok()) {
                                let weight = if line.starts_with("Receiving") { 0.2 + pct / 100.0 * 0.7 } else if line.starts_with("Resolving") { 0.9 + pct / 1000.0 } else { pct / 500.0 };
                                let _ = tx.send_blocking(Ok(weight));
                            }
                            all.push_str(&line);
                            all.push('\n');
                            line.clear();
                        } else {
                            line.push(c);
                        }
                    }
                }
                let ok = child.wait().is_ok_and(|s| s.success());
                let _ = tx.send_blocking(if ok { Ok(1.0) } else { Err(all.lines().filter(|l| l.contains("fatal") || l.contains("error")).collect::<Vec<_>>().join("\n")) });
            });
            let (window, progress, go, error, open) = (window.clone(), progress.clone(), go.clone(), error.clone(), open.clone());
            glib::spawn_future_local(async move {
                while let Ok(update) = rx.recv().await {
                    match update {
                        Ok(f) if f >= 1.0 => {
                            window.close();
                            open(gio::File::for_path(&target));
                            return;
                        }
                        Ok(f) => progress.set_fraction(f),
                        Err(e) => {
                            error(if e.is_empty() { t("o clone falhou").to_owned() } else { e });
                            go.set_sensitive(true);
                            progress.set_visible(false);
                            return;
                        }
                    }
                }
            });
        })
    };
    let s = start.clone();
    url.connect_activate(move |_| s());
    go.connect_clicked(move |_| start());
}

/// `gh auth login` in a terminal: the browser does the rest, no tokens to
/// make by hand. Afterwards git uses gh for https.
pub fn connect_github() {
    super::run_in_terminal("gh auth login --web --git-protocol https --hostname github.com && gh auth setup-git && echo && echo 'pronto, pode fechar' ; read -r _");
}
