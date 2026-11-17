use unicode_width::UnicodeWidthChar;
use unicode_width::UnicodeWidthStr;

pub fn width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub fn truncate(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if width(text) <= max {
        return text.to_owned();
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_width > max.saturating_sub(1) {
            break;
        }
        out.push(ch);
        used += ch_width;
    }
    out.push('…');
    out
}

pub fn basename(path: &str) -> &str {
    path.trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
}

/// The executable name from a stringified command line.
pub fn command_name(command: &str) -> Option<String> {
    let first = command.split_whitespace().next()?;
    let name = basename(first);
    if name.is_empty() {
        None
    } else {
        Some(name.to_owned())
    }
}

pub fn is_shell(name: &str) -> bool {
    matches!(
        name,
        "sh" | "bash" | "zsh" | "fish" | "nu" | "nushell" | "dash" | "ksh" | "tcsh" | "csh"
    )
}

/// Prefer a readable `~/…` form and keep the tail of long paths.
pub fn shorten_path(path: &str, max: usize, home: Option<&str>) -> String {
    if max == 0 {
        return String::new();
    }
    let mut display = path.to_owned();
    if let Some(home) = home {
        let home = home.trim_end_matches('/');
        if !home.is_empty() && (path == home || path.starts_with(&format!("{}/", home))) {
            display = format!("~{}", &path[home.len()..]);
        }
    }
    if width(&display) <= max {
        return display;
    }
    let tail: Vec<&str> = display
        .rsplit('/')
        .filter(|part| !part.is_empty())
        .collect();
    let mut kept: Vec<&str> = Vec::new();
    let mut used = 2; // room for "…/"
    for part in tail {
        let part_width = width(part);
        let extra = if kept.is_empty() {
            part_width
        } else {
            part_width + 1
        };
        if used + extra > max {
            break;
        }
        used += extra;
        kept.push(part);
    }
    if kept.is_empty() {
        return truncate(&display, max);
    }
    kept.reverse();
    format!("…/{}", kept.join("/"))
}

pub fn expand_home(path: &str, home: Option<&str>) -> String {
    let path = path.trim();
    if path == "~" {
        return home.unwrap_or(path).to_owned();
    }
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = home {
            return format!("{}/{}", home.trim_end_matches('/'), rest);
        }
    }
    path.to_owned()
}

pub fn format_age(ms: u64) -> String {
    let secs = ms / 1000;
    if secs < 5 {
        "now".to_owned()
    } else if secs < 60 {
        format!("{}s", secs)
    } else if secs < 60 * 60 {
        format!("{}m", secs / 60)
    } else if secs < 24 * 60 * 60 {
        format!("{}h", secs / (60 * 60))
    } else {
        format!("{}d", secs / (24 * 60 * 60))
    }
}

pub fn format_age_of(duration: std::time::Duration) -> String {
    format_age(duration.as_millis().min(u64::MAX as u128) as u64)
}

pub fn format_ago(duration: std::time::Duration) -> String {
    let age = format_age_of(duration);
    if age == "now" {
        "just now".to_owned()
    } else {
        format!("{} ago", age)
    }
}

pub fn unique_name(base: &str, existing: impl IntoIterator<Item = String>) -> String {
    let base = base.trim();
    let base = if base.is_empty() { "space" } else { base };
    let existing: std::collections::BTreeSet<String> = existing.into_iter().collect();
    if !existing.contains(base) {
        return base.to_owned();
    }
    for suffix in 2..1000 {
        let candidate = format!("{}-{}", base, suffix);
        if !existing.contains(&candidate) {
            return candidate;
        }
    }
    format!("{}-{}", base, std::process::id())
}

pub fn sanitize_space_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("A space needs a name.".to_owned());
    }
    if name.contains('/') {
        return Err("Space names cannot contain '/'.".to_owned());
    }
    if name.chars().any(|ch| ch.is_control()) {
        return Err("Space names cannot contain control characters.".to_owned());
    }
    Ok(name.to_owned())
}

pub fn normalize_path(path: &str, cwd: Option<&str>, home: Option<&str>) -> Option<String> {
    let path = path.trim();
    if path.is_empty() {
        return None;
    }
    let expanded = expand_home(path, home);
    if expanded.starts_with('/') {
        return Some(expanded);
    }
    cwd.map(|cwd| format!("{}/{}", cwd.trim_end_matches('/'), expanded))
}

pub fn wrap(text: &str, max: usize) -> Vec<String> {
    if max == 0 {
        return vec![];
    }
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let word_width = width(word);
        let needed = if line.is_empty() {
            word_width
        } else {
            width(&line) + 1 + word_width
        };
        if needed > max && !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        if word_width > max {
            line.push_str(&truncate(word, max));
        } else {
            line.push_str(word);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncation_respects_display_width_and_marks_the_cut() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello", 4), "hel…");
        assert_eq!(truncate("日本語", 4), "日…");
        assert!(width(&truncate("日本語", 4)) <= 4);
    }

    #[test]
    fn long_paths_keep_their_tail() {
        let path = "/home/me/dev/some/really/long/project";
        assert_eq!(
            shorten_path(path, 80, Some("/home/me")),
            "~/dev/some/really/long/project"
        );
        assert_eq!(shorten_path(path, 16, Some("/home/me")), "…/long/project");
        assert!(width(&shorten_path(path, 12, Some("/home/me"))) <= 12);
    }

    #[test]
    fn paths_expand_only_a_leading_home_component() {
        assert_eq!(expand_home("~/dev", Some("/home/me")), "/home/me/dev");
        assert_eq!(expand_home("~", Some("/home/me")), "/home/me");
        assert_eq!(expand_home("~/dev", None), "~/dev");
        assert_eq!(expand_home("a/~/b", Some("/home/me")), "a/~/b");
    }

    #[test]
    fn ages_are_compact_and_ordered() {
        assert_eq!(format_age(0), "now");
        assert_eq!(format_age(12_000), "12s");
        assert_eq!(format_age(5 * 60_000), "5m");
        assert_eq!(format_age(3 * 60 * 60_000), "3h");
        assert_eq!(format_age(2 * 24 * 60 * 60_000), "2d");
    }

    #[test]
    fn ages_read_naturally_in_sentences() {
        assert_eq!(format_ago(std::time::Duration::from_secs(1)), "just now");
        assert_eq!(format_ago(std::time::Duration::from_secs(120)), "2m ago");
    }

    #[test]
    fn names_are_validated_and_made_unique() {
        assert_eq!(sanitize_space_name(" dots "), Ok("dots".to_owned()));
        assert!(sanitize_space_name("").is_err());
        assert!(sanitize_space_name("a/b").is_err());
        assert_eq!(unique_name("dots", ["dots".to_owned()]), "dots-2");
        assert_eq!(
            unique_name("dots", ["dots".to_owned(), "dots-2".to_owned()]),
            "dots-3"
        );
    }

    #[test]
    fn wrapping_keeps_words_whole() {
        assert_eq!(wrap("one two three", 7), vec!["one two", "three"]);
        assert_eq!(wrap("supercalifragilistic", 6), vec!["super…"]);
    }

    #[test]
    fn command_names_are_the_executable() {
        assert_eq!(command_name("opencode --auto"), Some("opencode".to_owned()));
        assert_eq!(command_name("/usr/bin/pi"), Some("pi".to_owned()));
        assert!(is_shell("zsh"));
        assert!(!is_shell("opencode"));
    }
}
