use std::collections::HashMap;
use std::path::PathBuf;

use gio_unix::prelude::*;
use gio_unix::DesktopAppInfo;
use gtk::gio;
use gtk::prelude::*;

/// An app, or one of the actions its .desktop file offers.
pub struct Entry {
    pub name: String,
    pub detail: String,
    pub icon: Option<gio::Icon>,
    pub app: DesktopAppInfo,
    pub action: Option<String>,
    id: String,
    words: Vec<String>,
    extra: String,
}

/// Every app that shows itself, searched by name, generic name, keywords
/// and command, with what was launched most often first.
pub struct Apps {
    entries: Vec<Entry>,
    counts: HashMap<String, u32>,
}

fn state() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| gtk::glib::home_dir().join(".local/state"))
        .join("octopus")
        .join("launches")
}

fn fold(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            other => other,
        })
        .collect()
}

fn subsequence(needle: &str, hay: &str) -> bool {
    let mut chars = hay.chars();
    needle.chars().all(|n| chars.any(|h| h == n))
}

impl Apps {
    pub fn load() -> Self {
        let mut entries = Vec::new();
        for info in gio::AppInfo::all() {
            if !info.should_show() {
                continue;
            }
            let Ok(app) = info.downcast::<DesktopAppInfo>() else {
                continue;
            };
            let name = app.display_name().to_string();
            let id = app
                .id()
                .map(|i| i.to_string())
                .unwrap_or_else(|| name.clone());
            let generic = app
                .generic_name()
                .map(|g| g.to_string())
                .unwrap_or_default();
            let command = app
                .executable()
                .and_then(|e| e.file_name().map(|f| f.to_string_lossy().into_owned()))
                .unwrap_or_default();
            let keywords: Vec<String> = app.keywords().iter().map(|k| k.to_string()).collect();
            let extra = fold(&format!("{generic} {} {command}", keywords.join(" ")));
            for action in app.list_actions() {
                let label = app.action_name(&action).to_string();
                entries.push(Entry {
                    words: fold(&format!("{label} {name}"))
                        .split_whitespace()
                        .map(str::to_owned)
                        .collect(),
                    extra: fold(&name),
                    detail: name.clone(),
                    name: label,
                    icon: app.icon(),
                    id: format!("{id}#{action}"),
                    action: Some(action.to_string()),
                    app: app.clone(),
                });
            }
            entries.push(Entry {
                words: fold(&name).split_whitespace().map(str::to_owned).collect(),
                extra,
                detail: generic,
                icon: app.icon(),
                id,
                action: None,
                name,
                app,
            });
        }
        let counts = std::fs::read_to_string(state())
            .unwrap_or_default()
            .lines()
            .filter_map(|l| {
                let (count, id) = l.split_once('\t')?;
                Some((id.to_owned(), count.parse().ok()?))
            })
            .collect();
        Self { entries, counts }
    }

    fn score(&self, entry: &Entry, query: &str) -> Option<f64> {
        let name = fold(&entry.name);
        let mut score = if name.starts_with(query) {
            100.0
        } else if entry.words.iter().any(|w| w.starts_with(query)) {
            80.0
        } else if name.contains(query) {
            60.0
        } else if entry.extra.contains(query) {
            40.0
        } else if query.chars().count() >= 3 && subsequence(query, &name) {
            20.0
        } else {
            return None;
        };
        if entry.action.is_some() {
            if query.chars().count() < 2 {
                return None;
            }
            score -= 25.0;
        }
        let used = self.counts.get(&entry.id).copied().unwrap_or(0).min(30) as f64;
        Some(score + used * 1.5 - name.len() as f64 * 0.05)
    }

    /// The best `limit` entries for `query`; with no query, the most used.
    pub fn search(&self, query: &str, limit: usize) -> Vec<usize> {
        let query = fold(query.trim());
        let mut found: Vec<(usize, f64)> = if query.is_empty() {
            self.entries
                .iter()
                .enumerate()
                .filter(|(_, e)| e.action.is_none())
                .filter_map(|(i, e)| self.counts.get(&e.id).map(|c| (i, *c as f64)))
                .collect()
        } else {
            self.entries
                .iter()
                .enumerate()
                .filter_map(|(i, e)| self.score(e, &query).map(|s| (i, s)))
                .collect()
        };
        found.sort_by(|a, b| b.1.total_cmp(&a.1));
        found.into_iter().take(limit).map(|(i, _)| i).collect()
    }

    /// Every app (not their actions): the most opened first, then by name.
    pub fn ranked(&self) -> Vec<usize> {
        let mut list: Vec<usize> = (0..self.entries.len())
            .filter(|i| self.entries[*i].action.is_none())
            .collect();
        list.sort_by_cached_key(|i| {
            let e = &self.entries[*i];
            (
                std::cmp::Reverse(self.counts.get(&e.id).copied().unwrap_or(0)),
                fold(&e.name),
            )
        });
        list
    }

    pub fn get(&self, index: usize) -> &Entry {
        &self.entries[index]
    }

    pub fn launch(&mut self, index: usize) {
        let entry = &self.entries[index];
        let context = gtk::gdk::Display::default().map(|d| d.app_launch_context());
        let result = match &entry.action {
            Some(action) => {
                entry.app.launch_action(action, context.as_ref());
                Ok(())
            }
            None => entry.app.launch(&[], context.as_ref()),
        };
        if let Err(err) = result {
            eprintln!("octopus: cannot launch {}: {err}", entry.name);
            return;
        }
        *self.counts.entry(entry.id.clone()).or_insert(0) += 1;
        let mut text = String::new();
        for (id, count) in &self.counts {
            text.push_str(&format!("{count}\t{id}\n"));
        }
        let path = state();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, text);
    }
}
