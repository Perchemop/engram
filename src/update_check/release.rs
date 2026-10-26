//! GitHub release parsing and download selection.
//!
//! Everything here is **untrusted input**: the JSON body, the asset names, the
//! `SHA256SUMS` text and the redirect target. Parsing is defensive — malformed
//! input yields `None`/`Err`, never a panic — and a download URL is only kept
//! when it points inside this repository's own release downloads.
//!
//! Asset names follow the actual release convention of Engram (Rust target
//! triples), for example:
//!
//! ```text
//! engram-0.2.0-x86_64-unknown-linux-gnu
//! engram-0.2.0-aarch64-apple-darwin
//! engram-0.2.0-x86_64-pc-windows-msvc.exe
//! checksums-sha256.txt
//! ```
//!
//! Behaviour is covered by tests that use these real names, so a change in the
//! release workflow that breaks selection fails here rather than in the field.

use serde::{Deserialize, Serialize};

/// The repository whose releases this module understands.
pub const REPO: &str = "Perchemop/engram";

/// File names that carry the checksum list, in the order we prefer them.
const SUMS_NAMES: [&str; 4] = [
    "checksums-sha256.txt",
    "SHA256SUMS.txt",
    "SHA256SUMS",
    "sha256sums.txt",
];

/// One downloadable file of a release.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asset {
    /// Bare file name, as published.
    pub name: String,
    /// Size in bytes; `0` when GitHub did not report one.
    pub size: u64,
    /// `https://github.com/<repo>/releases/download/<tag>/<name>`.
    pub url: String,
    /// Lowercase hex SHA-256, when GitHub's digest or a sums file gave one.
    pub sha256: Option<String>,
}

/// A published release.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub tag: String,
    /// Release title, or the tag when the title is empty.
    pub name: String,
    /// Unix seconds; `None` when unknown.
    pub published_at: Option<i64>,
    pub html_url: String,
    /// Release notes (Markdown), capped.
    pub notes: String,
    pub prerelease: bool,
    pub assets: Vec<Asset>,
}

impl Release {
    /// The version to show a user.
    #[must_use]
    pub fn version_label(&self) -> String {
        self.tag.trim_start_matches('v').to_string()
    }

    /// The checksum-list asset, when the release publishes one.
    #[must_use]
    pub fn sums_asset(&self) -> Option<&Asset> {
        self.assets.iter().find(|a| {
            let lower = a.name.to_ascii_lowercase();
            SUMS_NAMES.iter().any(|n| lower == n.to_ascii_lowercase())
        })
    }

    /// Fills in checksums from a `SHA256SUMS` file for assets that lack one.
    pub fn apply_sums(&mut self, sums: &[(String, String)]) {
        for asset in &mut self.assets {
            if asset.sha256.is_none() {
                asset.sha256 = sums
                    .iter()
                    .find(|(_, name)| name == &asset.name)
                    .map(|(hash, _)| hash.clone());
            }
        }
    }

    /// The asset to download on this machine, if the release ships one.
    #[must_use]
    pub fn asset_for(&self, platform: Platform) -> Option<&Asset> {
        let triples = platform.triples();
        self.assets.iter().find(|a| {
            let stem = a.name.strip_suffix(".exe").unwrap_or(&a.name);
            stem.starts_with("engram-")
                && triples
                    .iter()
                    .any(|t| stem.ends_with(format!("-{t}").as_str()))
        })
    }
}

/// How many bytes of release notes we keep.
const MAX_NOTES: usize = 64 * 1024;

/// The prefix every legitimate download of [`REPO`] starts with.
#[must_use]
pub fn download_prefix(repo: &str) -> String {
    format!("https://github.com/{repo}/releases/download/")
}

fn is_hex64(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Characters a bare file name may never contain.
const RESERVED_NAME_CHARS: [char; 9] = ['/', '\\', ':', '<', '>', '"', '|', '?', '*'];

/// A bare file name we would be willing to save: no paths, no control
/// characters, not a hidden file, not the `..` traversal name.
#[must_use]
pub fn valid_file_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 200
        && name != "."
        && name != ".."
        && !name.starts_with('.')
        && !name
            .chars()
            .any(|c| c.is_control() || RESERVED_NAME_CHARS.contains(&c))
}

/// Whether `url` is safe to hand to the system browser: `https` only, a host,
/// no user-info, no spaces or control characters.
#[must_use]
pub fn safe_web_link(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
    url.len() <= 2000
        && !authority.is_empty()
        && !authority.contains(['@', '\\'])
        && !url.chars().any(|c| c.is_control() || c.is_whitespace())
}

/// Compares a computed digest with the one we expected, both hex.
///
/// Constant-time in the compared length so a partial match cannot be timed.
#[must_use]
pub fn verify_sha256_hex(actual: &str, expected: &str) -> bool {
    let (a, e) = (
        actual.trim().to_ascii_lowercase(),
        expected.trim().to_ascii_lowercase(),
    );
    if !is_hex64(&a) || !is_hex64(&e) {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.bytes().zip(e.bytes()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Reads `GET /repos/<repo>/releases/latest` (or one element of `/releases`).
pub fn parse_release(json: &str, repo: &str) -> Result<Release, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("unreadable answer from GitHub ({e})"))?;
    release_from_value(&value, repo)
}

/// Reads `GET /repos/<repo>/releases` and returns the newest published release,
/// skipping drafts and — unless `allow_prerelease` — prereleases.
///
/// Ordering uses `published_at`, so it does not depend on the tag format.
pub fn parse_release_list(
    json: &str,
    repo: &str,
    allow_prerelease: bool,
) -> Result<Option<Release>, String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("unreadable answer from GitHub ({e})"))?;
    let list = value.as_array().ok_or("unexpected answer from GitHub")?;
    let mut best: Option<Release> = None;
    for item in list {
        if item["draft"].as_bool().unwrap_or(false) {
            continue;
        }
        let Ok(release) = release_from_value(item, repo) else {
            continue;
        };
        if !allow_prerelease && (release.prerelease || is_prerelease_tag(&release.tag)) {
            continue;
        }
        let newer = match (&best, release.published_at) {
            (None, _) => true,
            (Some(b), Some(at)) => b.published_at.is_none_or(|prev| at > prev),
            (Some(b), None) => b.published_at.is_none(),
        };
        if newer {
            best = Some(release);
        }
    }
    Ok(best)
}

/// A tag is a prerelease when its version part carries a suffix (`v0.2.0-rc.1`).
fn is_prerelease_tag(tag: &str) -> bool {
    tag.trim_start_matches('v').contains('-')
}

fn release_from_value(value: &serde_json::Value, repo: &str) -> Result<Release, String> {
    if let Some(message) = value["message"].as_str() {
        return Err(if message == "Not Found" {
            "no releases yet".to_string()
        } else {
            format!("GitHub says: {}", truncate(message, 200))
        });
    }
    let tag = value["tag_name"]
        .as_str()
        .filter(|t| !t.is_empty() && t.len() <= 64)
        .ok_or("no release found")?
        .to_string();
    let releases = format!("https://github.com/{repo}/releases");
    let html_url = value["html_url"]
        .as_str()
        .filter(|u| {
            u.strip_prefix(releases.as_str())
                .is_some_and(|rest| rest.starts_with('/') && !rest.contains(['?', '#', '\\']))
        })
        .map_or_else(|| releases.clone(), str::to_string);
    let prefix = download_prefix(repo);
    let assets = value["assets"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|a| {
                    let name = a["name"]
                        .as_str()
                        .filter(|n| valid_file_name(n))?
                        .to_string();
                    let url = a["browser_download_url"]
                        .as_str()
                        .filter(|u| u.starts_with(&prefix) && !u.contains(['?', '#', '\\']))?
                        .to_string();
                    let sha256 = a["digest"]
                        .as_str()
                        .and_then(|d| d.strip_prefix("sha256:"))
                        .filter(|h| is_hex64(h))
                        .map(str::to_ascii_lowercase);
                    Some(Asset {
                        name,
                        size: a["size"].as_u64().unwrap_or(0),
                        url,
                        sha256,
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Release {
        name: value["name"]
            .as_str()
            .map(|n| truncate(n, 120))
            .filter(|n| !n.is_empty())
            .unwrap_or_else(|| tag.clone()),
        tag,
        published_at: value["published_at"].as_str().and_then(parse_rfc3339),
        html_url,
        notes: value["body"]
            .as_str()
            .map(|b| truncate(b, MAX_NOTES))
            .unwrap_or_default(),
        prerelease: value["prerelease"].as_bool().unwrap_or(false),
        assets,
    })
}

/// The tag in a `https://github.com/<repo>/releases/tag/<tag>` redirect target.
#[must_use]
pub fn tag_from_redirect(location: &str, repo: &str) -> Option<String> {
    let prefix = format!("https://github.com/{repo}/releases/tag/");
    let rest = location.strip_prefix(prefix.as_str())?;
    (!rest.is_empty()
        && rest.len() <= 64
        && rest
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-+".contains(&b)))
    .then(|| rest.to_string())
}

/// Reads `sha256sum` output: `<hex>  <name>` (or `<hex> *<name>`) per line.
#[must_use]
pub fn parse_sha256sums(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let (hash, rest) = line.trim().split_once(char::is_whitespace)?;
            let name = rest.trim_start().trim_start_matches('*');
            (is_hex64(hash) && valid_file_name(name))
                .then(|| (hash.to_ascii_lowercase(), name.to_string()))
        })
        .collect()
}

/// Whether `text` has the exact shape GitHub emits for `published_at`.
fn is_rfc3339_shape(bytes: &[u8]) -> bool {
    bytes.len() >= 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
}

/// Parses `2026-10-25T09:50:06Z` into Unix seconds.
///
/// GitHub always emits this exact shape, so a small strict parser beats a
/// date-time dependency here.
fn parse_rfc3339(text: &str) -> Option<i64> {
    let bytes = text.as_bytes();
    if !is_rfc3339_shape(bytes) {
        return None;
    }
    let num = |range: std::ops::Range<usize>| text.get(range)?.parse::<i64>().ok();
    let (year, month, day) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hour, minute, second) = (num(11..13)?, num(14..16)?, num(17..19)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }
    // Days since the Unix epoch, proleptic Gregorian (Howard Hinnant's algorithm).
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

/// Operating systems the release workflow builds for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Os {
    Windows,
    MacOs,
    Linux,
}

/// Processor architectures the release workflow builds for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Arch {
    X64,
    Arm64,
}

/// The machine the app is running on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Platform {
    pub os: Os,
    pub arch: Arch,
}

impl Platform {
    /// The platform this binary was built for, or `None` for anything the
    /// release workflow does not build.
    #[must_use]
    pub fn current() -> Option<Self> {
        let os = if cfg!(target_os = "windows") {
            Os::Windows
        } else if cfg!(target_os = "macos") {
            Os::MacOs
        } else if cfg!(target_os = "linux") {
            Os::Linux
        } else {
            return None;
        };
        let arch = if cfg!(target_arch = "x86_64") {
            Arch::X64
        } else if cfg!(target_arch = "aarch64") {
            Arch::Arm64
        } else {
            return None;
        };
        Some(Self { os, arch })
    }

    /// Human label for the UI.
    #[must_use]
    pub fn label(self) -> String {
        let os = match self.os {
            Os::Windows => "Windows",
            Os::MacOs => "macOS",
            Os::Linux => "Linux",
        };
        let arch = match self.arch {
            Arch::X64 => "x64",
            Arch::Arm64 => "ARM64",
        };
        format!("{os} {arch}")
    }

    /// Rust target triples this platform can run, native first.
    ///
    /// Apple silicon also runs Intel builds under Rosetta; the reverse is not
    /// true, so an Intel Mac never selects an ARM build.
    fn triples(self) -> &'static [&'static str] {
        match (self.os, self.arch) {
            (Os::Linux, Arch::X64) => &["x86_64-unknown-linux-gnu"],
            (Os::Linux, Arch::Arm64) => &["aarch64-unknown-linux-gnu"],
            (Os::MacOs, Arch::X64) => &["x86_64-apple-darwin"],
            (Os::MacOs, Arch::Arm64) => &["aarch64-apple-darwin", "x86_64-apple-darwin"],
            (Os::Windows, Arch::X64) => &["x86_64-pc-windows-msvc"],
            (Os::Windows, Arch::Arm64) => &["aarch64-pc-windows-msvc"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH: &str = "8a5edab282632443219e051e4ade2d1d5bbc671c781051bf1437897cbdfea0f1";

    /// The real asset list of Engram v0.2.0.
    fn release_json() -> String {
        let prefix = download_prefix(REPO);
        let tag = "v0.2.0";
        let mut assets = Vec::new();
        for name in [
            "engram-0.2.0-aarch64-apple-darwin",
            "engram-0.2.0-aarch64-unknown-linux-gnu",
            "engram-0.2.0-x86_64-pc-windows-msvc.exe",
            "engram-0.2.0-x86_64-unknown-linux-gnu",
            "checksums-sha256.txt",
        ] {
            assets.push(format!(
                r#"{{"name":"{name}","size":42,"browser_download_url":"{prefix}{tag}/{name}","digest":"sha256:{HASH}"}}"#
            ));
        }
        format!(
            r#"{{"tag_name":"{tag}","name":"Engram v0.2.0","html_url":"https://github.com/{REPO}/releases/tag/{tag}",
            "published_at":"2026-10-25T09:50:06Z","prerelease":false,"body":"Notes","assets":[{}]}}"#,
            assets.join(",")
        )
    }

    fn platform(os: Os, arch: Arch) -> Platform {
        Platform { os, arch }
    }

    #[test]
    fn reads_the_engram_release() {
        let release = parse_release(&release_json(), REPO).expect("fixture must parse");
        assert_eq!(release.tag, "v0.2.0");
        assert_eq!(release.version_label(), "0.2.0");
        assert_eq!(release.published_at, Some(1_792_921_806));
        assert_eq!(release.assets.len(), 5);
        // GitHub's `digest` supplies the checksum without a sums file.
        assert!(release
            .assets
            .iter()
            .all(|a| a.sha256.as_deref() == Some(HASH)));
    }

    #[test]
    fn picks_the_asset_for_each_supported_platform() {
        let release = parse_release(&release_json(), REPO).expect("fixture must parse");
        let pick = |os, arch| {
            release
                .asset_for(platform(os, arch))
                .map(|a| a.name.clone())
        };
        assert_eq!(
            pick(Os::Linux, Arch::X64).as_deref(),
            Some("engram-0.2.0-x86_64-unknown-linux-gnu")
        );
        assert_eq!(
            pick(Os::Linux, Arch::Arm64).as_deref(),
            Some("engram-0.2.0-aarch64-unknown-linux-gnu")
        );
        assert_eq!(
            pick(Os::Windows, Arch::X64).as_deref(),
            Some("engram-0.2.0-x86_64-pc-windows-msvc.exe")
        );
        assert_eq!(
            pick(Os::MacOs, Arch::Arm64).as_deref(),
            Some("engram-0.2.0-aarch64-apple-darwin")
        );
        // Intel macOS must never select an ARM-only build: it cannot run one.
        assert_eq!(pick(Os::MacOs, Arch::X64), None);
    }

    #[test]
    fn apple_silicon_falls_back_to_the_intel_build() {
        let prefix = download_prefix(REPO);
        let name = "engram-0.2.0-x86_64-apple-darwin";
        let json = format!(
            r#"{{"tag_name":"v0.2.0","assets":[{{"name":"{name}","size":1,"browser_download_url":"{prefix}v0.2.0/{name}"}}]}}"#
        );
        let release = parse_release(&json, REPO).expect("fixture must parse");
        assert_eq!(
            release
                .asset_for(platform(Os::MacOs, Arch::Arm64))
                .map(|a| a.name.as_str()),
            Some(name)
        );
    }

    #[test]
    fn only_engram_prefixed_assets_match() {
        let prefix = download_prefix(REPO);
        let name = "other-1.0.0-x86_64-unknown-linux-gnu";
        let json = format!(
            r#"{{"tag_name":"v1.0.0","assets":[{{"name":"{name}","size":1,"browser_download_url":"{prefix}v1.0.0/{name}"}}]}}"#
        );
        let release = parse_release(&json, REPO).expect("fixture must parse");
        assert!(release.asset_for(platform(Os::Linux, Arch::X64)).is_none());
    }

    #[test]
    fn drops_downloads_that_point_elsewhere() {
        for url in [
            "https://evil.example/engram",
            "https://github.com/Perchemop/other/releases/download/v1/engram-1.0.0-x86_64-unknown-linux-gnu",
            "javascript:alert(1)",
        ] {
            let json = format!(
                r#"{{"tag_name":"v1.0.0","assets":[{{"name":"engram-1.0.0-x86_64-unknown-linux-gnu","size":1,"browser_download_url":"{url}"}}]}}"#
            );
            let release = parse_release(&json, REPO).ok();
            assert!(release.is_some_and(|r| r.assets.is_empty()));
        }
        assert_eq!(
            parse_release(r#"{"message":"Not Found"}"#, REPO).err().as_deref(),
            Some("no releases yet")
        );
        assert!(parse_release("<html>", REPO).is_err());
    }

    #[test]
    fn list_picks_the_newest_stable_release() {
        let json = r#"[
            {"tag_name":"v0.3.0-rc.1","prerelease":false,"draft":false,"published_at":"2026-10-24T00:00:00Z","assets":[]},
            {"tag_name":"v0.2.1","prerelease":true,"draft":false,"published_at":"2026-10-23T00:00:00Z","assets":[]},
            {"tag_name":"v0.4.0","draft":true,"published_at":"2026-10-25T00:00:00Z","assets":[]},
            {"tag_name":"v0.2.0","prerelease":false,"draft":false,"published_at":"2026-10-22T00:00:00Z","assets":[]}
        ]"#;
        assert_eq!(
            parse_release_list(json, REPO, false).ok().flatten().map(|r| r.tag).as_deref(),
            Some("v0.2.0")
        );
        assert_eq!(
            parse_release_list(json, REPO, true).ok().flatten().map(|r| r.tag).as_deref(),
            Some("v0.3.0-rc.1")
        );
        assert_eq!(parse_release_list("[]", REPO, false).ok().flatten(), None);
    }

    #[test]
    fn reads_checksums_and_redirects() {
        let sums = parse_sha256sums(&format!(
            "{HASH}  engram-0.2.0-x86_64-unknown-linux-gnu\n{HASH} *b.zip\nnot a line\n{HASH}  ../escape\n"
        ));
        assert_eq!(
            sums,
            [
                (HASH.to_string(), "engram-0.2.0-x86_64-unknown-linux-gnu".to_string()),
                (HASH.to_string(), "b.zip".to_string())
            ]
        );
        let location = "https://github.com/Perchemop/engram/releases/tag/v0.2.0";
        assert_eq!(tag_from_redirect(location, REPO).as_deref(), Some("v0.2.0"));
        assert_eq!(tag_from_redirect("https://github.com/Perchemop/engram/releases", REPO), None);
        assert_eq!(tag_from_redirect("https://evil/Perchemop/engram/releases/tag/v1", REPO), None);
    }

    #[test]
    fn a_sums_file_fills_in_missing_digests() {
        let prefix = download_prefix(REPO);
        let name = "engram-1.0.0-x86_64-unknown-linux-gnu";
        let json = format!(
            r#"{{"tag_name":"v1.0.0","assets":[
                {{"name":"{name}","size":1,"browser_download_url":"{prefix}v1.0.0/{name}"}},
                {{"name":"checksums-sha256.txt","size":1,"browser_download_url":"{prefix}v1.0.0/checksums-sha256.txt"}}]}}"#
        );
        let mut release = parse_release(&json, REPO).expect("fixture must parse");
        assert!(release.assets[0].sha256.is_none());
        assert_eq!(release.sums_asset().map(|a| a.name.as_str()), Some("checksums-sha256.txt"));
        release.apply_sums(&[(HASH.to_string(), name.to_string())]);
        assert_eq!(release.assets[0].sha256.as_deref(), Some(HASH));
    }

    #[test]
    fn verify_sha256_compares_hex_digests() {
        assert!(verify_sha256_hex(HASH, HASH));
        assert!(verify_sha256_hex(&HASH.to_ascii_uppercase(), HASH));
        assert!(!verify_sha256_hex(HASH, &format!("{HASH}0")));
        assert!(!verify_sha256_hex("", HASH));
        assert!(!verify_sha256_hex("zz", "zz"));
    }

    #[test]
    fn only_plain_https_links_pass() {
        assert!(safe_web_link("https://github.com/Perchemop/engram/releases/tag/v1"));
        for bad in [
            "http://example.com/",
            "javascript:alert(1)",
            "file:///etc/passwd",
            "https://",
            "https://user@evil.example/",
            "https://example.com/a b",
            "https://example.com/\nx",
        ] {
            assert!(!safe_web_link(bad), "{bad:?}");
        }
    }

    #[test]
    fn file_names_cannot_escape() {
        for bad in ["", "..", "a/b", "a\\b", ".hidden", "c:x", "x\u{0}"] {
            assert!(!valid_file_name(bad), "{bad:?}");
        }
        assert!(valid_file_name("engram-0.2.0-x86_64-unknown-linux-gnu"));
    }
}
