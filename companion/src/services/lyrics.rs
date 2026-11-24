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

impl Lyrics {
    /// Index of the line being sung at `position`.
    pub fn current(&self, position: f64) -> Option<usize> {
        if !self.synced {
            return None;
        }
        self.lines
            .iter()
            .rposition(|line| line.t <= position + 0.25)
    }
}
