//! Update check (guide §7.2): query the GitHub Releases API, compare
//! versions, fail soft offline, cache to avoid API abuse.
//!
//! - Manual check: Help → "Check for updates" (always allowed).
//! - Automatic check: at most once per 24h at startup, result shown
//!   quietly; network errors are silent (fail soft).
//! - No tokens are embedded; the public Releases API needs no auth.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Human-visible releases page.
pub const RELEASES_PAGE_URL: &str = "https://github.com/Perchemop/engram/releases/latest";
/// GitHub REST endpoint for the latest release.
pub const RELEASES_API_URL: &str = "https://api.github.com/repos/Perchemop/engram/releases/latest";
/// GitHub API requires a User-Agent.
pub const USER_AGENT: &str = concat!("engram-update-check/", env!("CARGO_PKG_VERSION"));
/// One automatic check per day at most (no API abuse).
pub const AUTO_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Result of an update check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpdateStatus {
    /// Running the newest release.
    UpToDate,
    /// A newer release exists; `url` links to it.
    Available { latest: String, url: String },
    /// Check could not be completed (offline, API error, unparseable tag).
    Unknown(String),
}

/// The app's own version.
#[must_use]
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Classify a `latest` tag (e.g. `v0.2.0`) against the running version.
#[must_use]
pub fn classify(current: &str, latest_tag: &str) -> UpdateStatus {
    let parse = |v: &str| -> Option<semver::Version> {
        semver::Version::parse(v.trim().trim_start_matches('v')).ok()
    };
    let (Some(cur), Some(latest)) = (parse(current), parse(latest_tag)) else {
        return UpdateStatus::Unknown(format!(
            "could not parse versions: current={current}, latest={latest_tag}"
        ));
    };
    if latest > cur {
        UpdateStatus::Available {
            latest: format!("v{}", latest),
            url: RELEASES_PAGE_URL.to_string(),
        }
    } else {
        UpdateStatus::UpToDate
    }
}

/// Fetch the latest release tag from the GitHub API (network). Fails soft:
/// any error becomes a string for the UI.
pub fn fetch_latest_tag() -> Result<String, String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build()
        .new_agent();
    let mut response = agent
        .get(RELEASES_API_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| format!("request failed: {e}"))?;
    let text = response
        .body_mut()
        .read_to_string()
        .map_err(|e| format!("read failed: {e}"))?;
    parse_tag_name(&text).ok_or_else(|| "response had no tag_name".to_string())
}

/// Extract `tag_name` from a GitHub release JSON payload.
#[must_use]
pub fn parse_tag_name(json: &str) -> Option<String> {
    #[derive(Deserialize)]
    struct Release {
        #[serde(rename = "tag_name")]
        tag_name: String,
    }
    serde_json::from_str::<Release>(json)
        .ok()
        .map(|r| r.tag_name)
}

/// Run a full check: fetch + classify. Errors land in
/// [`UpdateStatus::Unknown`].
#[must_use]
pub fn check_now() -> UpdateStatus {
    match fetch_latest_tag() {
        Ok(tag) => classify(current_version(), &tag),
        Err(e) => UpdateStatus::Unknown(e),
    }
}

/// True when the automatic 24-hour check should run.
#[must_use]
pub fn should_auto_check(last_check_secs: Option<i64>, now_secs: i64) -> bool {
    match last_check_secs {
        None => true,
        Some(last) => now_secs - last >= AUTO_CHECK_INTERVAL.as_secs() as i64,
    }
}

/// Current unix time in seconds (for the cache stamp).
#[must_use]
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_knows_up_to_date_and_available() {
        assert_eq!(classify("0.1.0", "v0.1.0"), UpdateStatus::UpToDate);
        assert_eq!(classify("0.1.0", "0.1.0"), UpdateStatus::UpToDate);
        assert_eq!(
            classify("0.1.0", "v0.2.0"),
            UpdateStatus::Available {
                latest: "v0.2.0".into(),
                url: RELEASES_PAGE_URL.into()
            }
        );
        // Running a newer build than the latest release.
        assert_eq!(classify("0.3.0", "v0.2.0"), UpdateStatus::UpToDate);
    }

    #[test]
    fn classify_fails_soft_on_garbage() {
        assert!(matches!(
            classify("0.1.0", "not-a-version"),
            UpdateStatus::Unknown(_)
        ));
        assert!(matches!(
            classify("dev", "v0.2.0"),
            UpdateStatus::Unknown(_)
        ));
    }

    #[test]
    fn parse_tag_name_reads_github_payload() {
        let json = r#"{"url":"https://api.github.com/x","tag_name":"v0.2.0","name":"0.2.0"}"#;
        assert_eq!(parse_tag_name(json), Some("v0.2.0".to_string()));
        assert_eq!(parse_tag_name("{\"other\":1}"), None);
        assert_eq!(parse_tag_name("not json"), None);
    }

    #[test]
    fn auto_check_respects_the_24h_cache() {
        assert!(should_auto_check(None, 1_000));
        assert!(should_auto_check(Some(1_000), 1_000 + 86_400));
        assert!(!should_auto_check(Some(1_000), 1_000 + 86_399));
        // Clock skew: a "future" stamp also blocks the check.
        assert!(!should_auto_check(Some(5_000), 1_000));
    }

    #[test]
    fn user_agent_carries_version() {
        assert!(USER_AGENT.contains(current_version()));
    }
}
