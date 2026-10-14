//! File › Home: greeting, template row, search, Recent/Pinned documents (spec:
//! docs/superpowers/specs/2026-10-14-start-page-design.md).

/// A local wall-clock time, plus its day number (days since 1970-01-01) for day arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clock {
    pub y: i64,
    pub m: u32,
    pub d: u32,
    pub hh: u32,
    pub mm: u32,
    pub days: i64,
}

/// Unix seconds shifted by `offset` seconds east of UTC, as a local clock. Saturates instead of
/// overflowing, so any file time is safe.
pub fn clock(unix: i64, offset: i32) -> Clock {
    let t = unix.saturating_add(i64::from(offset));
    let days = t.div_euclid(86_400);
    let secs = t.rem_euclid(86_400);
    let (y, m, d) = wordcraft_engine::cmd::civil_from_days(days);
    Clock { y, m, d, hh: (secs / 3600) as u32, mm: ((secs % 3600) / 60) as u32, days }
}

/// 1970-01-01 (day 0) was a Thursday.
const WEEKDAYS: [&str; 7] = ["Thursday", "Friday", "Saturday", "Sunday", "Monday", "Tuesday", "Wednesday"];

/// "Today, 14:32", "Yesterday", a weekday within the last week, else `YYYY-MM-DD` (also for times
/// in the future); empty when the time is unknown.
pub fn friendly_time(file: Option<Clock>, now: Clock) -> String {
    let Some(f) = file else { return String::new() };
    match now.days.saturating_sub(f.days) {
        0 => format!("{}, {:02}:{:02}", tl!("Today"), f.hh, f.mm),
        1 => tl!("Yesterday").to_string(),
        2..=6 => WEEKDAYS.get(f.days.rem_euclid(7) as usize).map(|w| tl!(w).to_string()).unwrap_or_default(),
        _ => format!("{:04}-{:02}-{:02}", f.y, f.m, f.d),
    }
}

/// The documents whose path (name or folder) contains `query`, ignoring case; all for a blank query.
pub fn filter_docs(query: &str, docs: &[String]) -> Vec<String> {
    let q = query.trim().to_lowercase();
    docs.iter().filter(|p| q.is_empty() || p.to_lowercase().contains(&q)).cloned().collect()
}

/// What kind of file a document is, for its badge colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocKind {
    Word,
    Text,
    Rich,
    Web,
    Pdf,
    Other,
}

pub fn kind_of(path: &str) -> DocKind {
    let ext = std::path::Path::new(path).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    match ext.as_str() {
        "docx" | "doc" | "docm" | "dotx" | "dotm" => DocKind::Word,
        "md" | "markdown" | "txt" => DocKind::Text,
        "rtf" | "odt" | "fodt" => DocKind::Rich,
        "html" | "htm" | "mht" => DocKind::Web,
        "pdf" => DocKind::Pdf,
        _ => DocKind::Other,
    }
}

/// Pin `path` (newest pin first) or unpin it if it is pinned.
pub fn toggle_pin(ui: &mut crate::UiState, path: &str) {
    if ui.pinned.iter().any(|p| p == path) {
        ui.pinned.retain(|p| p != path);
    } else {
        ui.pinned.insert(0, path.to_string());
        ui.pinned.truncate(crate::MAX_RECENT);
    }
}

/// Remove `path` from Recent; a pin stays.
pub fn remove_recent(ui: &mut crate::UiState, path: &str) {
    ui.recent.retain(|p| p != path);
}

/// The Home greeting for a local hour.
pub fn greeting(hour: u32) -> &'static str {
    if hour < 12 {
        "Good morning"
    } else if hour < 18 {
        "Good afternoon"
    } else {
        "Good evening"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Days since 1970-01-01 for a civil date (inverse of `civil_from_days`, Howard Hinnant).
    fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
        let y = if m <= 2 { y - 1 } else { y };
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = y - era * 400;
        let mp = i64::from((m + 9) % 12);
        let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe - 719_468
    }

    fn c(y: i64, m: u32, d: u32, hh: u32, mm: u32) -> Clock {
        Clock { y, m, d, hh, mm, days: days_from_civil(y, m, d) }
    }

    #[test]
    fn friendly_time_today_yesterday_week_older() {
        let now = c(2026, 10, 14, 18, 0);
        assert_eq!(friendly_time(Some(c(2026, 10, 14, 9, 5)), now), "Today, 09:05");
        assert_eq!(friendly_time(Some(c(2026, 10, 13, 23, 59)), now), "Yesterday");
        assert_eq!(friendly_time(Some(c(2026, 10, 10, 8, 0)), now), "Saturday");
        assert_eq!(friendly_time(Some(c(2025, 1, 2, 8, 0)), now), "2025-01-02");
    }

    #[test]
    fn future_and_ancient_times_format_as_dates() {
        let now = c(2026, 10, 14, 18, 0);
        assert_eq!(friendly_time(Some(c(2030, 5, 1, 8, 0)), now), "2030-05-01");
        assert_eq!(friendly_time(Some(clock(-86_400 * 400, 0)), now), "1968-11-27");
        assert!(!friendly_time(Some(clock(i64::MIN, i32::MIN)), now).is_empty());
        assert!(!friendly_time(Some(clock(i64::MAX, i32::MAX)), now).is_empty());
    }

    #[test]
    fn missing_file_has_no_time() {
        assert_eq!(friendly_time(None, c(2026, 10, 14, 18, 0)), "");
    }

    #[test]
    fn clock_applies_offset() {
        let t = clock(1_760_468_400, -3 * 3600); // 2025-10-14 19:00 UTC
        assert_eq!((t.y, t.m, t.d, t.hh, t.mm), (2025, 10, 14, 16, 0));
    }

    #[test]
    fn filter_is_case_insensitive_and_unicode_safe() {
        let docs = vec!["/a/Relatório Final.docx".to_string(), "/b/notes.md".to_string()];
        assert_eq!(filter_docs("RELATÓRIO", &docs), vec![docs[0].clone()]);
        assert_eq!(filter_docs("  ", &docs), docs);
        assert_eq!(filter_docs("/b", &docs), vec![docs[1].clone()]);
        assert!(filter_docs("zzz", &docs).is_empty());
    }

    #[test]
    fn kinds_by_extension() {
        assert_eq!(kind_of("x.DOCX"), DocKind::Word);
        assert_eq!(kind_of("x.md"), DocKind::Text);
        assert_eq!(kind_of("x.odt"), DocKind::Rich);
        assert_eq!(kind_of("x.html"), DocKind::Web);
        assert_eq!(kind_of("x.pdf"), DocKind::Pdf);
        assert_eq!(kind_of("noext"), DocKind::Other);
    }

    #[test]
    fn toggle_pin_adds_newest_first_and_removes() {
        let mut ui = crate::UiState::default();
        toggle_pin(&mut ui, "/a");
        toggle_pin(&mut ui, "/b");
        assert_eq!(ui.pinned, vec!["/b", "/a"]);
        toggle_pin(&mut ui, "/a");
        assert_eq!(ui.pinned, vec!["/b"]);
    }

    #[test]
    fn remove_recent_keeps_pin() {
        let mut ui = crate::UiState::default();
        ui.recent = vec!["/a".into(), "/b".into()];
        toggle_pin(&mut ui, "/a");
        remove_recent(&mut ui, "/a");
        assert_eq!(ui.recent, vec!["/b"]);
        assert_eq!(ui.pinned, vec!["/a"]);
    }

    #[test]
    fn old_prefs_without_pinned_load() {
        let ui: crate::UiState = serde_json::from_str(r#"{"recent":["/a"]}"#).unwrap();
        assert!(ui.pinned.is_empty());
        assert_eq!(ui.recent, vec!["/a"]);
    }

    #[test]
    fn greeting_by_hour() {
        assert_eq!(greeting(5), "Good morning");
        assert_eq!(greeting(12), "Good afternoon");
        assert_eq!(greeting(18), "Good evening");
    }
}
