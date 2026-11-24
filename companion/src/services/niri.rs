use std::rc::Rc;

use serde_json::Value;

use crate::exec;
use crate::store::Store;

/// A toplevel as niri reports it. `tile` is its rectangle in the
/// workspace's view (x, y, width, height), when it is on screen.
#[derive(Clone, Debug, PartialEq)]
pub struct Window {
    pub id: u64,
    pub title: String,
    pub app_id: String,
    pub workspace: Option<u64>,
    pub focused: bool,
    pub tile: Option<[f64; 4]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Workspace {
    pub id: u64,
    pub output: String,
    pub active: bool,
}

/// niri's windows and workspaces, kept current from its event stream. Only
/// started by what needs it (the dock).
pub struct Niri {
    pub windows: Rc<Store<Vec<Window>>>,
    pub workspaces: Rc<Store<Vec<Workspace>>>,
    _events: Option<exec::Stream>,
}

fn tile(layout: &Value) -> Option<[f64; 4]> {
    let pos = layout["tile_pos_in_workspace_view"].as_array()?;
    let size = layout["tile_size"].as_array()?;
    Some([
        pos.first()?.as_f64()?,
        pos.get(1)?.as_f64()?,
        size.first()?.as_f64()?,
        size.get(1)?.as_f64()?,
    ])
}

fn window(v: &Value) -> Option<Window> {
    let tile = tile(&v["layout"]);
    Some(Window {
        id: v["id"].as_u64()?,
        title: v["title"].as_str().unwrap_or("").to_owned(),
        app_id: v["app_id"].as_str().unwrap_or("").to_owned(),
        workspace: v["workspace_id"].as_u64(),
        focused: v["is_focused"].as_bool().unwrap_or(false),
        tile,
    })
}

fn workspace(v: &Value) -> Option<Workspace> {
    Some(Workspace {
        id: v["id"].as_u64()?,
        output: v["output"].as_str().unwrap_or("").to_owned(),
        active: v["is_active"].as_bool().unwrap_or(false),
    })
}

fn apply(windows: &Store<Vec<Window>>, workspaces: &Store<Vec<Workspace>>, line: &str) {
    let Ok(event) = serde_json::from_str::<Value>(line) else {
        return;
    };
    let Some((name, body)) = event.as_object().and_then(|o| o.iter().next()) else {
        return;
    };
    match name.as_str() {
        "WindowsChanged" => {
            let list = body["windows"]
                .as_array()
                .map(|l| l.iter().filter_map(window).collect());
            windows.set(list.unwrap_or_default());
        }
        "WindowOpenedOrChanged" => {
            let Some(w) = window(&body["window"]) else {
                return;
            };
            windows.update(|list| {
                if w.focused {
                    for other in list.iter_mut() {
                        other.focused = false;
                    }
                }
                match list.iter_mut().find(|o| o.id == w.id) {
                    Some(slot) => *slot = w,
                    None => list.push(w),
                }
            });
        }
        "WindowClosed" => {
            let id = body["id"].as_u64();
            windows.update(|list| list.retain(|w| Some(w.id) != id));
        }
        "WindowFocusChanged" => {
            let id = body["id"].as_u64();
            windows.update(|list| {
                for w in list.iter_mut() {
                    w.focused = Some(w.id) == id;
                }
            });
        }
        "WindowLayoutsChanged" => {
            let Some(changes) = body["changes"].as_array() else {
                return;
            };
            windows.update(|list| {
                for change in changes {
                    let (Some(id), Some(layout)) = (change[0].as_u64(), change.get(1)) else {
                        continue;
                    };
                    if let Some(w) = list.iter_mut().find(|w| w.id == id) {
                        w.tile = tile(layout);
                    }
                }
            });
        }
        "WorkspacesChanged" => {
            let list = body["workspaces"]
                .as_array()
                .map(|l| l.iter().filter_map(workspace).collect());
            workspaces.set(list.unwrap_or_default());
        }
        "WorkspaceActivated" => {
            let Some(id) = body["id"].as_u64() else {
                return;
            };
            workspaces.update(|list| {
                let output = list.iter().find(|w| w.id == id).map(|w| w.output.clone());
                for w in list.iter_mut() {
                    if Some(&w.output) == output.as_ref() {
                        w.active = w.id == id;
                    }
                }
            });
        }
        _ => {}
    }
}

impl Niri {
    pub fn new() -> Rc<Self> {
        let windows: Rc<Store<Vec<Window>>> = Store::new(Vec::new());
        let workspaces: Rc<Store<Vec<Workspace>>> = Store::new(Vec::new());
        let events = exec::stream("niri", &["msg", "--json", "event-stream"], {
            let (windows, workspaces) = (windows.clone(), workspaces.clone());
            move |line| apply(&windows, &workspaces, &line)
        });
        Rc::new(Self {
            windows,
            workspaces,
            _events: events,
        })
    }

    pub fn focus(&self, id: u64) {
        exec::spawn(
            "niri",
            &["msg", "action", "focus-window", "--id", &id.to_string()],
        );
    }

    pub fn close(&self, id: u64) {
        exec::spawn(
            "niri",
            &["msg", "action", "close-window", "--id", &id.to_string()],
        );
    }
}
