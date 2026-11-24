use std::path::PathBuf;

use crate::exec;

/// Files in the home folder whose name has `query`, through fd (or plocate
/// when fd is missing). Only asked for behind the files prefix.
pub async fn search(query: &str, limit: usize) -> Vec<PathBuf> {
    let home = gtk::glib::home_dir();
    let home_text = home.to_string_lossy().into_owned();
    let limit_text = limit.to_string();
    let out = match exec::output(
        "fd",
        &[
            "-i",
            "--absolute-path",
            "--max-results",
            &limit_text,
            "--",
            query,
            &home_text,
        ],
    )
    .await
    {
        Some(out) => out,
        None => exec::output("plocate", &["-i", "-b", "-l", "200", "--", query])
            .await
            .unwrap_or_default(),
    };
    out.lines()
        .filter(|l| l.starts_with(&home_text))
        .take(limit)
        .map(PathBuf::from)
        .collect()
}

/// `~/…` for a path under the home folder.
pub fn pretty(path: &std::path::Path) -> String {
    let home = gtk::glib::home_dir();
    match path.strip_prefix(&home) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => path.display().to_string(),
    }
}
