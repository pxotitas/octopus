//! org.freedesktop.FileManager1: what apps call for "show in folder" (the
//! browser's downloads, a chat's files). With it, those open here.

use gtk::gio;
use gtk::glib;

pub enum Ask {
    /// Folders to open.
    Folders(Vec<String>),
    /// Files to show selected, in their folder.
    Items(Vec<String>),
    /// Files whose info to show.
    Properties(Vec<String>),
}

struct Server {
    tx: async_channel::Sender<Ask>,
}

#[zbus::interface(name = "org.freedesktop.FileManager1")]
impl Server {
    fn show_folders(&self, uris: Vec<String>, _startup_id: String) {
        let _ = self.tx.try_send(Ask::Folders(uris));
    }

    fn show_items(&self, uris: Vec<String>, _startup_id: String) {
        let _ = self.tx.try_send(Ask::Items(uris));
    }

    fn show_item_properties(&self, uris: Vec<String>, _startup_id: String) {
        let _ = self.tx.try_send(Ask::Properties(uris));
    }
}

/// Serves the interface while the app runs; `ask` gets each request.
pub fn serve(ask: impl Fn(Ask) + 'static) {
    let (tx, rx) = async_channel::unbounded();
    glib::spawn_future_local(async move {
        let built = async {
            let connection = zbus::connection::Builder::session()?
                .serve_at("/org/freedesktop/FileManager1", Server { tx })?
                .build()
                .await?;
            connection.request_name_with_flags("org.freedesktop.FileManager1", zbus::fdo::RequestNameFlags::ReplaceExisting.into()).await?;
            zbus::Result::Ok(connection)
        }
        .await;
        let _connection = match built {
            Ok(c) => c,
            Err(err) => {
                eprintln!("octopus files: FileManager1 unavailable: {err}");
                return;
            }
        };
        while let Ok(request) = rx.recv().await {
            ask(request);
        }
    });
}

pub fn files(uris: &[String]) -> Vec<gio::File> {
    uris.iter().map(|u| gio::File::for_uri(u)).collect()
}
