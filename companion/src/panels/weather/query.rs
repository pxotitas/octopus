//! What the weather panel's `>` line asks for, in portuguese or english:
//! `> amanhã`, `> ontem`, `> quinta`, `> sexta passada`, `> +3`, `> 28/11`,
//! `> semana`, `> fim de semana`…

use chrono::{Datelike, NaiveDate};

#[derive(Debug, PartialEq)]
pub enum Query {
    /// The current conditions.
    Now,
    /// One day, as days from today.
    Day(i32),
    /// Seven days from `start` (days from today).
    Week(i32),
    /// The next (or last) saturday and sunday.
    Weekend(i32),
    Unknown,
}

const WEEKDAYS: [&[&str]; 7] = [
    &["segunda", "segunda-feira", "seg", "monday", "mon"],
    &["terca", "terca-feira", "ter", "tuesday", "tue", "tues"],
    &["quarta", "quarta-feira", "qua", "wednesday", "wed"],
    &["quinta", "quinta-feira", "qui", "thursday", "thu", "thurs"],
    &["sexta", "sexta-feira", "sex", "friday", "fri"],
    &["sabado", "sab", "saturday", "sat"],
    &["domingo", "dom", "sunday", "sun"],
];

const MONTHS: [&[&str]; 12] = [
    &["janeiro", "jan", "january"],
    &["fevereiro", "fev", "february", "feb"],
    &["marco", "mar", "march"],
    &["abril", "abr", "april", "apr"],
    &["maio", "mai", "may"],
    &["junho", "jun", "june"],
    &["julho", "jul", "july"],
    &["agosto", "ago", "august", "aug"],
    &["setembro", "set", "september", "sep", "sept"],
    &["outubro", "out", "october", "oct"],
    &["novembro", "nov", "november"],
    &["dezembro", "dez", "december", "dec"],
];

const FILLER: &[&str] = &[
    "o", "a", "os", "as", "no", "na", "de", "do", "da", "dia", "em", "pra", "para", "como", "vai",
    "ser", "foi", "tempo", "clima", "the", "on", "of", "this", "how", "is", "was", "will", "be",
    "weather", "essa", "esta", "esse", "este", "que", "vem", "day",
];

const PAST: &[&str] = &["passada", "passado", "ultima", "ultimo", "last", "previous", "anterior"];
const NEXT: &[&str] = &["proxima", "proximo", "next", "seguinte"];

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

pub fn parse(text: &str, today: NaiveDate) -> Query {
    let text = fold(&text.trim().trim_start_matches('>').trim().to_lowercase());
    let words: Vec<&str> = text
        .split(|c: char| c.is_whitespace() || c == ',' || c == '?')
        .filter(|w| !w.is_empty())
        .collect();
    let past = words.iter().any(|w| PAST.contains(w));
    let next = words.iter().any(|w| NEXT.contains(w));
    let words: Vec<&str> = words
        .into_iter()
        .filter(|w| !FILLER.contains(w) && !PAST.contains(w) && !NEXT.contains(w))
        .collect();
    let phrase = words.join(" ");

    match phrase.as_str() {
        "" | "agora" | "now" | "atual" | "current" => return Query::Now,
        "hoje" | "today" => return Query::Day(0),
        "amanha" | "tomorrow" => return Query::Day(1),
        "depois amanha" | "after tomorrow" => return Query::Day(2),
        "ontem" | "yesterday" => return Query::Day(-1),
        "anteontem" | "before yesterday" => return Query::Day(-2),
        "semana" | "week" | "7 dias" | "7 days" | "dias" | "days" => {
            return Query::Week(if past { -7 } else if next { 7 } else { 0 })
        }
        "fim semana" | "fds" | "weekend" | "fim-de-semana" => {
            return Query::Weekend(weekend(today, past))
        }
        _ => {}
    }

    if let Some(offset) = relative(&words) {
        return Query::Day(offset);
    }
    if let Some(day) = weekday(&phrase) {
        let today_index = today.weekday().num_days_from_monday() as i32;
        let ahead = (day - today_index).rem_euclid(7);
        return Query::Day(if past {
            if ahead == 0 { -7 } else { ahead - 7 }
        } else if next && ahead == 0 {
            7
        } else {
            ahead
        });
    }
    if let Some(date) = date(&words, today) {
        return Query::Day((date - today).num_days() as i32);
    }
    Query::Unknown
}

/// `+3`, `-2`, `em 3 dias`, `daqui 3 dias`, `3 dias atras`, `in 3 days`, `3 days ago`.
fn relative(words: &[&str]) -> Option<i32> {
    if let [one] = words {
        if let Some(rest) = one.strip_prefix('+') {
            return rest.parse().ok();
        }
        if one.starts_with('-') {
            return one.parse().ok();
        }
    }
    let number = words.iter().find_map(|w| w.parse::<i32>().ok())?;
    let unit = words.iter().any(|w| matches!(*w, "dia" | "dias" | "day" | "days" | "d"));
    if !unit {
        return None;
    }
    let back = words.iter().any(|w| matches!(*w, "atras" | "ago" | "antes"));
    Some(if back { -number } else { number })
}

fn weekday(phrase: &str) -> Option<i32> {
    let word = phrase.split(' ').next()?;
    WEEKDAYS
        .iter()
        .position(|names| names.contains(&word) || names.contains(&phrase))
        .map(|i| i as i32)
}

/// `28/11`, `28/11/2026`, `2026-11-28`, `28 novembro`, `nov 28`.
fn date(words: &[&str], today: NaiveDate) -> Option<NaiveDate> {
    let joined = words.join(" ");
    if let Ok(d) = NaiveDate::parse_from_str(&joined, "%Y-%m-%d") {
        return Some(d);
    }
    let parts: Vec<&str> = joined.split(['/', '.']).collect();
    if parts.len() >= 2 && parts.iter().all(|p| p.chars().all(|c| c.is_ascii_digit()) && !p.is_empty()) {
        let day: u32 = parts[0].parse().ok()?;
        let month: u32 = parts[1].parse().ok()?;
        let year = match parts.get(2) {
            Some(y) => {
                let y: i32 = y.parse().ok()?;
                if y < 100 { 2000 + y } else { y }
            }
            None => nearest_year(today, month, day),
        };
        return NaiveDate::from_ymd_opt(year, month, day);
    }
    let month = words
        .iter()
        .find_map(|w| MONTHS.iter().position(|names| names.contains(w)))? as u32
        + 1;
    let day: u32 = words.iter().find_map(|w| w.parse().ok())?;
    NaiveDate::from_ymd_opt(nearest_year(today, month, day), month, day)
}

/// A date without a year is the closest one to today, so late december
/// still reaches early january.
fn nearest_year(today: NaiveDate, month: u32, day: u32) -> i32 {
    let year = today.year();
    [year - 1, year, year + 1]
        .into_iter()
        .filter_map(|y| NaiveDate::from_ymd_opt(y, month, day).map(|d| (y, (d - today).num_days().abs())))
        .min_by_key(|(_, distance)| *distance)
        .map_or(year, |(y, _)| y)
}

/// Days from today to the saturday of the next (or last) weekend.
fn weekend(today: NaiveDate, past: bool) -> i32 {
    let index = today.weekday().num_days_from_monday() as i32;
    let ahead = (5 - index).rem_euclid(7);
    if past {
        if ahead == 0 { -7 } else { ahead - 7 }
    } else if index == 6 {
        -1
    } else {
        ahead
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // a wednesday
    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 11, 25).unwrap()
    }

    #[test]
    fn words() {
        let t = today();
        assert_eq!(parse("> amanhã", t), Query::Day(1));
        assert_eq!(parse(">ontem", t), Query::Day(-1));
        assert_eq!(parse("> hoje", t), Query::Day(0));
        assert_eq!(parse(">", t), Query::Now);
        assert_eq!(parse("> depois de amanhã", t), Query::Day(2));
        assert_eq!(parse("> day after tomorrow", t), Query::Day(2));
        assert_eq!(parse("> anteontem", t), Query::Day(-2));
        assert_eq!(parse("> tomorrow", t), Query::Day(1));
        assert_eq!(parse("> como vai ser o tempo amanhã?", t), Query::Day(1));
    }

    #[test]
    fn weekdays() {
        let t = today();
        assert_eq!(parse("> quinta", t), Query::Day(1));
        assert_eq!(parse("> próxima quinta", t), Query::Day(1));
        assert_eq!(parse("> quarta", t), Query::Day(0));
        assert_eq!(parse("> próxima quarta", t), Query::Day(7));
        assert_eq!(parse("> quarta passada", t), Query::Day(-7));
        assert_eq!(parse("> segunda passada", t), Query::Day(-2));
        assert_eq!(parse("> next monday", t), Query::Day(5));
        assert_eq!(parse("> last friday", t), Query::Day(-5));
        assert_eq!(parse("> sábado", t), Query::Day(3));
        assert_eq!(parse("> sexta-feira", t), Query::Day(2));
    }

    #[test]
    fn numbers_and_dates() {
        let t = today();
        assert_eq!(parse("> +3", t), Query::Day(3));
        assert_eq!(parse("> -2", t), Query::Day(-2));
        assert_eq!(parse("> em 4 dias", t), Query::Day(4));
        assert_eq!(parse("> 3 dias atrás", t), Query::Day(-3));
        assert_eq!(parse("> in 5 days", t), Query::Day(5));
        assert_eq!(parse("> 2 days ago", t), Query::Day(-2));
        assert_eq!(parse("> 28/11", t), Query::Day(3));
        assert_eq!(parse("> 2026-12-01", t), Query::Day(6));
        assert_eq!(parse("> 30 de novembro", t), Query::Day(5));
        assert_eq!(parse("> nov 20", t), Query::Day(-5));
    }

    #[test]
    fn ranges() {
        let t = today();
        assert_eq!(parse("> semana", t), Query::Week(0));
        assert_eq!(parse("> semana que vem", t), Query::Week(0));
        assert_eq!(parse("> próxima semana", t), Query::Week(7));
        assert_eq!(parse("> semana passada", t), Query::Week(-7));
        assert_eq!(parse("> fim de semana", t), Query::Weekend(3));
        assert_eq!(parse("> fim de semana passado", t), Query::Weekend(-4));
        assert_eq!(parse("> weekend", t), Query::Weekend(3));
        assert_eq!(parse("> banana", t), Query::Unknown);
    }
}
