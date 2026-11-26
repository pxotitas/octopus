use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

use gtk::{gio, glib};

/// Runs a program to completion off the main thread and returns its stdout
/// when it exits successfully. Output is parsed, so the locale is pinned:
/// nmcli answers "sim"/"não" instead of "yes"/"no" under pt_BR.
pub async fn output(program: &str, args: &[&str]) -> Option<String> {
    let program = program.to_owned();
    let args: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
    let result = gio::spawn_blocking(move || {
        Command::new(program)
            .args(args)
            .env("LC_ALL", "C.UTF-8")
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .output()
    })
    .await
    .ok()?
    .ok()?;
    result
        .status
        .success()
        .then(|| String::from_utf8_lossy(&result.stdout).into_owned())
}

/// Like `output`, but reports stderr on failure.
pub async fn run(program: &str, args: &[String]) -> Result<String, String> {
    let program = program.to_owned();
    let args = args.to_vec();
    let result = gio::spawn_blocking(move || {
        Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
    })
    .await
    .map_err(|_| "task panicked".to_owned())?
    .map_err(|err| err.to_string())?;
    if result.status.success() {
        Ok(String::from_utf8_lossy(&result.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&result.stderr).trim().to_owned();
        Err(if err.is_empty() {
            String::from_utf8_lossy(&result.stdout).trim().to_owned()
        } else {
            err
        })
    }
}

/// Starts a program and forgets it; GSubprocess reaps it when it exits.
pub fn spawn(program: &str, args: &[&str]) {
    let argv: Vec<&OsStr> = std::iter::once(program)
        .chain(args.iter().copied())
        .map(OsStr::new)
        .collect();
    if let Err(err) = gio::Subprocess::newv(&argv, gio::SubprocessFlags::NONE) {
        eprintln!("octopus: cannot start {program}: {err}");
    }
}

/// A long-running program whose stdout lines are delivered on the main loop.
/// Dropping the handle kills the program.
pub struct Stream {
    child: Arc<Mutex<Child>>,
}

impl Drop for Stream {
    fn drop(&mut self) {
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

const PR_SET_PDEATHSIG: i32 = 1;
const SIGTERM: i32 = 15;

extern "C" {
    fn prctl(option: i32, ...) -> i32;
    fn kill(pid: i32, signal: i32) -> i32;
    fn waitpid(pid: i32, status: *mut i32, options: i32) -> i32;
}

static STREAMS: Mutex<Vec<u32>> = Mutex::new(Vec::new());

/// Ends every long-running child, for when the daemon replaces itself and no
/// destructor gets to run.
pub fn end_streams() {
    if let Ok(pids) = STREAMS.lock() {
        for &pid in pids.iter() {
            unsafe {
                kill(pid as i32, SIGTERM);
                waitpid(pid as i32, std::ptr::null_mut(), 0);
            }
        }
    }
}

pub fn stream(program: &str, args: &[&str], on_line: impl Fn(String) + 'static) -> Option<Stream> {
    use std::os::unix::process::CommandExt;
    let mut command = Command::new(program);
    unsafe {
        // the child goes when the daemon goes, even if it crashes
        command.pre_exec(|| {
            prctl(PR_SET_PDEATHSIG, SIGTERM);
            Ok(())
        });
    }
    let mut child = command
        .args(args)
        .env("LC_ALL", "C.UTF-8")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| eprintln!("octopus: cannot start {program}: {err}"))
        .ok()?;
    if let Ok(mut pids) = STREAMS.lock() {
        pids.push(child.id());
    }
    let stdout = child.stdout.take()?;
    let (tx, rx) = async_channel::unbounded::<String>();

    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if tx.send_blocking(line).is_err() {
                break;
            }
        }
    });
    glib::spawn_future_local(async move {
        while let Ok(line) = rx.recv().await {
            on_line(line);
        }
    });

    Some(Stream {
        child: Arc::new(Mutex::new(child)),
    })
}

/// Runs `action` once things go quiet for `ms` milliseconds, however many
/// times `trigger` is called before that.
pub fn debounce(ms: u64, action: impl Fn() + 'static) -> impl Fn() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let pending: Rc<RefCell<Option<glib::SourceId>>> = Rc::default();
    let action = Rc::new(action);
    move || {
        if let Some(id) = pending.borrow_mut().take() {
            id.remove();
        }
        let action = action.clone();
        let slot = pending.clone();
        let id = glib::timeout_add_local_once(std::time::Duration::from_millis(ms), move || {
            slot.borrow_mut().take();
            action();
        });
        *pending.borrow_mut() = Some(id);
    }
}

pub fn config_dir() -> std::path::PathBuf {
    glib::user_config_dir().join("octopus")
}

pub fn cache_dir() -> std::path::PathBuf {
    glib::user_cache_dir().join("octopus")
}

pub fn script(name: &str) -> String {
    config_dir()
        .join("scripts")
        .join(name)
        .to_string_lossy()
        .into_owned()
}
