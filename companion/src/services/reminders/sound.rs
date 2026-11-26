//! The alarm's ring, looped until it's answered (or about two minutes pass),
//! and a single chime for reminders.

use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};

const RING: &str = "/usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga";
const CHIME: &str = "/usr/share/sounds/freedesktop/stereo/message-new-instant.oga";

pub fn ring() -> Option<Child> {
    Command::new("sh")
        .args(["-c", &format!("for i in $(seq 40); do pw-play '{RING}' || paplay '{RING}'; sleep 0.4; done")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .ok()
}

pub fn chime() {
    let _ = Command::new("pw-play").arg(CHIME).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}

/// Stops the loop and the sound playing now: the whole process group.
pub fn stop(child: &mut Child) {
    let _ = Command::new("kill").args(["--", &format!("-{}", child.id())]).status();
    let _ = child.wait();
}
