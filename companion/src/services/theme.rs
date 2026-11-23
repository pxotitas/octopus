use std::rc::Rc;

use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;

use crate::exec;
use crate::store::Store;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Mode {
    Light,
    Dark,
    /// Light for a light wallpaper, dark for a dark one.
    Auto,
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::Light => "light",
            Mode::Dark => "dark",
            Mode::Auto => "auto",
        }
    }

    fn parse(text: &str) -> Self {
        match text.trim() {
            "light" => Mode::Light,
            "auto" => Mode::Auto,
            _ => Mode::Dark,
        }
    }
}

/// The palette's mode. It lives in a state file that the theme script reads,
/// so Matuwall's hook and the shell agree.
pub struct Theme {
    pub mode: Rc<Store<Mode>>,
    /// What is actually on screen, after resolving `Auto`.
    pub dark: Rc<Store<bool>>,
}

fn state_file() -> std::path::PathBuf {
    std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| glib::home_dir().join(".local/state"))
        .join("octopus")
        .join("mode")
}

/// Mean luminance of an image, 0 to 1, from a small decode.
pub fn luma(path: &str) -> Option<f64> {
    let pixbuf = Pixbuf::from_file_at_scale(path, 48, -1, true).ok()?;
    let (w, h, stride, n) = (
        pixbuf.width() as usize,
        pixbuf.height() as usize,
        pixbuf.rowstride() as usize,
        pixbuf.n_channels() as usize,
    );
    let bytes = pixbuf.read_pixel_bytes();
    let mut sum = 0.0;
    for y in 0..h {
        for x in 0..w {
            let i = y * stride + x * n;
            sum += 0.2126 * bytes[i] as f64
                + 0.7152 * bytes[i + 1] as f64
                + 0.0722 * bytes[i + 2] as f64;
        }
    }
    Some(sum / (w * h) as f64 / 255.0)
}

async fn wallpaper() -> Option<String> {
    let query = exec::output("awww", &["query"]).await?;
    query
        .lines()
        .find_map(|l| l.split("image: ").nth(1))
        .map(|p| p.trim().to_owned())
}

impl Theme {
    pub fn new() -> Rc<Self> {
        let mode = Mode::parse(&std::fs::read_to_string(state_file()).unwrap_or_default());
        let theme = Rc::new(Self {
            mode: Store::new(mode),
            dark: Store::new(mode != Mode::Light),
        });
        if mode == Mode::Auto {
            let dark = theme.dark.clone();
            glib::spawn_future_local(async move {
                if let Some(level) = wallpaper().await.and_then(|p| luma(&p)) {
                    dark.set(level < 0.5);
                }
            });
        }
        theme
    }

    /// The tile: flips what is on screen, leaving automatic mode.
    pub fn toggle(&self) {
        let dark = *self.dark.get();
        self.set_mode(if dark { Mode::Light } else { Mode::Dark });
    }

    pub fn set_mode(&self, mode: Mode) {
        self.mode.set(mode);
        let path = state_file();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(&path, mode.name());
        let dark = self.dark.clone();
        glib::spawn_future_local(async move {
            let image = wallpaper().await;
            let is_dark = match mode {
                Mode::Light => false,
                Mode::Dark => true,
                Mode::Auto => image.as_deref().and_then(luma).is_none_or(|l| l < 0.5),
            };
            dark.set(is_dark);
            exec::spawn(
                "gsettings",
                &[
                    "set",
                    "org.gnome.desktop.interface",
                    "color-scheme",
                    if is_dark {
                        "prefer-dark"
                    } else {
                        "prefer-light"
                    },
                ],
            );
            if let Some(image) = image {
                exec::spawn(&exec::script("theme.sh"), &[&image]);
            }
        });
    }
}
