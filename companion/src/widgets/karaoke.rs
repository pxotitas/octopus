//! A lyric line lit word by word, karaoke style.

use gtk::glib;

/// Pango markup for `text` with the words up to `fraction` of it (by
/// characters) lit, and the rest faded. Whole words light at once, so the
/// line reads as sung word by word.
pub fn markup(text: &str, fraction: f64) -> String {
    let total = text.chars().count().max(1) as f64;
    let sung = fraction.clamp(0.0, 1.0) * total;
    let mut lit = String::new();
    let mut rest = String::new();
    let mut seen = 0usize;
    for piece in text.split_inclusive(char::is_whitespace) {
        let start = seen as f64;
        seen += piece.chars().count();
        if start < sung || fraction >= 1.0 {
            lit.push_str(piece);
        } else {
            rest.push_str(piece);
        }
    }
    let mut out = glib::markup_escape_text(&lit).to_string();
    if !rest.is_empty() {
        out.push_str(&format!(
            "<span alpha=\"38%\">{}</span>",
            glib::markup_escape_text(&rest)
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::markup;

    #[test]
    fn lights_whole_words() {
        assert_eq!(markup("a bc d", 0.0), "<span alpha=\"38%\">a bc d</span>");
        assert_eq!(markup("a bc d", 0.4), "a bc <span alpha=\"38%\">d</span>");
        assert_eq!(markup("a & b", 1.0), "a &amp; b");
    }
}
