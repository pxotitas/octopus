//! Someone's latest commits on github, from the public commit search: no
//! token, and within its limits when asked every quarter hour.

use std::rc::Rc;
use std::time::Duration;

use gtk::glib;
use serde::Deserialize;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Debug)]
pub struct Commit {
    pub repo: String,
    /// The first line of the message.
    pub message: String,
    pub sha: String,
    pub url: String,
    pub when: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Deserialize)]
struct Search {
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    sha: String,
    html_url: String,
    repository: Repo,
    commit: Inner,
}

#[derive(Deserialize)]
struct Repo {
    full_name: String,
}

#[derive(Deserialize)]
struct Inner {
    message: String,
    author: Author,
}

#[derive(Deserialize)]
struct Author {
    date: String,
}

pub struct Github {
    pub commits: Rc<Store<Option<Vec<Commit>>>>,
}

impl Github {
    pub fn new(user: &str) -> Rc<Self> {
        let github = Rc::new(Self {
            commits: Store::new(None),
        });
        if user.is_empty() {
            return github;
        }
        let url = format!(
            "https://api.github.com/search/commits?q=author:{user}&sort=author-date&order=desc&per_page=8"
        );
        let fetch = {
            let store = github.commits.clone();
            move || {
                let (store, url) = (store.clone(), url.clone());
                glib::spawn_future_local(async move {
                    let out = exec::output(
                        "curl",
                        &["-sS", "--max-time", "15", "-H", "Accept: application/vnd.github+json", &url],
                    )
                    .await;
                    let Some(search) = out.and_then(|o| serde_json::from_str::<Search>(&o).ok()) else {
                        return;
                    };
                    let commits = search
                        .items
                        .into_iter()
                        .filter_map(|i| {
                            Some(Commit {
                                repo: i.repository.full_name,
                                message: i.commit.message.lines().next().unwrap_or("").to_owned(),
                                sha: i.sha.chars().take(7).collect(),
                                url: i.html_url,
                                when: chrono::DateTime::parse_from_rfc3339(&i.commit.author.date).ok()?,
                            })
                        })
                        .collect();
                    store.set(Some(commits));
                });
            }
        };
        fetch();
        glib::timeout_add_local(Duration::from_secs(15 * 60), move || {
            fetch();
            glib::ControlFlow::Continue
        });
        github
    }
}

/// "agora", "há 5 min", "há 3 h", "há 2 dias".
pub fn ago(when: &chrono::DateTime<chrono::FixedOffset>) -> String {
    let minutes = (chrono::Utc::now() - when.with_timezone(&chrono::Utc)).num_minutes().max(0);
    match minutes {
        0 => crate::i18n::t("agora").to_owned(),
        1..=59 => crate::i18n::tf("há {} min", minutes),
        60..=1439 => crate::i18n::tf("há {} h", minutes / 60),
        1440..=2879 => crate::i18n::t("há 1 dia").to_owned(),
        _ => crate::i18n::tf("há {} dias", minutes / 1440),
    }
}
