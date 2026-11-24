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

    /// How far into line `index` the singing is at `position`, from 0 to 1.
    /// Lyrics carry one time per line, so the words are spread over the
    /// line at a singing pace (about 13 characters a second), and never past
    /// the next line, so a pause before it does not stretch the words.
    pub fn progress(&self, index: usize, position: f64) -> f64 {
        let Some(line) = self.lines.get(index) else {
            return 0.0;
        };
        let pace = line.text.chars().count() as f64 / 13.0 + 0.4;
        let end = self
            .lines
            .get(index + 1)
            .map(|next| next.t.min(line.t + pace))
            .unwrap_or(line.t + pace);
        let span = (end - line.t).max(0.3);
        ((position + 0.25 - line.t) / span).clamp(0.0, 1.0)
    }
}
