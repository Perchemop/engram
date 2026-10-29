//! Crash reporting (guide §7.1): a panic hook that persists a scrubbed
//! crash report, plus user-consent submission helpers.
//!
//! Design (DECISIONS D-015):
//! - The panic hook never touches the UI and never touches the network.
//!   It writes `crash-latest.txt` into the data directory and logs.
//! - On the next launch the app offers the user a preview; nothing leaves
//!   the machine without explicit consent.
//! - Submission is via a pre-filled GitHub "New issue" URL — **no tokens
//!   are embedded in the client** (guide §7.1 / §2.5).
//! - Paths (the home directory) are scrubbed from backtraces.

use std::path::{Path, PathBuf};

/// GitHub repository used for issue submission.
pub const GITHUB_REPO: &str = "Perchemop/engram";

/// Pre-filled "New issue" URL base (no token; user authenticates in browser).
pub const NEW_ISSUE_URL: &str = "https://github.com/Perchemop/engram/issues/new";

/// App version from the build.
#[must_use]
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// OS/arch description for reports.
#[must_use]
pub fn os_description() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// Replace occurrences of the user's home directory with `~` so reports do
/// not leak absolute paths.
#[must_use]
pub fn scrub_paths(text: &str) -> String {
    let home = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .map(|p| p.to_string_lossy().to_string());
    match home {
        Some(home) if !home.is_empty() => text.replace(&home, "~"),
        _ => text.to_string(),
    }
}

/// Where the latest crash report is stored (`crash-latest.txt` in the data
/// directory). `None` when no data directory is available.
#[must_use]
pub fn crash_file() -> Option<PathBuf> {
    directories::ProjectDirs::from("org", "engram", "Engram")
        .map(|d| d.data_dir().join("crash-latest.txt"))
}

/// Same as [`crash_file`] but rooted at an explicit directory (tests).
#[must_use]
pub fn crash_file_in(dir: &Path) -> PathBuf {
    dir.join("crash-latest.txt")
}

/// Install the global panic hook. Call once at startup, before any UI.
pub fn install() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic payload".to_string());
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());

        let backtrace = std::backtrace::Backtrace::force_capture().to_string();
        let report = format_report(&payload, &location, &backtrace);
        tracing::error!("Panic: {payload} at {location}");

        if let Some(path) = crash_file() {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            // Best effort only: a failing write during a panic must not
            // panic again.
            let _ = std::fs::write(&path, &report);
        }
    }));
}

/// Compose the full report text (scrubbed). The panic message and location
/// are scrubbed too: stable rustc embeds absolute `/home/<user>/.cargo`
/// paths for registry-dependency panics, and the report is submitted
/// voluntarily to a public issue (audit finding
/// crash-report-partial-path-scrub).
#[must_use]
pub fn format_report(message: &str, location: &str, backtrace: &str) -> String {
    let mut out = String::new();
    out.push_str("Engram crashed. The user attached this report voluntarily.\n");
    out.push_str(&format!("App version: {}\n", current_version()));
    out.push_str(&format!("OS: {}\n", os_description()));
    out.push_str(&format!("Panic: {}\n", scrub_paths(message)));
    out.push_str(&format!("Location: {}\n", scrub_paths(location)));
    out.push_str("Backtrace (home paths scrubbed):\n");
    out.push_str(&scrub_paths(backtrace));
    out
}

/// A pending (unsent) crash report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrashReport {
    /// Short, human-friendly issue title.
    pub title: String,
    /// Full report body (scrubbed).
    pub body: String,
}

/// Load a pending crash report, if one exists, rooted at `dir`.
#[must_use]
pub fn pending_report_in(dir: &Path) -> Option<CrashReport> {
    let text = std::fs::read_to_string(crash_file_in(dir)).ok()?;
    Some(to_report(&text))
}

/// Load a pending crash report from the real data directory.
#[must_use]
pub fn pending_report() -> Option<CrashReport> {
    let text = std::fs::read_to_string(crash_file()?).ok()?;
    Some(to_report(&text))
}

fn to_report(text: &str) -> CrashReport {
    let mut title = String::from("Crash on startup");
    let mut body = text.to_string();
    // First "Panic: " line makes a good title.
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Panic: ") {
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                title = format!("Crash: {}", trimmed.chars().take(80).collect::<String>());
            }
            break;
        }
    }
    // Keep issues small: cut at a char boundary so a multi-byte character
    // spanning the offset cannot panic while the app is starting up (audit
    // finding crash-report-truncate-utf8-panic).
    let cut = if body.is_char_boundary(16_000) {
        16_000
    } else {
        body[..16_000]
            .char_indices()
            .map(|(i, _)| i)
            .last()
            .unwrap_or(0)
    };
    body.truncate(cut);
    CrashReport { title, body }
}

/// Delete the pending crash report (user chose "Don't send").
pub fn discard() {
    if let Some(path) = crash_file() {
        let _ = std::fs::remove_file(path);
    }
}

/// Percent-encode a string for use in a URL query value.
#[must_use]
pub fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for b in text.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Build the pre-filled "New issue" URL (labels `crash,bug`; no token).
#[must_use]
pub fn build_issue_url(report: &CrashReport) -> String {
    format!(
        "{NEW_ISSUE_URL}?title={}&body={}&labels=crash,bug",
        percent_encode(&report.title),
        percent_encode(&report.body)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scrub_replaces_home_directory() {
        let home = directories::BaseDirs::new()
            .unwrap()
            .home_dir()
            .to_path_buf();
        let text = format!("at {}/src/main.rs:10", home.display());
        assert_eq!(scrub_paths(&text), "at ~/src/main.rs:10");
        assert_eq!(scrub_paths("no paths here"), "no paths here");
    }

    #[test]
    fn format_report_contains_headers_and_scrubs() {
        let home = directories::BaseDirs::new()
            .unwrap()
            .home_dir()
            .to_path_buf();
        let bt = format!("backtrace frame at {}/library", home.display());
        let report = format_report("bad math", "lib.rs:7", &bt);
        assert!(report.contains(&format!("App version: {}", current_version())));
        assert!(report.contains("OS: "));
        assert!(report.contains("Panic: bad math"));
        assert!(report.contains("Location: lib.rs:7"));
        assert!(report.contains("~/library"));
        assert!(!report.contains(&home.to_string_lossy().to_string()));
    }

    #[test]
    fn percent_encode_is_query_safe() {
        assert_eq!(percent_encode("plain-text_1.2"), "plain-text_1.2");
        assert_eq!(percent_encode("a b&c=d/e"), "a%20b%26c%3Dd%2Fe");
        assert_eq!(percent_encode("multi\nline"), "multi%0Aline");
    }

    #[test]
    fn issue_url_carries_title_body_labels() {
        let report = CrashReport {
            title: "Crash: bad math".into(),
            body: "line one\nline two".into(),
        };
        let url = build_issue_url(&report);
        assert!(url.starts_with(NEW_ISSUE_URL));
        assert!(url.contains("title=Crash%3A%20bad%20math"));
        assert!(url.contains("body=line%20one%0Aline%20two"));
        assert!(url.contains("labels=crash,bug"));
    }

    #[test]
    fn pending_report_roundtrip_and_missing() {
        let dir = std::env::temp_dir().join(format!("engram_crash_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(pending_report_in(&dir).is_none());

        let report = format_report("test panic", "lib.rs:1", "frame");
        std::fs::write(crash_file_in(&dir), &report).unwrap();
        let pending = pending_report_in(&dir).unwrap();
        assert_eq!(pending.title, "Crash: test panic");
        assert!(pending.body.contains("test panic"));
        assert!(pending.body.contains("App version:"));

        // Long payloads are truncated in the body but title stays sane.
        std::fs::write(
            crash_file_in(&dir),
            format_report(&"x".repeat(20_000), "l:1", ""),
        )
        .unwrap();
        let long = pending_report_in(&dir).unwrap();
        assert!(long.body.len() <= 16_000);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn to_report_uses_panic_line_for_title() {
        let text = "App version: 0.1.0\nOS: linux\nPanic: index out of range\nmore";
        let r = to_report(text);
        assert_eq!(r.title, "Crash: index out of range");
    }
}
