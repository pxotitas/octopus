use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{gdk, gio};

use crate::exec;

const BUILTIN: &str = include_str!("../style/octopus.css");

thread_local! {
    static MONITORS: RefCell<Vec<gio::FileMonitor>> = const { RefCell::new(Vec::new()) };
}

/// Colors come from matugen (`colors.css`); the stylesheet is built in, and a
/// `style.css` next to them is appended for personal overrides.
pub fn install() {
    let provider = gtk::CssProvider::new();
    load(&provider);
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_USER,
        );
    }

    let reload = exec::debounce(120, {
        let provider = provider.clone();
        move || load(&provider)
    });
    let reload = std::rc::Rc::new(reload);
    for name in ["colors.css", "style.css"] {
        let file = gio::File::for_path(exec::config_dir().join(name));
        if let Ok(monitor) =
            file.monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
        {
            let reload = reload.clone();
            monitor.connect_changed(move |_, _, _, _| reload());
            MONITORS.with(|monitors| monitors.borrow_mut().push(monitor));
        }
    }
}

fn load(provider: &gtk::CssProvider) {
    let dir = exec::config_dir();
    let colors =
        std::fs::read_to_string(dir.join("colors.css")).unwrap_or_else(|_| FALLBACK.to_owned());
    let overrides = std::fs::read_to_string(dir.join("style.css")).unwrap_or_default();
    provider.load_from_string(&format!("{colors}\n{BUILTIN}\n{overrides}"));
}

/// Used until matugen writes the first palette.
const FALLBACK: &str = "
@define-color primary #b7c4ff;
@define-color on_primary #1f2d61;
@define-color primary_container #364479;
@define-color on_primary_container #dce1ff;
@define-color secondary #c2c5dd;
@define-color secondary_container #424659;
@define-color on_secondary_container #dee1f9;
@define-color tertiary #e3bada;
@define-color error #ffb4ab;
@define-color on_error #690005;
@define-color surface #121318;
@define-color surface_container_lowest #0d0e13;
@define-color surface_container_low #1a1b21;
@define-color surface_container #1e1f25;
@define-color surface_container_high #292a2f;
@define-color surface_container_highest #34343a;
@define-color on_surface #e3e1e9;
@define-color on_surface_variant #c6c5d0;
@define-color outline #90909a;
@define-color outline_variant #45464f;
";
