use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

pub struct Request {
    pub command: String,
    reply: mpsc::Sender<String>,
}

impl Request {
    pub fn reply(self, text: String) {
        let _ = self.reply.send(text);
    }
}

fn socket_path() -> PathBuf {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    dir.join("octopus.sock")
}

pub fn send(command: &str) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(socket_path())?;
    stream.set_read_timeout(Some(Duration::from_secs(3)))?;
    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;
    let mut reply = String::new();
    stream.read_to_string(&mut reply)?;
    Ok(reply.trim_end().to_owned())
}

pub fn listen() -> std::io::Result<async_channel::Receiver<Request>> {
    let path = socket_path();
    if UnixStream::connect(&path).is_ok() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "another octopus daemon is already running",
        ));
    }
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path)?;
    let (tx, rx) = async_channel::unbounded();

    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut line = String::new();
            if BufReader::new(&stream).read_line(&mut line).is_err() {
                continue;
            }
            let (reply_tx, reply_rx) = mpsc::channel();
            let request = Request {
                command: line.trim().to_owned(),
                reply: reply_tx,
            };
            if tx.send_blocking(request).is_err() {
                break;
            }
            let reply = reply_rx
                .recv_timeout(Duration::from_secs(2))
                .unwrap_or_default();
            let mut stream = stream;
            let _ = stream.write_all(reply.as_bytes());
        }
    });

    Ok(rx)
}
