//! Library persistence: human-readable JSON with atomic writes.

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::data::error::DataError;
use crate::domain::library::Library;
use crate::domain::Theme;

/// The file extension for Engram library files.
pub const LIBRARY_EXTENSION: &str = "engram.json";

/// Ensure a filename carries the `.engram.json` extension.
#[must_use]
pub fn normalise_filename(name: &str) -> String {
    if name.to_lowercase().ends_with(LIBRARY_EXTENSION) {
        name.to_string()
    } else if name.to_lowercase().ends_with(".json") {
        format!("{}.engram.json", &name[..name.len() - 5])
    } else {
        format!("{name}.{LIBRARY_EXTENSION}")
    }
}

/// Per-user app preferences (not part of the library).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    pub theme: Theme,
    /// Unix seconds of the last automatic update check (24h cache, guide
    /// §7.2 "no API abuse").
    pub last_update_check_secs: Option<i64>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            last_update_check_secs: None,
        }
    }
}

fn data_dir() -> Option<PathBuf> {
    directories::ProjectDirs::from("org", "engram", "Engram").map(|d| d.data_dir().to_path_buf())
}

/// Default library file location, e.g.
/// `~/.local/share/engram/Engram/library.engram.json`.
#[must_use]
pub fn default_library_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join(format!("library.{LIBRARY_EXTENSION}")))
}

fn prefs_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("prefs.json"))
}

/// Atomically write text to `path`: write to a sibling temp file, then
/// rename over the target. A crash mid-write can never truncate existing
/// data.
pub(crate) fn atomic_write(path: &std::path::Path, contents: &str) -> Result<(), DataError> {
    let parent = path
        .parent()
        .ok_or_else(|| DataError::Path(format!("no parent directory for {}", path.display())))?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(
        ".{}.tmp",
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "engram".to_string())
    ));
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Serialise the library to a file (pretty-printed, atomic).
///
/// # Errors
/// [`DataError`] on IO or serialisation failure.
pub fn save_library(lib: &Library, path: &std::path::Path) -> Result<(), DataError> {
    let json = serde_json::to_string_pretty(lib)?;
    atomic_write(path, &json)
}

/// Load a library from a file. A missing file yields `Ok(None)`.
///
/// # Errors
/// [`DataError`] on IO or deserialisation failure.
pub fn load_library(path: &std::path::Path) -> Result<Option<Library>, DataError> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path)?;
    Ok(Some(serde_json::from_str(&text)?))
}

/// Move a library file that failed to load aside, preserving the user's
/// bytes, so an autosave can never overwrite them with a fresh empty
/// library (audit finding `data-loss-corrupt-load-overwrite`).
///
/// The quarantined copy keeps its original name plus a
/// `.corrupt-<unix-seconds>` suffix; a `quarantined` marker file is not
/// needed because the suffix is the record. Idempotent under repeated
/// failures only in the sense that each failure renames the file that
/// currently sits at `path`.
///
/// # Errors
/// [`DataError`] when the rename fails — the caller must then refuse to
/// save to that path for the rest of the session.
pub fn quarantine_corrupt_file(path: &std::path::Path) -> Result<std::path::PathBuf, DataError> {
    let stamped = format!(
        "{}.corrupt-{}",
        path.to_string_lossy(),
        chrono::Utc::now().timestamp()
    );
    let target = std::path::PathBuf::from(&stamped);
    fs::rename(path, &target)?;
    Ok(target)
}

/// Save the app preferences.
///
/// # Errors
/// [`DataError`] on failure. Callers usually treat this as non-fatal.
pub fn save_prefs(prefs: &Prefs) -> Result<(), DataError> {
    let Some(path) = prefs_path() else {
        return Err(DataError::Path("no data directory".to_string()));
    };
    atomic_write(&path, &serde_json::to_string_pretty(prefs)?)
}

/// Load the app preferences; missing file yields defaults.
#[must_use]
pub fn load_prefs() -> Prefs {
    let Some(path) = prefs_path() else {
        return Prefs::default();
    };
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::CardKind;

    #[test]
    fn normalise_filename_adds_extension() {
        assert_eq!(normalise_filename("study"), "study.engram.json");
        assert_eq!(normalise_filename("study.json"), "study.engram.json");
        assert_eq!(normalise_filename("a.engram.json"), "a.engram.json");
        assert_eq!(normalise_filename("B.ENGRAM.JSON"), "B.ENGRAM.JSON");
    }

    #[test]
    fn library_roundtrips_through_disk() {
        let dir = std::env::temp_dir().join(format!("engram_test_{}", uuid::Uuid::new_v4()));
        let path = dir.join("library.engram.json");
        let now = chrono::Utc::now();
        let mut lib = Library::new("Roundtrip", now);
        let deck = lib.decks[0].id;
        lib.add_card(
            deck,
            CardKind::Basic {
                front: "f".into(),
                back: "b".into(),
            },
            vec!["t".into()],
            now,
        )
        .unwrap();
        save_library(&lib, &path).unwrap();
        let loaded = load_library(&path).unwrap().unwrap();
        assert_eq!(loaded, lib);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_library_is_none() {
        let path =
            std::env::temp_dir().join(format!("missing_{}.engram.json", uuid::Uuid::new_v4()));
        assert!(load_library(&path).unwrap().is_none());
    }

    #[test]
    fn quarantine_moves_the_failed_file_aside_and_preserves_bytes() {
        let dir = std::env::temp_dir().join(format!("engram_q_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("library.engram.json");
        std::fs::write(&path, "{ not json").unwrap();

        let quarantined = quarantine_corrupt_file(&path).unwrap();
        assert!(!path.exists(), "original path must be vacated");
        assert!(quarantined.exists());
        let preserved = std::fs::read_to_string(&quarantined).unwrap();
        assert_eq!(preserved, "{ not json");
        assert!(
            quarantined.to_string_lossy().contains(".corrupt-"),
            "quarantine name must carry the corrupt marker"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn quarantined_file_no_longer_loads_at_the_original_path() {
        // The full chain the audit flagged: corrupt file at the default path,
        // quarantine, then a fresh library may save to that path safely.
        let dir = std::env::temp_dir().join(format!("engram_q2_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("library.engram.json");
        std::fs::write(&path, "[]").unwrap();
        assert!(load_library(&path).is_err());

        quarantine_corrupt_file(&path).unwrap();
        let fresh = Library::new("My Engram", chrono::Utc::now());
        save_library(&fresh, &path).unwrap();
        let reloaded = load_library(&path).unwrap().unwrap();
        assert_eq!(reloaded.name, "My Engram");
        // The user's original bytes survive under the quarantine name.
        let entries: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
        assert_eq!(entries.len(), 2, "corrupt backup + fresh library");
        std::fs::remove_dir_all(&dir).ok();
    }
}
