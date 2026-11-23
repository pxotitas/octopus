mod bar;
mod config;
mod exec;
mod i18n;
mod icons;
mod ipc;
mod lock;
mod memory;
mod panels;
mod popup;
mod services;
mod shell;
mod store;
mod theme;
mod widgets;

use gtk::prelude::*;
use gtk::{gio, glib};

const APP_ID: &str = "dev.pxota.Octopus";

const USAGE: &str = "usage: octopus [daemon]
       octopus toggle <control-center|dash|wifi|bluetooth|audio|tray|clipboard|power>
       octopus settings
       octopus close
       octopus lock
       octopus volume <up|down|mute>
       octopus mic mute
       octopus brightness <up|down>
       octopus kbd <up|down>
       octopus dnd
       octopus luma <image>
       octopus reload";

fn main() -> glib::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("daemon") => run_daemon(),
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

    app.run_with_args::<&str>(&[])
}
