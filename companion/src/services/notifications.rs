use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};

use gtk::glib;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Structure, Value};

use crate::store::{Listeners, Store};

type Arrivals = RefCell<Vec<Box<dyn Fn(&Rc<Notification>)>>>;

const PATH: &str = "/org/freedesktop/Notifications";
const INTERFACE: &str = "org.freedesktop.Notifications";

#[derive(Debug)]
pub enum Image {
    Data {
        width: i32,
        height: i32,
        stride: i32,
        alpha: bool,
        bytes: Vec<u8>,
    },
    Path(String),
}

#[derive(Debug)]
pub struct Notification {
    pub id: u32,
    pub app_name: String,
    pub app_icon: String,
    pub summary: String,
    pub body: String,
    /// The sender's picture (a contact avatar, an album cover) when the app
    /// sends one, apart from its app icon.
    pub image: Option<Image>,
    pub actions: Vec<(String, String)>,
    pub critical: bool,
    pub timeout: i32,
    pub time: chrono::DateTime<chrono::Local>,
}

impl Notification {
    pub fn default_action(&self) -> Option<&str> {
        self.actions
            .iter()
            .find(|(key, _)| key == "default")
            .map(|(key, _)| key.as_str())
    }
}

enum Event {
    Notify(Notification, u32),
    Close(u32),
}

#[repr(u32)]
#[derive(Clone, Copy)]
pub enum Reason {
    Dismissed = 2,
    Closed = 3,
}

pub struct Notifications {
    pub list: Rc<Store<Vec<Rc<Notification>>>>,
    pub dnd: Rc<Store<bool>>,
    arrivals: Arrivals,
    removals: Listeners<u32>,
    connection: RefCell<Option<zbus::Connection>>,
}

impl Notifications {
    pub fn new() -> Rc<Self> {
        let this = Rc::new(Self {
            list: Store::new(Vec::new()),
            dnd: Store::new(false),
            arrivals: RefCell::new(Vec::new()),
            removals: RefCell::new(Vec::new()),
            connection: RefCell::new(None),
        });

        let (tx, rx) = async_channel::unbounded();
        let weak = Rc::downgrade(&this);
        glib::spawn_future_local(async move {
            match serve(tx).await {
                Ok(connection) => {
                    if let Some(this) = weak.upgrade() {
                        *this.connection.borrow_mut() = Some(connection);
                    }
                }
                Err(err) => eprintln!("octopus: notification server unavailable: {err}"),
            }
            while let Ok(event) = rx.recv().await {
                let Some(this) = weak.upgrade() else { break };
                match event {
                    Event::Notify(notification, replaces) => this.add(notification, replaces),
                    Event::Close(id) => this.remove(id, Reason::Closed),
                }
            }
        });
        this
    }

    pub fn on_arrival(&self, f: impl Fn(&Rc<Notification>) + 'static) {
        self.arrivals.borrow_mut().push(Box::new(f));
    }

    pub fn on_removal(&self, f: impl Fn(u32) + 'static) {
        self.removals.borrow_mut().push(Box::new(f));
    }

    fn add(&self, notification: Notification, replaces: u32) {
        let notification = Rc::new(notification);
        self.list.update(|list| {
            list.retain(|n| n.id != replaces && n.id != notification.id);
            list.insert(0, notification.clone());
        });
        if replaces != 0 {
            for f in self.removals.borrow().iter() {
                f(replaces);
            }
        }
        if !*self.dnd.get() || notification.critical {
            for f in self.arrivals.borrow().iter() {
                f(&notification);
            }
        }
    }

    pub fn dismiss(&self, id: u32) {
        self.remove(id, Reason::Dismissed);
    }

    pub fn clear(&self) {
        let ids: Vec<u32> = self.list.get().iter().map(|n| n.id).collect();
        for id in ids {
            self.remove(id, Reason::Dismissed);
        }
    }

    fn remove(&self, id: u32, reason: Reason) {
        if !self.list.get().iter().any(|n| n.id == id) {
            return;
        }
        self.list.update(|list| list.retain(|n| n.id != id));
        for f in self.removals.borrow().iter() {
            f(id);
        }
        self.signal("NotificationClosed", (id, reason as u32));
    }

    pub fn invoke(&self, id: u32, action: &str) {
        self.signal("ActionInvoked", (id, action.to_owned()));
        self.remove(id, Reason::Dismissed);
    }

    fn signal<B>(&self, name: &'static str, body: B)
    where
        B: serde::Serialize + zbus::zvariant::DynamicType + 'static,
    {
        let Some(connection) = self.connection.borrow().clone() else {
            return;
        };
        glib::spawn_future_local(async move {
            let _ = connection
                .emit_signal(None::<()>, PATH, INTERFACE, name, &body)
                .await;
        });
    }
}

struct Server {
    tx: async_channel::Sender<Event>,
    next: AtomicU32,
}

#[zbus::interface(name = "org.freedesktop.Notifications")]
impl Server {
    fn get_capabilities(&self) -> Vec<&'static str> {
        vec![
            "body",
            "body-markup",
            "body-hyperlinks",
            "actions",
            "icon-static",
            "persistence",
        ]
    }

    #[allow(clippy::too_many_arguments)]
    async fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        app_icon: String,
        summary: String,
        body: String,
        actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let id = if replaces_id != 0 {
            replaces_id
        } else {
            self.next.fetch_add(1, Ordering::Relaxed)
        };
        let critical = hints
            .get("urgency")
            .and_then(|v| u8::try_from(v).ok())
            .is_some_and(|u| u >= 2);
        let notification = Notification {
            id,
            app_name,
            app_icon,
            summary,
            body,
            image: image(&hints),
            actions: actions
                .chunks(2)
                .filter(|p| p.len() == 2)
                .map(|p| (p[0].clone(), p[1].clone()))
                .collect(),
            critical,
            timeout: expire_timeout,
            time: chrono::Local::now(),
        };
        let _ = self.tx.send(Event::Notify(notification, replaces_id)).await;
        id
    }

    async fn close_notification(&self, id: u32) {
        let _ = self.tx.send(Event::Close(id)).await;
    }

    fn get_server_information(&self) -> (&'static str, &'static str, &'static str, &'static str) {
        ("octopus", "pxota", env!("CARGO_PKG_VERSION"), "1.2")
    }

    #[zbus(signal)]
    async fn notification_closed(
        emitter: &SignalEmitter<'_>,
        id: u32,
        reason: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn action_invoked(
        emitter: &SignalEmitter<'_>,
        id: u32,
        action_key: &str,
    ) -> zbus::Result<()>;
}

async fn serve(tx: async_channel::Sender<Event>) -> zbus::Result<zbus::Connection> {
    let server = Server {
        tx,
        next: AtomicU32::new(1),
    };
    let connection = zbus::connection::Builder::session()?
        .serve_at(PATH, server)?
        .build()
        .await?;
    connection
        .request_name_with_flags(
            INTERFACE,
            zbus::fdo::RequestNameFlags::ReplaceExisting
                | zbus::fdo::RequestNameFlags::AllowReplacement,
        )
        .await?;
    Ok(connection)
}

fn image(hints: &HashMap<String, OwnedValue>) -> Option<Image> {
    for key in ["image-data", "image_data", "icon_data"] {
        if let Some(Value::Structure(s)) = hints.get(key).map(|v| &**v) {
            if let Some(image) = image_data(s) {
                return Some(image);
            }
        }
    }
    for key in ["image-path", "image_path"] {
        if let Some(Value::Str(path)) = hints.get(key).map(|v| &**v) {
            if !path.is_empty() {
                return Some(Image::Path(path.to_string()));
            }
        }
    }
    None
}

fn image_data(s: &Structure) -> Option<Image> {
    let f = s.fields();
    let int = |i: usize| match f.get(i) {
        Some(Value::I32(v)) => Some(*v),
        _ => None,
    };
    let alpha = matches!(f.get(3), Some(Value::Bool(true)));
    let bytes = match f.get(6) {
        Some(Value::Array(array)) => array
            .iter()
            .filter_map(|v| match v {
                Value::U8(b) => Some(*b),
                _ => None,
            })
            .collect(),
        _ => return None,
    };
    Some(Image::Data {
        width: int(0)?,
        height: int(1)?,
        stride: int(2)?,
        alpha,
        bytes,
    })
}
