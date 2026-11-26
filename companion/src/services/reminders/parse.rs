//! Reminders typed in a few words, in portuguese or english:
//! `10 min tea`, `em 1h30 reunião`, `7:30 wake up`, `amanhã 9h dentista`,
//! `alarm 6:45`, `sexta 18h buy bread`, `tomorrow at 9 call mom`.

use chrono::{DateTime, Datelike, Duration, Local, NaiveTime, TimeZone, Weekday};

#[derive(Debug, PartialEq)]
pub struct Quick {
    pub at: DateTime<Local>,
    pub title: String,
    /// The text asked for an alarm ("alarme", "alarm", "acordar").
    pub alarm: bool,
}

fn fold(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' => 'a',
            'é' | 'ê' => 'e',
            'í' => 'i',
            'ó' | 'ô' | 'õ' => 'o',
            'ú' | 'ü' => 'u',
            'ç' => 'c',
            c => c,
        })
        .collect()
}

const ALARM: &[&str] = &["alarme", "alarm", "despertador"];
const REMIND: &[&str] = &["lembrete", "lembrar", "lembra", "remind", "reminder"];
/// Words that only lead into a time or a title: dropped when next to one.
const LEAD: &[&str] = &["me", "em", "in", "daqui", "at", "as", "a", "on", "no", "na", "de", "pra", "para", "to", "que", "for", "por"];

fn unit_minutes(word: &str) -> Option<i64> {
    match word {
        "m" | "min" | "mins" | "minuto" | "minutos" | "minute" | "minutes" => Some(1),
        "h" | "hr" | "hrs" | "hora" | "horas" | "hour" | "hours" => Some(60),
        _ => None,
    }
}

/// `10m`, `90min`, `2h`, `1h30` written after "em"/"in": minutes from now.
fn span(word: &str, after_in: bool) -> Option<i64> {
    let digits: String = word.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || digits.len() == word.len() {
        return None;
    }
    let n: i64 = digits.parse().ok()?;
    let rest = &word[digits.len()..];
    if let Some(m) = rest.strip_prefix('h') {
        if !m.is_empty() && m.chars().all(|c| c.is_ascii_digit()) {
            return after_in.then(|| n * 60 + m.parse::<i64>().unwrap_or(0));
        }
    }
    let unit = unit_minutes(rest)?;
    // "18h" alone is a time of day; "2h" after "em"/"in" is a span
    if unit == 60 && rest == "h" && !after_in {
        return None;
    }
    Some(n * unit)
}

/// `7:30`, `07h30`, `18h`, `6pm`, `6:30am`: a time of day.
fn clock(word: &str) -> Option<NaiveTime> {
    let (body, shift) = if let Some(b) = word.strip_suffix("pm") {
        (b, 12)
    } else if let Some(b) = word.strip_suffix("am") {
        (b, 0)
    } else {
        (word, -1)
    };
    let (h, m) = if let Some((h, m)) = body.split_once(':') {
        (h, m)
    } else if let Some((h, m)) = body.split_once('h') {
        (h, if m.is_empty() { "0" } else { m })
    } else if shift >= 0 {
        (body, "0")
    } else {
        return None;
    };
    let mut h: u32 = h.parse().ok()?;
    let m: u32 = m.parse().ok()?;
    if shift == 12 && h < 12 {
        h += 12;
    } else if shift == 0 && h == 12 {
        h = 0;
    }
    NaiveTime::from_hms_opt(h, m, 0)
}

fn weekday(word: &str) -> Option<Weekday> {
    Some(match word {
        "segunda" | "seg" | "monday" | "mon" => Weekday::Mon,
        "terca" | "ter" | "tuesday" | "tue" => Weekday::Tue,
        "quarta" | "qua" | "wednesday" | "wed" => Weekday::Wed,
        "quinta" | "qui" | "thursday" | "thu" => Weekday::Thu,
        "sexta" | "sex" | "friday" | "fri" => Weekday::Fri,
        "sabado" | "sab" | "saturday" | "sat" => Weekday::Sat,
        "domingo" | "dom" | "sunday" | "sun" => Weekday::Sun,
        _ => return None,
    })
}

pub fn parse(text: &str, now: DateTime<Local>) -> Option<Quick> {
    let original: Vec<&str> = text.split_whitespace().collect();
    let words: Vec<String> = original.iter().map(|w| fold(&w.to_lowercase())).collect();
    let mut used = vec![false; words.len()];
    let mut alarm = false;
    let mut minutes: Option<i64> = None;
    let mut time: Option<NaiveTime> = None;
    let mut day: Option<i64> = None;

    let mut i = 0;
    while i < words.len() {
        let w = words[i].as_str();
        let before = if i > 0 { words[i - 1].as_str() } else { "" };
        let after_in = matches!(before, "em" | "in" | "daqui");
        let free = minutes.is_none() && time.is_none();
        let number = w.chars().all(|c| c.is_ascii_digit());
        let next_unit = words.get(i + 1).and_then(|n| unit_minutes(n));

        // "alarme"/"lembrar" only count as the first word ("me lembra" too);
        // later on they're part of the title
        let leading = used[..i].iter().all(|u| *u) || words[..i].iter().all(|w| w == "me");
        if leading && ALARM.contains(&w) {
            alarm = true;
            used[i] = true;
        } else if w == "acordar" || (w == "wake" && words.get(i + 1).is_some_and(|n| n == "up")) {
            alarm = true;
        } else if leading && REMIND.contains(&w) {
            used[i] = true;
        } else if free && span(w, after_in).is_some() {
            minutes = span(w, after_in);
            used[i] = true;
        } else if free && number && next_unit.is_some() {
            minutes = Some(w.parse::<i64>().ok()? * next_unit?);
            used[i] = true;
            used[i + 1] = true;
            i += 1;
        } else if free && clock(w).is_some() {
            time = clock(w);
            used[i] = true;
        } else if free && number && matches!(before, "as" | "at") {
            time = NaiveTime::from_hms_opt(w.parse().ok()?, 0, 0);
            used[i] = true;
        } else if day.is_none() && matches!(w, "hoje" | "today") {
            day = Some(0);
            used[i] = true;
        } else if day.is_none() && matches!(w, "amanha" | "tomorrow") {
            day = Some(1);
            used[i] = true;
        } else if day.is_none() && weekday(w).is_some() {
            let wd = weekday(w)?;
            let ahead = (wd.num_days_from_monday() as i64 - now.weekday().num_days_from_monday() as i64).rem_euclid(7);
            day = Some(if ahead == 0 { 7 } else { ahead });
            used[i] = true;
        }
        i += 1;
    }
    if minutes.is_none() && time.is_none() {
        return None;
    }
    // lead-in words before what was read ("em", "às", "me") and before the
    // title ("to", "pra") go too
    for _ in 0..3 {
        for i in 0..words.len() {
            if used[i] || !LEAD.contains(&words[i].as_str()) {
                continue;
            }
            let next_used = used.get(i + 1).copied().unwrap_or(false);
            let all_before_used = used[..i].iter().all(|u| *u);
            if next_used || all_before_used {
                used[i] = true;
            }
        }
    }

    let at = match (minutes, time) {
        (Some(m), _) => now + Duration::minutes(m),
        (None, Some(t)) => {
            let date = now.date_naive() + Duration::days(day.unwrap_or(0));
            let mut at = Local.from_local_datetime(&date.and_time(t)).single()?;
            if day.is_none() && at <= now {
                at += Duration::days(1);
            }
            at
        }
        _ => return None,
    };
    let title: Vec<&str> = original.iter().zip(&used).filter(|(_, u)| !**u).map(|(w, _)| *w).collect();
    Some(Quick { at, title: title.join(" "), alarm })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn now() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 11, 26, 14, 0, 0).unwrap()
    }

    fn at(d: u32, h: u32, m: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 11, d, h, m, 0).unwrap()
    }

    #[test]
    fn relative() {
        let q = parse("10 min tea", now()).unwrap();
        assert_eq!((q.at, q.title.as_str(), q.alarm), (now() + Duration::minutes(10), "tea", false));
        let q = parse("em 1h30 reunião", now()).unwrap();
        assert_eq!((q.at, q.title.as_str()), (now() + Duration::minutes(90), "reunião"));
        let q = parse("remind me in 25m to stretch", now()).unwrap();
        assert_eq!((q.at, q.title.as_str()), (now() + Duration::minutes(25), "stretch"));
        let q = parse("me lembra em 2h de tirar o bolo", now()).unwrap();
        assert_eq!((q.at, q.title.as_str()), (now() + Duration::minutes(120), "tirar o bolo"));
    }

    #[test]
    fn clock_times() {
        let q = parse("alarm 6:45", now()).unwrap();
        assert_eq!((q.at, q.title.as_str(), q.alarm), (at(27, 6, 45), "", true));
        let q = parse("18h comprar pão de queijo", now()).unwrap();
        assert_eq!((q.at, q.title.as_str()), (at(26, 18, 0), "comprar pão de queijo"));
        let q = parse("amanhã às 9 dentista", now()).unwrap();
        assert_eq!((q.at, q.title.as_str()), (at(27, 9, 0), "dentista"));
        let q = parse("tomorrow at 9:30pm call mom", now()).unwrap();
        assert_eq!((q.at, q.title.as_str()), (at(27, 21, 30), "call mom"));
        let q = parse("sexta 18h buy bread", now()).unwrap();
        assert_eq!((q.at, q.title.as_str()), (at(27, 18, 0), "buy bread"));
        let q = parse("alarme 18:59 test alarm", now()).unwrap();
        assert_eq!((q.at, q.title.as_str(), q.alarm), (at(26, 18, 59), "test alarm", true));
        let q = parse("7:30 acordar", now()).unwrap();
        assert_eq!((q.at, q.title.as_str(), q.alarm), (at(27, 7, 30), "acordar", true));
    }

    #[test]
    fn nothing() {
        assert_eq!(parse("buy bread", now()), None);
        assert_eq!(parse("firefox", now()), None);
    }
}
