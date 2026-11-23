use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use futures_util::StreamExt;
use gtk::glib;
use zbus::message::Header;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::{OwnedValue, Structure, Value};

use crate::store::Store;

const WATCHER: &str = "org.kde.StatusNotifierWatcher";
const WATCHER_PATH: &str = "/StatusNotifierWatcher";
const ITEM: &str = "org.kde.StatusNotifierItem";
const MENU: &str = "com.canonical.dbusmenu";

#[derive(Clone, Debug, Default)]
pub struct Item {
    pub bus: String,
    pub path: String,
    pub title: String,
    pub icon_name: String,
    pub icon_theme_path: String,
    /// ARGB32 big endian, the largest the item offers.
    pub pixmap: Option<(i32, i32, Vec<u8>)>,
    pub menu: Option<String>,
    pub item_is_menu: bool,
    pub passive: bool,
}

impl Item {
    fn key(&self) -> String {
        format!("{}{}", self.bus, self.path)
    }
}

#[derive(Clone, Debug, Default)]
pub struct MenuNode {
    pub id: i32,
    pub label: String,
    pub enabled: bool,
    pub separator: bool,
    /// "checkmark" or "radio" with its state.
    pub toggle: Option<(String, bool)>,
    pub children: Vec<MenuNode>,
}

enum Event {
    Registered(String, String),
    Gone(String),
}

/// StatusNotifierWatcher and host: apps register their tray items here, and
/// the panel draws them and their menus itself.
pub struct Tray {
    pub items: Rc<Store<Vec<Item>>>,
    connection: std::cell::RefCell<Option<zbus::Connection>>,
}

impl Tray {
    pub fn new() -> Rc<Self> {
        let tray = Rc::new(Self {
            items: Store::new(Vec::new()),
            connection: std::cell::RefCell::new(None),
        });
        let (tx, rx) = async_channel::unbounded();
        let weak = Rc::downgrade(&tray);
        glib::spawn_future_local(async move {
            let connection = match serve(tx.clone()).await {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("octopus: tray unavailable: {err}");
                    return;
                }
            };
            if let Some(t) = weak.upgrade() {
                *t.connection.borrow_mut() = Some(connection.clone());
            }
            glib::spawn_future_local(watch_owners(connection.clone(), tx));
            while let Ok(event) = rx.recv().await {
                let Some(tray) = weak.upgrade() else { break };
                match event {
                    Event::Registered(bus, path) => tray.add(bus, path),
                    Event::Gone(bus) => tray.items.update(|items| items.retain(|i| i.bus != bus)),
                }
            }
        });
        tray
    }

    fn connection(&self) -> Option<zbus::Connection> {
        self.connection.borrow().clone()
    }

    fn add(self: &Rc<Self>, bus: String, path: String) {
        let Some(connection) = self.connection() else {
            return;
        };
        let weak = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let Ok(proxy) = zbus::Proxy::new(&connection, bus.clone(), path.clone(), ITEM).await
            else {
                return;
            };
            let item = read(&proxy, &bus, &path).await;
            let Some(tray) = weak.upgrade() else { return };
            tray.items.update(|items| {
                items.retain(|i| i.key() != item.key());
                items.push(item);
            });
            let Ok(mut changes) = proxy.receive_all_signals().await else {
                return;
            };
            drop(tray);
            while let Some(signal) = changes.next().await {
                let member = signal
                    .header()
                    .member()
                    .map(|m| m.to_string())
                    .unwrap_or_default();
                if !member.starts_with("New") {
                    continue;
                }
                let fresh = read(&proxy, &bus, &path).await;
                let Some(tray) = weak.upgrade() else { break };
                if !tray.items.get().iter().any(|i| i.key() == fresh.key()) {
                    break;
                }
                tray.items.update(|items| {
                    if let Some(slot) = items.iter_mut().find(|i| i.key() == fresh.key()) {
                        *slot = fresh;
                    }
                });
            }
        });
    }

    /// Left click. Items that are only a menu return false so the caller
    /// shows the menu instead.
    pub async fn activate(&self, item: &Item) -> bool {
        if item.item_is_menu {
            return false;
        }
        let Some(connection) = self.connection() else {
            return false;
        };
        let Ok(proxy) =
            zbus::Proxy::new(&connection, item.bus.clone(), item.path.clone(), ITEM).await
        else {
            return false;
        };
        proxy.call_method("Activate", &(0i32, 0i32)).await.is_ok() || item.menu.is_none()
    }

    pub fn secondary(&self, item: &Item) {
        let Some(connection) = self.connection() else {
            return;
        };
        let item = item.clone();
        glib::spawn_future_local(async move {
            if let Ok(proxy) = zbus::Proxy::new(&connection, item.bus, item.path, ITEM).await {
                let _ = proxy.call_method("SecondaryActivate", &(0i32, 0i32)).await;
            }
        });
    }

    pub async fn menu(&self, item: &Item) -> Option<MenuNode> {
        let connection = self.connection()?;
        let path = item.menu.clone()?;
        let proxy = zbus::Proxy::new(&connection, item.bus.clone(), path, MENU)
            .await
            .ok()?;
        let _ = proxy.call_method("AboutToShow", &(0i32)).await;
        let reply = proxy
            .call_method("GetLayout", &(0i32, -1i32, Vec::<String>::new()))
            .await
            .ok()?;
        type Layout = (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>);
        let (_, (id, props, children)): (u32, Layout) = reply.body().deserialize().ok()?;
        let children: Vec<Value> = children
            .iter()
            .map(|c| (**c).try_clone())
            .filter_map(Result::ok)
            .collect();
        build(id, &props, &children)
    }

    pub fn click(&self, item: &Item, id: i32) {
        let (Some(connection), Some(path)) = (self.connection(), item.menu.clone()) else {
            return;
        };
        let bus = item.bus.clone();
        glib::spawn_future_local(async move {
            if let Ok(proxy) = zbus::Proxy::new(&connection, bus, path, MENU).await {
                let data = Value::from(0i32);
                let _ = proxy
                    .call_method("Event", &(id, "clicked", data, 0u32))
                    .await;
            }
        });
    }
}

async fn read(proxy: &zbus::Proxy<'_>, bus: &str, path: &str) -> Item {
    let text = |v: Result<String, zbus::Error>| v.unwrap_or_default();
    let title = text(proxy.get_property::<String>("Title").await);
    let id = text(proxy.get_property::<String>("Id").await);
    let pixmap = proxy
        .get_property::<Vec<(i32, i32, Vec<u8>)>>("IconPixmap")
        .await
        .ok()
        .and_then(|list| list.into_iter().max_by_key(|(w, h, _)| w * h));
    Item {
        bus: bus.to_owned(),
        path: path.to_owned(),
        title: if title.is_empty() { id } else { title },
        icon_name: text(proxy.get_property::<String>("IconName").await),
        icon_theme_path: text(proxy.get_property::<String>("IconThemePath").await),
        pixmap,
        menu: proxy
            .get_property::<zbus::zvariant::OwnedObjectPath>("Menu")
            .await
            .ok()
            .map(|p| p.to_string())
            .filter(|p| p != "/"),
        item_is_menu: proxy
            .get_property::<bool>("ItemIsMenu")
            .await
            .unwrap_or(false),
        passive: text(proxy.get_property::<String>("Status").await) == "Passive",
    }
}

fn node(s: &Structure) -> Option<MenuNode> {
    let fields = s.fields();
    let id = match fields.first()? {
        Value::I32(id) => *id,
        _ => return None,
    };
    let props: HashMap<String, OwnedValue> = match fields.get(1)? {
        Value::Dict(d) => HashMap::try_from(d.try_clone().ok()?).ok()?,
        _ => HashMap::new(),
    };
    let children = match fields.get(2) {
        Some(Value::Array(array)) => array.iter().filter_map(|v| v.try_clone().ok()).collect(),
        _ => Vec::new(),
    };
    build(id, &props, &children)
}

fn build(id: i32, props: &HashMap<String, OwnedValue>, children: &[Value]) -> Option<MenuNode> {
    let string = |k: &str| {
        props
            .get(k)
            .and_then(|v| String::try_from(v.try_clone().ok()?).ok())
    };
    let boolean = |k: &str, default: bool| {
        props
            .get(k)
            .and_then(|v| bool::try_from(v).ok())
            .unwrap_or(default)
    };
    if !boolean("visible", true) {
        return None;
    }
    let toggle = string("toggle-type").filter(|t| !t.is_empty()).map(|t| {
        (
            t,
            props
                .get("toggle-state")
                .and_then(|v| i32::try_from(v).ok())
                == Some(1),
        )
    });
    let children = children
        .iter()
        .filter_map(|v| match v {
            Value::Value(inner) => match &**inner {
                Value::Structure(s) => node(s),
                _ => None,
            },
            Value::Structure(s) => node(s),
            _ => None,
        })
        .collect();
    Some(MenuNode {
        id,
        label: string("label").unwrap_or_default().replace('_', ""),
        enabled: boolean("enabled", true),
        separator: string("type").as_deref() == Some("separator"),
        toggle,
        children,
    })
}

struct Watcher {
    tx: async_channel::Sender<Event>,
    registered: Arc<Mutex<Vec<String>>>,
}

#[zbus::interface(name = "org.kde.StatusNotifierWatcher")]
impl Watcher {
    async fn register_status_notifier_item(
        &self,
        service: &str,
        #[zbus(header)] header: Header<'_>,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) {
        let sender = header.sender().map(|s| s.to_string()).unwrap_or_default();
        let (bus, path) = if service.starts_with('/') {
            (sender, service.to_owned())
        } else {
            (service.to_owned(), "/StatusNotifierItem".to_owned())
        };
        let key = format!("{bus}{path}");
        if let Ok(mut list) = self.registered.lock() {
            if !list.contains(&key) {
                list.push(key.clone());
            }
        }
        let _ = Self::status_notifier_item_registered(&emitter, &key).await;
        let _ = self.tx.send(Event::Registered(bus, path)).await;
    }

    async fn register_status_notifier_host(&self, _service: &str) {}

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Vec<String> {
        self.registered
            .lock()
            .map(|l| l.clone())
            .unwrap_or_default()
    }

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> bool {
        true
    }

    #[zbus(property)]
    fn protocol_version(&self) -> i32 {
        0
    }

    #[zbus(signal)]
    async fn status_notifier_item_registered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_item_unregistered(
        emitter: &SignalEmitter<'_>,
        service: &str,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    async fn status_notifier_host_registered(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;
}

async fn serve(tx: async_channel::Sender<Event>) -> zbus::Result<zbus::Connection> {
    let watcher = Watcher {
        tx,
        registered: Arc::default(),
    };
    let connection = zbus::connection::Builder::session()?
        .serve_at(WATCHER_PATH, watcher)?
        .build()
        .await?;
    connection
        .request_name_with_flags(
            WATCHER,
            zbus::fdo::RequestNameFlags::ReplaceExisting
                | zbus::fdo::RequestNameFlags::AllowReplacement,
        )
        .await?;
    if let Ok(iface) = connection
        .object_server()
        .interface::<_, Watcher>(WATCHER_PATH)
        .await
    {
        let _ = Watcher::status_notifier_host_registered(iface.signal_emitter()).await;
    }
    Ok(connection)
}

/// Items vanish when the process behind them leaves the bus.
async fn watch_owners(connection: zbus::Connection, tx: async_channel::Sender<Event>) {
    let Ok(dbus) = zbus::fdo::DBusProxy::new(&connection).await else {
        return;
    };
    let Ok(mut changes) = dbus.receive_name_owner_changed().await else {
        return;
    };
    while let Some(change) = changes.next().await {
        let Ok(args) = change.args() else { continue };
        if args.new_owner().is_none() {
            let _ = tx.send(Event::Gone(args.name().to_string())).await;
        }
    }
}
