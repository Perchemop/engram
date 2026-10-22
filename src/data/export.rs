//! CSV import/export for deck exchange, plus whole-library JSON export.
//!
//! CSV format (our own, documented):
//! `Deck,Type,Front,Back,Tags` with RFC-4180 quoting. `Type` is `Basic` or
//! `Cloze`; for Cloze rows the deletion markup lives in `Front` and `Back`
//! mirrors it. Tags are space-separated.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::data::error::DataError;
use crate::domain::card::{CardId, CardKind};
use crate::domain::deck::DeckId;
use crate::domain::library::{CardFilter, Library, Scope};

/// The CSV header row Engram writes.
pub const CSV_HEADER: &str = "Deck,Type,Front,Back,Tags";

/// Quote one RFC-4180 field when needed.
fn csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Parse one CSV record (already unquoted). Used by tests and the importer.
fn parse_csv_record(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                while let Some(c2) = chars.next() {
                    if c2 == '"' {
                        if chars.peek() == Some(&'"') {
                            current.push('"');
                            chars.next();
                        } else {
                            break;
                        }
                    } else {
                        current.push(c2);
                    }
                }
            }
            ',' => {
                fields.push(std::mem::take(&mut current));
            }
            _ => current.push(c),
        }
    }
    fields.push(current);
    fields
}

/// Export the cards of `scope` as CSV. Returns the number of card rows.
///
/// # Errors
/// [`DataError`] on IO failure.
pub fn export_csv(path: &Path, lib: &Library, scope: Scope) -> Result<usize, DataError> {
    let mut rows = vec![CSV_HEADER.to_string()];
    let ids: Vec<CardId> = lib.cards_in_scope(scope).iter().map(|c| c.id).collect();
    let mut count = 0usize;
    for id in ids {
        let card = lib.card(id).expect("id from cards_in_scope exists");
        let deck = lib.deck_display_name(card.deck_id).unwrap_or_default();
        let (kind_label, front, back) = match &card.kind {
            CardKind::Basic { front, back } => ("Basic", front.clone(), back.clone()),
            CardKind::Cloze { text } => ("Cloze", text.clone(), text.clone()),
        };
        rows.push(format!(
            "{},{},{},{},{}",
            csv_field(&deck),
            kind_label,
            csv_field(&front),
            csv_field(&back),
            csv_field(&card.tags.join(" "))
        ));
        count += 1;
    }
    fs::write(path, format!("{}\n", rows.join("\n")))?;
    Ok(count)
}

/// Import cards from a CSV file into `deck_id`. Skips the header row when
/// present. Returns the number of imported cards.
///
/// # Errors
/// [`DataError`] on IO/parse failure; [`crate::domain::LibraryError`]
/// propagated from card creation (invalid rows are skipped, not fatal).
pub fn import_csv(
    path: &Path,
    lib: &mut Library,
    deck_id: DeckId,
    now: DateTime<Utc>,
) -> Result<usize, DataError> {
    let text = fs::read_to_string(path)?;
    let mut imported = 0usize;
    for line in text.lines() {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        let fields = parse_csv_record(line);
        if fields.len() < 3 {
            continue;
        }
        let (kind_label, front, back) = if fields.len() >= 4 {
            (fields[1].trim(), fields[2].trim(), fields[3].trim())
        } else {
            ("Basic", fields[0].trim(), fields[1].trim())
        };
        if front.eq_ignore_ascii_case("front") && kind_label.eq_ignore_ascii_case("type") {
            continue; // header
        }
        let tags: Vec<String> = fields
            .get(4)
            .map(|t| t.split_whitespace().map(str::to_string).collect())
            .unwrap_or_default();
        let kind = if kind_label.eq_ignore_ascii_case("cloze") {
            CardKind::Cloze {
                text: front.to_string(),
            }
        } else {
            CardKind::Basic {
                front: front.to_string(),
                back: back.to_string(),
            }
        };
        // Invalid rows are skipped; a broken line must not abort the import.
        if lib.add_card(deck_id, kind, tags, now).is_ok() {
            imported += 1;
        }
    }
    Ok(imported)
}

/// Export the whole library as pretty JSON (a portable backup).
///
/// # Errors
/// [`DataError`] on failure.
pub fn export_json(path: &Path, lib: &Library) -> Result<(), DataError> {
    fs::write(path, serde_json::to_string_pretty(lib)?).map_err(DataError::from)
}

/// Number of cards matching a plain-text preview of a scope (used by the
/// export dialog).
#[must_use]
pub fn count_scope(lib: &Library, scope: Scope, query: &str) -> usize {
    if query.trim().is_empty() {
        lib.cards_in_scope(scope).len()
    } else {
        lib.search(&CardFilter {
            deck: match scope {
                Scope::All => None,
                Scope::Deck(id) => Some(id),
            },
            tags: vec![],
            query: query.to_string(),
        })
        .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::CardKind;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    fn sample() -> Library {
        let mut l = Library::new("Sample", now());
        let root = l.decks[0].id;
        l.rename_deck(root, "Biology").unwrap();
        l.add_card(
            root,
            CardKind::Basic {
                front: "What is ATP?".into(),
                back: "Energy currency".into(),
            },
            vec!["bio".into(), "chem".into()],
            now(),
        )
        .unwrap();
        l.add_card(
            root,
            CardKind::Cloze {
                text: "The {{c1::hippocampus}} consolidates memory".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
        l.add_card(
            root,
            CardKind::Basic {
                front: "Has, comma".into(),
                back: "Has \"quotes\"".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
        l
    }

    #[test]
    fn csv_roundtrip() {
        let lib = sample();
        let deck = lib.decks[0].id;
        let path = std::env::temp_dir().join(format!("engram_export_{}.csv", uuid::Uuid::new_v4()));
        let n = export_csv(&path, &lib, Scope::All).unwrap();
        assert_eq!(n, 3);

        let mut target = Library::new("Target", now());
        let tdeck = target.decks[0].id;
        let imported = import_csv(&path, &mut target, tdeck, now()).unwrap();
        assert_eq!(imported, 3);
        let cards = target.cards_in_scope(Scope::All);
        assert!(cards
            .iter()
            .any(|c| c.kind.searchable_text().contains("Has, comma")
                && c.kind.searchable_text().contains("Has \"quotes\"")));
        assert!(cards
            .iter()
            .any(|c| matches!(c.kind, CardKind::Cloze { .. })));
        assert!(cards.iter().any(|c| c.tags.contains(&"chem".to_string())));
        let _ = deck;
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn csv_header_is_skipped_on_import() {
        let path = std::env::temp_dir().join(format!("engram_header_{}.csv", uuid::Uuid::new_v4()));
        fs::write(&path, format!("{CSV_HEADER}\nWhat is 2+2?,Basic,4,math\n")).unwrap();
        let mut l = Library::new("T", now());
        let deck = l.decks[0].id;
        let n = import_csv(&path, &mut l, deck, now()).unwrap();
        assert_eq!(n, 1);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn parse_csv_record_handles_quotes() {
        assert_eq!(
            parse_csv_record(r#""a, b","He said ""hi""",c"#),
            vec!["a, b", "He said \"hi\"", "c"]
        );
    }

    #[test]
    fn json_export_roundtrip() {
        let lib = sample();
        let path =
            std::env::temp_dir().join(format!("engram_backup_{}.json", uuid::Uuid::new_v4()));
        export_json(&path, &lib).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        let back: Library = serde_json::from_str(&text).unwrap();
        assert_eq!(back, lib);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn count_scope_matches_query() {
        let lib = sample();
        assert_eq!(count_scope(&lib, Scope::All, ""), 3);
        assert_eq!(count_scope(&lib, Scope::All, "ATP"), 1);
    }
}
