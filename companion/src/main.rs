mod bar;
mod config;
mod desktop;
mod dock;
mod exec;
mod files;
mod i18n;
mod icons;
mod ipc;
mod launcher;
mod lazy;
mod lock;
mod memory;
mod panels;
mod popup;
mod services;
mod setup;
mod shell;
mod store;
mod theme;
mod widgets;

use gtk::prelude::*;
use gtk::{gio, glib};

const APP_ID: &str = "dev.pxota.Octopus";

const USAGE: &str = "usage: octopus [daemon]
       octopus toggle <control-center|dash|wifi|bluetooth|audio|tray|clipboard|emoji|power|launcher>
       octopus settings
       octopus close
       octopus lock
       octopus volume <up|down|mute>
       octopus mic mute
       octopus brightness <up|down>
       octopus kbd <up|down>
       octopus dnd
       octopus caffeine [on|off]
       octopus remind <when> <what>   (10 min tea, 18h bread, alarm 6:45)
       octopus reminders
       octopus files [path]
       octopus setup                  (after installing the package)
       octopus luma <image>
       octopus reload
       octopus restart";

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("daemon") => run_daemon(),
        Some("files") => files::run(&args[1..]),
        Some("setup") => setup::run(),
        Some("-h" | "--help" | "help") => {
            println!("{USAGE}");
            glib::ExitCode::SUCCESS
        }
        Some("luma") => match args.get(1).and_then(|p| services::theme::luma(p)) {
            Some(l) => {
                println!("{}", if l < 0.5 { "dark" } else { "light" });
                glib::ExitCode::SUCCESS
            }
            None => glib::ExitCode::FAILURE,
        },
        Some(_) => match ipc::send(&args.join(" ")) {
            Ok(reply) => {
                if !reply.is_empty() {
                    println!("{reply}");
                }
                glib::ExitCode::SUCCESS
            }
            Err(err) => {
                eprintln!("octopus: {err} (is the daemon running?)");
                glib::ExitCode::FAILURE
            }
        },
    }
}

fn run_daemon() -> glib::ExitCode {
    memory::tune();
    if let Err(err) = setup::sync() {
        eprintln!("octopus: couldn't update ~/.config/octopus: {err}");
    }
    let app = gtk::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(|app| {
        if !gtk4_layer_shell::is_supported() {
            eprintln!("octopus: the compositor does not support wlr-layer-shell");
            app.quit();
            return;
        }
        let guard = app.hold();
        let shell = shell::Shell::new(app);
        let commands = match ipc::listen() {
            Ok(rx) => rx,
            Err(err) => {
                eprintln!("octopus: cannot open the control socket: {err}");
                app.quit();
                return;
            }
        };
        glib::spawn_future_local(async move {
            let _guard = guard;
            while let Ok(request) = commands.recv().await {
                let reply = shell.handle(&request.command);
                request.reply(reply);
            }
        });
    });

    let code = app.run_with_args::<&str>(&[]);
    if RESTART.load(std::sync::atomic::Ordering::Relaxed) {
        use std::os::unix::process::CommandExt;
        exec::end_streams();
        // after an update the running binary is gone and Linux names it
        // "… (deleted)"; the new one sits at the same path
        let exe = std::env::current_exe().map(|exe| {
            let name = exe.to_string_lossy().into_owned();
            match name.strip_suffix(" (deleted)") {
                Some(path) => std::path::PathBuf::from(path),
                None => exe,
            }
        });
        if let Ok(exe) = exe {
            let err = std::process::Command::new(exe).exec();
            eprintln!("octopus: cannot restart: {err}");
        }
    }
    code
}

static RESTART: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Starts the daemon over, for what is only read at startup (the language).
pub fn restart(app: &gtk::Application) {
    RESTART.store(true, std::sync::atomic::Ordering::Relaxed);
    app.quit();
}
