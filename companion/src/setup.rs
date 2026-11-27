//! Octopus installed as a package keeps its files in /usr/share/octopus. The
//! shell and its scripts read them from ~/.config/octopus, so they're copied
//! there, and copied again whenever the package brings a new version. A clone
//! of the repo living in ~/.config/octopus is never touched.

use std::path::{Path, PathBuf};

use gtk::glib;

use crate::exec;

const DATA: &str = match option_env!("OCTOPUS_DATA_DIR") {
    Some(dir) => dir,
    None => "/usr/share/octopus",
};

/// Replaced on every update: nobody edits these.
const MANAGED: &[&str] = &["scripts", "matugen", "assets", "rofi/config.rasi", "waybar/style.css", "cava.conf"];
/// Copied once; from then on they're yours.
const SEEDED: &[&str] = &["octopus.toml", "niri.kdl"];
const STAMP: &str = ".octopus-version";

/// OCTOPUS_DATA_DIR at runtime wins, for trying a package out of its usual place.
fn data() -> PathBuf {
    std::env::var_os("OCTOPUS_DATA_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(DATA))
}

fn version() -> Option<String> {
    std::fs::read_to_string(data().join("VERSION")).ok().map(|v| v.trim().to_owned())
}

fn is_clone(dir: &Path) -> bool {
    dir.join(".git").exists()
}

fn copy(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        if let Some(parent) = to.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(from, to).map(|_| ())
    }
}

/// Brings ~/.config/octopus up to the installed version. Nothing happens
/// without a package, with a clone, or when it's already current; the answer
/// says whether files were copied.
pub fn sync() -> Result<bool, String> {
    let Some(version) = version() else { return Ok(false) };
    let dir = exec::config_dir();
    if is_clone(&dir) {
        return Ok(false);
    }
    let stamp = dir.join(STAMP);
    if std::fs::read_to_string(&stamp).is_ok_and(|v| v.trim() == version) {
        return Ok(false);
    }
    for part in MANAGED {
        let from = data().join(part);
        if from.exists() {
            copy(&from, &dir.join(part)).map_err(|e| format!("{part}: {e}"))?;
        }
    }
    for part in SEEDED {
        let to = dir.join(part);
        if !to.exists() {
            copy(&data().join(part), &to).map_err(|e| format!("{part}: {e}"))?;
        }
    }
    std::fs::write(&stamp, &version).map_err(|e| format!("{STAMP}: {e}"))?;
    Ok(true)
}

/// Waybar reads its config from ~/.config/waybar, which becomes a link to
/// octopus' own; a waybar config already there is moved aside first.
fn link_waybar(dir: &Path) -> Result<(), String> {
    let target = dir.join("waybar");
    let path = glib::user_config_dir().join("waybar");
    if std::fs::read_link(&path).is_ok_and(|t| t == target) {
        return Ok(());
    }
    if path.exists() || path.is_symlink() {
        let aside = path.with_file_name(format!("waybar.bak-{}", glib::DateTime::now_local().map(|d| d.to_unix()).unwrap_or(0)));
        std::fs::rename(&path, &aside).map_err(|e| format!("waybar: {e}"))?;
        println!("  moved aside: {}", aside.display());
    }
    std::os::unix::fs::symlink(&target, &path).map_err(|e| format!("waybar: {e}"))?;
    println!("  linked: {}", path.display());
    Ok(())
}

/// `octopus setup`: once after installing the package.
pub fn run() -> glib::ExitCode {
    let dir = exec::config_dir();
    if is_clone(&dir) {
        println!("{} is a clone of the repo: install.sh sets it up, and git pull updates it.", dir.display());
        return glib::ExitCode::SUCCESS;
    }
    let Some(version) = version() else {
        eprintln!("octopus: no installed files in {DATA}; from a clone, run install.sh instead");
        return glib::ExitCode::FAILURE;
    };
    println!(":: octopus {version}");
    match sync() {
        Ok(true) => println!("  copied to {}", dir.display()),
        Ok(false) => println!("  {} is up to date", dir.display()),
        Err(err) => {
            eprintln!("octopus: couldn't copy the files: {err}");
            return glib::ExitCode::FAILURE;
        }
    }
    if let Err(err) = link_waybar(&dir) {
        eprintln!("octopus: {err}");
        return glib::ExitCode::FAILURE;
    }
    if !dir.join("colors.css").exists() {
        println!("  the colors come with the first wallpaper you pick (Mod+Shift+W)");
    }
    println!(
        "\nDone. Add this line to the end of ~/.config/niri/config.kdl:\n\n  include \"~/.config/octopus/niri.kdl\"\n\n\
         and remove the old shell's spawn-at-startup lines and any binds it repeats.\n\
         Updates come with the package; octopus copies them over when it starts."
    );
    glib::ExitCode::SUCCESS
}
