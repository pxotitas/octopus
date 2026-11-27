//! What changes files: copy, move, trash, delete, rename, new folder,
//! compress and extract. Each runs off the main loop, reports its progress,
//! and leaves a way back for ctrl+z.

use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gdk, gio, glib};

use crate::i18n::{t, tf};
use crate::store::Store;

pub enum Undo {
    /// (from, to): moved or renamed; undone by moving back.
    Moved(Vec<(gio::File, gio::File)>),
    /// Copies or new folders; undone by sending them to the trash.
    Created(Vec<gio::File>),
    /// Originals sent to the trash; undone by restoring them.
    Trashed(Vec<String>),
}

pub struct Ops {
    /// What's running: a label and 0..1, or None.
    pub progress: Rc<Store<Option<(String, f64)>>>,
    /// The last error, for the window to show.
    pub error: Rc<Store<Option<String>>>,
    undo: RefCell<Vec<Undo>>,
    cut: RefCell<Vec<String>>,
}

thread_local! {
    static OPS: Rc<Ops> = Rc::new(Ops {
        progress: Store::new(None),
        error: Store::new(None),
        undo: RefCell::default(),
        cut: RefCell::default(),
    });
}

pub fn get() -> Rc<Ops> {
    OPS.with(|o| o.clone())
}

const P: glib::Priority = glib::Priority::DEFAULT;

/// "name.txt" → "name 2.txt", until it's free.
fn free_name(dir: &gio::File, name: &str) -> gio::File {
    let candidate = dir.child(name);
    if !candidate.query_exists(gio::Cancellable::NONE) {
        return candidate;
    }
    let (stem, ext) = match name.rfind('.') {
        Some(i) if i > 0 => (&name[..i], &name[i..]),
        _ => (name, ""),
    };
    (2..)
        .map(|n| dir.child(format!("{stem} {n}{ext}")))
        .find(|f| !f.query_exists(gio::Cancellable::NONE))
        .unwrap_or(candidate)
}

fn copy_tree<'a>(src: &'a gio::File, dst: &'a gio::File, done: &'a dyn Fn()) -> Pin<Box<dyn Future<Output = Result<(), glib::Error>> + 'a>> {
    Box::pin(async move {
        let info = src.query_info_future("standard::type", gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS, P).await?;
        if info.file_type() == gio::FileType::Directory {
            dst.make_directory_future(P).await?;
            let children = src.enumerate_children_future("standard::name", gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS, P).await?;
            loop {
                let batch = children.next_files_future(64, P).await?;
                if batch.is_empty() {
                    break;
                }
                for child in batch {
                    let name = child.name();
                    copy_tree(&src.child(&name), &dst.child(&name), done).await?;
                }
            }
        } else {
            let (copy, _progress) = src.copy_future(dst, gio::FileCopyFlags::NOFOLLOW_SYMLINKS | gio::FileCopyFlags::ALL_METADATA, P);
            copy.await?;
        }
        done();
        Ok(())
    })
}

async fn count(files: &[gio::File]) -> usize {
    let paths: Vec<Option<std::path::PathBuf>> = files.iter().map(|f| f.path()).collect();
    gio::spawn_blocking(move || {
        fn walk(p: &std::path::Path) -> usize {
            match std::fs::symlink_metadata(p) {
                Ok(m) if m.is_dir() => 1 + std::fs::read_dir(p).map(|d| d.flatten().map(|e| walk(&e.path())).sum()).unwrap_or(0),
                _ => 1,
            }
        }
        paths.iter().map(|p| p.as_deref().map(walk).unwrap_or(1)).sum::<usize>()
    })
    .await
    .unwrap_or(files.len())
    .max(1)
}

impl Ops {
    fn fail(&self, what: &str, err: impl std::fmt::Display) {
        self.error.set(Some(format!("{what}: {err}")));
    }

    pub fn push_undo(&self, undo: Undo) {
        let mut u = self.undo.borrow_mut();
        u.push(undo);
        if u.len() > 50 {
            u.remove(0);
        }
    }

    /// Copies (or moves) files into a folder; a copy into the same folder
    /// gets a free name, like Finder's duplicates.
    pub async fn transfer(self: Rc<Self>, files: Vec<gio::File>, into: gio::File, moving: bool) {
        let total = count(&files).await as f64;
        let label = if moving { tf("movendo {} itens", files.len()) } else { tf("copiando {} itens", files.len()) };
        let done_count = std::cell::Cell::new(0.0);
        let tick = || {
            done_count.set(done_count.get() + 1.0);
            self.progress.set(Some((label.clone(), (done_count.get() / total).min(1.0))));
        };
        self.progress.set(Some((label.clone(), 0.0)));
        let mut moved = Vec::new();
        let mut created = Vec::new();
        for src in &files {
            let Some(name) = src.basename() else { continue };
            let name = name.to_string_lossy().into_owned();
            if moving && src.parent().is_some_and(|p| p.equal(&into)) {
                continue;
            }
            if into.has_prefix(src) || into.equal(src) {
                self.fail(&name, t("não dá pra pôr uma pasta dentro dela mesma"));
                continue;
            }
            let dst = free_name(&into, &name);
            if moving {
                let (s, d) = (src.clone(), dst.clone());
                let quick = gio::spawn_blocking(move || s.move_(&d, gio::FileCopyFlags::NOFOLLOW_SYMLINKS, gio::Cancellable::NONE, None).map_err(|e| e.to_string()))
                    .await
                    .unwrap_or_else(|_| Err(String::new()));
                match quick {
                    Ok(()) => {
                        tick();
                        moved.push((src.clone(), dst));
                    }
                    // another disk: copy, then remove the original
                    Err(_) => match copy_tree(src, &dst, &tick).await {
                        Ok(()) => {
                            self.clone().remove(vec![src.clone()]).await;
                            moved.push((src.clone(), dst));
                        }
                        Err(e) => self.fail(&name, e),
                    },
                }
            } else {
                match copy_tree(src, &dst, &tick).await {
                    Ok(()) => created.push(dst),
                    Err(e) => self.fail(&name, e),
                }
            }
        }
        if !moved.is_empty() {
            self.push_undo(Undo::Moved(moved));
        }
        if !created.is_empty() {
            self.push_undo(Undo::Created(created));
        }
        self.progress.set(None);
    }

    pub async fn trash(self: Rc<Self>, files: Vec<gio::File>) {
        let mut originals = Vec::new();
        for f in &files {
            match f.trash_future(P).await {
                Ok(()) => {
                    if let Some(p) = f.path() {
                        originals.push(p.to_string_lossy().into_owned());
                    }
                }
                Err(e) => self.fail(&crate::files::display_name(f), e),
            }
        }
        if !originals.is_empty() {
            self.push_undo(Undo::Trashed(originals));
        }
    }

    /// Gone for good (the trash itself, or shift+delete).
    pub async fn remove(self: Rc<Self>, files: Vec<gio::File>) {
        for f in files {
            let result = match f.path() {
                Some(path) => gio::spawn_blocking(move || {
                    if std::fs::symlink_metadata(&path).is_ok_and(|m| m.is_dir()) {
                        std::fs::remove_dir_all(&path)
                    } else {
                        std::fs::remove_file(&path)
                    }
                    .map_err(|e| e.to_string())
                })
                .await
                .unwrap_or_else(|_| Err(String::new())),
                None => f.delete_future(P).await.map_err(|e| e.to_string()),
            };
            if let Err(e) = result {
                self.fail(&crate::files::display_name(&f), e);
            }
        }
    }

    pub async fn empty_trash(self: Rc<Self>) {
        let trash = gio::File::for_uri("trash:///");
        let Ok(children) = trash.enumerate_children_future("standard::name", gio::FileQueryInfoFlags::NONE, P).await else { return };
        while let Ok(batch) = children.next_files_future(64, P).await {
            if batch.is_empty() {
                break;
            }
            for info in batch {
                let _ = trash.child(info.name()).delete_future(P).await;
            }
        }
    }

    pub async fn rename(self: Rc<Self>, file: gio::File, name: String) -> Option<gio::File> {
        match file.set_display_name_future(&name, P).await {
            Ok(renamed) => {
                self.push_undo(Undo::Moved(vec![(file, renamed.clone())]));
                Some(renamed)
            }
            Err(e) => {
                self.fail(&name, e);
                None
            }
        }
    }

    pub async fn new_folder(self: Rc<Self>, inside: gio::File) -> Option<gio::File> {
        let folder = free_name(&inside, t("pasta sem título"));
        match folder.make_directory_future(P).await {
            Ok(()) => {
                self.push_undo(Undo::Created(vec![folder.clone()]));
                Some(folder)
            }
            Err(e) => {
                self.fail(t("nova pasta"), e);
                None
            }
        }
    }

    pub async fn undo(self: Rc<Self>) {
        let Some(last) = self.undo.borrow_mut().pop() else { return };
        match last {
            Undo::Moved(pairs) => {
                for (from, to) in pairs {
                    let _ = gio::spawn_blocking(move || to.move_(&from, gio::FileCopyFlags::NOFOLLOW_SYMLINKS, gio::Cancellable::NONE, None).is_ok()).await;
                }
            }
            Undo::Created(files) => {
                for f in files {
                    let _ = f.trash_future(P).await;
                }
            }
            Undo::Trashed(originals) => {
                let trash = gio::File::for_uri("trash:///");
                let Ok(children) = trash.enumerate_children_future("standard::name,trash::orig-path", gio::FileQueryInfoFlags::NONE, P).await else { return };
                while let Ok(batch) = children.next_files_future(64, P).await {
                    if batch.is_empty() {
                        break;
                    }
                    for info in batch {
                        let orig = info.attribute_byte_string("trash::orig-path").map(|s| s.to_string()).unwrap_or_default();
                        if originals.contains(&orig) {
                            let item = trash.child(info.name());
                            let back = gio::File::for_path(&orig);
                            let _ = gio::spawn_blocking(move || item.move_(&back, gio::FileCopyFlags::NONE, gio::Cancellable::NONE, None).is_ok()).await;
                        }
                    }
                }
            }
        }
    }

    /// zip for anything; the archive lands beside the files.
    pub async fn compress(self: Rc<Self>, files: Vec<gio::File>) {
        let Some(dir) = files.first().and_then(|f| f.parent()) else { return };
        let Some(cwd) = dir.path() else { return };
        let name = if files.len() == 1 { format!("{}.zip", crate::files::display_name(&files[0])) } else { t("arquivos.zip").to_owned() };
        let target = free_name(&dir, &name);
        let names: Vec<String> = files.iter().filter_map(|f| f.basename()).map(|b| b.to_string_lossy().into_owned()).collect();
        let Some(out) = target.path() else { return };
        self.progress.set(Some((t("compactando").to_owned(), 0.5)));
        let ok = gio::spawn_blocking(move || {
            std::process::Command::new("zip").arg("-rqy").arg(&out).args(&names).current_dir(&cwd).status().is_ok_and(|s| s.success())
        })
        .await
        .unwrap_or(false);
        self.progress.set(None);
        if ok {
            self.push_undo(Undo::Created(vec![target]));
        } else {
            self.fail(t("compactar"), t("o zip falhou"));
        }
    }

    pub fn can_extract(name: &str) -> bool {
        let n = name.to_lowercase();
        [".zip", ".tar", ".tar.gz", ".tgz", ".tar.xz", ".txz", ".tar.bz2", ".tar.zst", ".7z", ".rar"].iter().any(|e| n.ends_with(e))
    }

    /// Into a folder named after the archive, beside it.
    pub async fn extract(self: Rc<Self>, archive: gio::File) {
        let (Some(dir), Some(path)) = (archive.parent(), archive.path()) else { return };
        let name = crate::files::display_name(&archive);
        let lower = name.to_lowercase();
        let stem = [".tar.gz", ".tar.xz", ".tar.bz2", ".tar.zst", ".tgz", ".txz", ".zip", ".tar", ".7z", ".rar"]
            .iter()
            .find_map(|e| lower.ends_with(e).then(|| name[..name.len() - e.len()].to_owned()))
            .unwrap_or(name.clone());
        let target = free_name(&dir, &stem);
        let Some(out) = target.path() else { return };
        self.progress.set(Some((t("extraindo").to_owned(), 0.5)));
        let ok = gio::spawn_blocking(move || {
            let _ = std::fs::create_dir_all(&out);
            let mut cmd = if lower.ends_with(".zip") {
                let mut c = std::process::Command::new("unzip");
                c.arg("-q").arg(&path).arg("-d").arg(&out);
                c
            } else if lower.ends_with(".7z") || lower.ends_with(".rar") {
                let mut c = std::process::Command::new("bsdtar");
                c.arg("-xf").arg(&path).arg("-C").arg(&out);
                c
            } else {
                let mut c = std::process::Command::new("tar");
                c.arg("-xf").arg(&path).arg("-C").arg(&out);
                c
            };
            cmd.status().is_ok_and(|s| s.success())
        })
        .await
        .unwrap_or(false);
        self.progress.set(None);
        if ok {
            self.push_undo(Undo::Created(vec![target]));
        } else {
            self.fail(&name, t("não deu pra extrair"));
        }
    }

    /// Copy or cut to the clipboard, in the form other file managers read.
    pub fn to_clipboard(&self, widget: &impl IsA<gtk::Widget>, files: &[gio::File], cut: bool) {
        *self.cut.borrow_mut() = if cut { files.iter().map(|f| f.uri().to_string()).collect() } else { Vec::new() };
        let list = gdk::FileList::from_array(files);
        let uris: Vec<String> = files.iter().map(|f| f.uri().to_string()).collect();
        let gnome = format!("{}\n{}", if cut { "cut" } else { "copy" }, uris.join("\n"));
        let provider = gdk::ContentProvider::new_union(&[
            gdk::ContentProvider::for_value(&list.to_value()),
            gdk::ContentProvider::for_bytes("x-special/gnome-copied-files", &glib::Bytes::from_owned(gnome.into_bytes())),
            gdk::ContentProvider::for_bytes("text/plain;charset=utf-8", &glib::Bytes::from_owned(files.iter().filter_map(|f| f.path()).map(|p| p.to_string_lossy().into_owned()).collect::<Vec<_>>().join("\n").into_bytes())),
        ]);
        let _ = widget.as_ref().clipboard().set_content(Some(&provider));
    }

    pub async fn paste(self: Rc<Self>, widget: gtk::Widget, into: gio::File) {
        let clipboard = widget.clipboard();
        let Ok(value) = clipboard.read_value_future(gdk::FileList::static_type(), P).await else { return };
        let Ok(list) = value.get::<gdk::FileList>() else { return };
        let files = list.files();
        let cut = {
            let cut = self.cut.borrow();
            !cut.is_empty() && files.iter().all(|f| cut.contains(&f.uri().to_string()))
        };
        if cut {
            self.cut.borrow_mut().clear();
            clipboard.set_content(None::<&gdk::ContentProvider>).ok();
        }
        self.transfer(files, into, cut).await;
    }
}
