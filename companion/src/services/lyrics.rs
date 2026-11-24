use std::sync::atomic::{AtomicI64, Ordering};

use serde::Deserialize;

use crate::exec;
use crate::services::media::Track;

#[derive(Clone, Debug, Deserialize)]
pub struct Line {
    pub t: f64,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Lyrics {
    pub available: bool,
    #[serde(default)]
    pub synced: bool,
    #[serde(default)]
    pub instrumental: bool,
    #[serde(default)]
    pub lines: Vec<Line>,
}

/// lrclib.net through the lyrics script, which caches every answer.
pub async fn fetch(track: &Track) -> Option<Lyrics> {
    let script = exec::script("lyrics.py");
    let length = format!("{}", track.length.round());
    let out = exec::output(
        "python3",
        &[&script, &track.artist, &track.title, &track.album, &length],
    )
    .await?;
    serde_json::from_str(&out).ok()
}

/// How early a line lights, so it is on screen as it is sung rather than
/// after: the eye needs the line a beat before the voice reaches it.
const LEAD: f64 = 0.45;

/// The user's correction, in milliseconds, from `[media] lyrics_offset_ms`.
static OFFSET_MS: AtomicI64 = AtomicI64::new(0);

pub fn set_offset(ms: i64) {
    OFFSET_MS.store(ms, Ordering::Relaxed);
}

impl Lyrics {
    /// Index of the line being sung at `position`.
    pub fn current(&self, position: f64) -> Option<usize> {
        if !self.synced {
            return None;
        }
        self.lines.iter().rposition(|line| {
            line.t <= position + LEAD + OFFSET_MS.load(Ordering::Relaxed) as f64 / 1000.0
        })
    }
}
