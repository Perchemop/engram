//! Delimited text (TAB/CSV) import with column mapping — Anki-style text
//! interchange.
//!
//! One line of the file is one card. The separator is auto-detected (TAB
//! preferred, then comma) and RFC-4180 quoting is honoured, so a quoted
//! field may contain the other separator, quotes (doubled) and embedded
//! newlines. An [`ImportMapping`] says which columns hold the front, back,
//! tags and (optionally) a `Parent::Child` deck.
//!
//! The importer is deliberately forgiving: rows that fail validation or
//! duplicate an existing front are counted and skipped, never fatal. See
//! `docs/research/anki-import.md` for the agreed behaviour.

use std::fs;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::data::error::DataError;
use crate::data::export::delimited_records;
use crate::domain::card::{count_cloze_groups, CardKind};
use crate::domain::deck::DeckId;
use crate::domain::library::Library;

/// Error message used when the import file is not valid UTF-8 text.
const NOT_UTF8: &str = "file is not UTF-8 text";

/// Upper bound on decks a single text import may create (audit finding
/// deck-explosion-frame-freeze: uncapped creation made the app durably
/// unusable via the per-frame deck summaries).
pub const MAX_IMPORT_DECKS: usize = 500;

/// Upper bound on `Parent::Child` levels per imported deck path.
pub const MAX_IMPORT_DECK_DEPTH: usize = 5;

/// Which columns of the imported file map to which card field.
///
/// The default matches Engram's own layout: `front`, `back`, `tags`, in
/// that order, with no deck column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportMapping {
    /// Zero-based column holding the card front (or the cloze text).
    pub front: usize,
    /// Zero-based column holding the card back.
    pub back: usize,
    /// Optional zero-based column holding tags (comma- or space-separated).
    #[serde(default)]
    pub tags: Option<usize>,
    /// Optional zero-based column holding a `Parent::Child` deck path,
    /// created on demand.
    #[serde(default)]
    pub deck: Option<usize>,
}

impl Default for ImportMapping {
    fn default() -> Self {
        Self {
            front: 0,
            back: 1,
            tags: Some(2),
            deck: None,
        }
    }
}

/// The outcome of an import: how many rows became cards and why the rest
/// did not.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportReport {
    /// Rows that passed validation and were added.
    pub imported: usize,
    /// Rows skipped because the same front already existed in the deck.
    pub skipped_duplicates: usize,
    /// Rows skipped because they were empty or failed card validation.
    pub skipped_invalid: usize,
}

impl ImportReport {
    /// Human-readable summary, e.g. `imported 120, skipped 8 duplicate(s)`.
    #[must_use]
    pub fn summary(&self) -> String {
        let imported = self.imported;
        let duplicates = self.skipped_duplicates;
        let invalid = self.skipped_invalid;
        let mut out = format!("imported {imported}");
        if duplicates > 0 {
            out.push_str(&format!(", skipped {duplicates} duplicate(s)"));
        }
        if invalid > 0 {
            out.push_str(&format!(", skipped {invalid} invalid"));
        }
        out
    }
}

/// Import cards from a delimited UTF-8 text file into `deck_id`.
///
/// The file is read once and handed to [`import_text_from_str`].
///
/// # Errors
/// [`DataError::Io`] when the file cannot be read, or [`DataError::Csv`]
/// when it is not valid UTF-8. Per-row problems never abort the import.
pub fn import_text(
    path: &Path,
    lib: &mut Library,
    deck_id: DeckId,
    mapping: &ImportMapping,
    now: DateTime<Utc>,
) -> Result<ImportReport, DataError> {
    let bytes = fs::read(path)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| DataError::Csv(NOT_UTF8.to_string()))?;
    import_text_from_str(text, lib, deck_id, mapping, now)
}

/// Import cards from an in-memory delimited text into `deck_id`.
///
/// The separator is auto-detected (see [`detect_separator`]). The first
/// non-empty row is treated as a header — and skipped — when its first
/// field starts with `Front` or `Question` (case-insensitive). A row whose
/// front contains a well-formed `{{cN::…}}` deletion becomes a Cloze card
/// regardless of the chosen type.
///
/// # Errors
/// [`DataError::Library`] only when a deck path named in the mapping
/// cannot be created; content problems are counted in the returned report
/// instead of failing the import.
pub fn import_text_from_str(
    text: &str,
    lib: &mut Library,
    deck_id: DeckId,
    mapping: &ImportMapping,
    now: DateTime<Utc>,
) -> Result<ImportReport, DataError> {
    let separator = detect_separator(text);
    let records = delimited_records(text, separator);
    let mut report = ImportReport::default();
    let mut first_content = true;
    // Decks created by this import, for the per-import cap (audit finding
    // deck-explosion-frame-freeze: uncapped deck creation made the app
    // durably unusable).
    let mut created_decks = 0usize;

    for fields in &records {
        if fields.iter().all(|f| f.trim().is_empty()) {
            continue; // blank line / trailing newline: no phantom rows
        }
        if first_content {
            first_content = false;
            if is_header_row(fields) {
                continue;
            }
        }

        let front = sanitize_html(field(fields, mapping.front))
            .trim()
            .to_string();
        let back = sanitize_html(field(fields, mapping.back))
            .trim()
            .to_string();
        if front.is_empty() || back.is_empty() {
            report.skipped_invalid += 1;
            continue; // would fail Library::add_card validation
        }

        let deck_spec = mapping
            .deck
            .and_then(|idx| fields.get(idx))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());

        // Duplicate front in the target deck? A deck that does not exist
        // yet cannot hold duplicates, so only check when it resolves.
        let dup_target = match deck_spec {
            Some(spec) => find_deck_path(lib, spec),
            None => Some(deck_id),
        };
        if let Some(target) = dup_target {
            if front_exists_in(lib, target, &front) {
                report.skipped_duplicates += 1;
                continue;
            }
        }

        let target = match deck_spec {
            Some(spec) => match resolve_deck_path(lib, spec, now, &mut created_decks) {
                Ok(id) => id,
                // Deck creation failed (cap reached, depth cap, sibling-name
                // conflict): the row is counted and skipped, never fatal —
                // matching the module's documented invariant. The deck-DoS
                // caps are audit finding deck-explosion-frame-freeze.
                Err(_) => {
                    report.skipped_invalid += 1;
                    continue;
                }
            },
            None => deck_id,
        };
        let kind = build_kind(front, back);
        let tags = mapping
            .tags
            .and_then(|idx| fields.get(idx))
            .map(|raw| split_tags(raw.as_str()))
            .unwrap_or_default();

        match lib.add_card(target, kind, tags, now) {
            Ok(_) => report.imported += 1,
            Err(_) => report.skipped_invalid += 1,
        }
    }
    Ok(report)
}

/// Pick the field separator: TAB when the first non-empty line contains
/// one, comma otherwise.
#[must_use]
pub fn detect_separator(text: &str) -> char {
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        return if line.contains('\t') { '\t' } else { ',' };
    }
    ','
}

/// Convert `<br>` (any casing, self-closing or not) to a newline and drop
/// `<b>`/`<i>` tags while keeping their text. Any other angle-bracket text
/// is left verbatim: Engram V1 cards are plain text.
#[must_use]
pub fn sanitize_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(lt) = rest.find('<') {
        out.push_str(&rest[..lt]);
        let after = &rest[lt + 1..];
        let Some(gt) = after.find('>') else {
            out.push_str(&rest[lt..]);
            return out;
        };
        let tag = &after[..gt];
        let looks_like_tag =
            tag.starts_with('/') || tag.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
        if !looks_like_tag {
            // Not a tag (e.g. `a < b > c`): keep the raw text.
            out.push_str(&rest[lt..lt + gt + 2]);
            rest = &rest[lt + gt + 2..];
            continue;
        }
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        match name.to_ascii_lowercase().as_str() {
            "br" => out.push('\n'),
            "b" | "i" => {} // drop the tag, keep the inner text
            _ => out.push_str(&rest[lt..lt + gt + 2]),
        }
        rest = &rest[lt + gt + 2..];
    }
    out.push_str(rest);
    out
}

/// The row's field at `index`, or `""` when the row is shorter.
fn field(fields: &[String], index: usize) -> &str {
    fields.get(index).map_or("", String::as_str)
}

/// A header row is the first content row when its first field starts with
/// `Front`/`Question` (case-insensitive).
fn is_header_row(fields: &[String]) -> bool {
    let first = fields
        .first()
        .map_or("", String::as_str)
        .trim()
        .to_lowercase();
    first.starts_with("front") || first.starts_with("question")
}

/// The front text a card is recognised by: Basic uses its front, Cloze its
/// deletion text.
fn card_front(kind: &CardKind) -> &str {
    match kind {
        CardKind::Basic { front, .. } => front,
        CardKind::Cloze { text } => text,
    }
}

/// Case-insensitive, whitespace-trimmed key for duplicate detection.
fn duplicate_key(front: &str) -> String {
    front.trim().to_lowercase()
}

/// Whether `deck` already holds a card whose normalised front matches.
fn front_exists_in(lib: &Library, deck: DeckId, front: &str) -> bool {
    let key = duplicate_key(front);
    for card in &lib.cards {
        if card.deck_id == deck && duplicate_key(card_front(&card.kind)) == key {
            return true;
        }
    }
    false
}

/// A cloze front becomes a Cloze card (text unchanged); anything else is a
/// Basic card.
fn build_kind(front: String, back: String) -> CardKind {
    if count_cloze_groups(&front) > 0 {
        CardKind::Cloze { text: front }
    } else {
        CardKind::Basic { front, back }
    }
}

/// Split a tags field on commas and whitespace: Anki separates tags with
/// spaces, while Engram's own CSV uses commas.
fn split_tags(raw: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for part in raw.split(|c: char| c == ',' || c.is_whitespace()) {
        if !part.is_empty() {
            tags.push(part.to_string());
        }
    }
    tags
}

/// Split a `Parent::Child` deck spec into trimmed, non-empty segments.
fn deck_segments(spec: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    for part in spec.split("::") {
        let part = part.trim();
        if !part.is_empty() {
            segments.push(part);
        }
    }
    segments
}

/// Resolve an existing `Parent::Child` deck path; `None` when any segment
/// is missing.
fn find_deck_path(lib: &Library, spec: &str) -> Option<DeckId> {
    let mut parent: Option<DeckId> = None;
    for segment in deck_segments(spec) {
        let id = lib
            .decks
            .iter()
            .find(|d| d.parent == parent && d.name.eq_ignore_ascii_case(segment))
            .map(|d| d.id)?;
        parent = Some(id);
    }
    parent
}

/// Resolve a `Parent::Child` deck path, creating missing decks on demand
/// with the same sibling-name uniqueness rules as manual creation.
///
/// Capped against deck-explosion DoS (audit finding
/// deck-explosion-frame-freeze): at most [`MAX_IMPORT_DECKS`] new decks per
/// import and at most [`MAX_IMPORT_DECK_DEPTH`] `::` levels per path. The
/// caller counts a capped row as skipped, never fatal.
fn resolve_deck_path(
    lib: &mut Library,
    spec: &str,
    now: DateTime<Utc>,
    created: &mut usize,
) -> Result<DeckId, DataError> {
    let segments = deck_segments(spec);
    if segments.len() > MAX_IMPORT_DECK_DEPTH {
        return Err(DataError::Csv(format!(
            "deck path deeper than {MAX_IMPORT_DECK_DEPTH} levels"
        )));
    }
    let mut parent: Option<DeckId> = None;
    for segment in segments {
        let existing = lib
            .decks
            .iter()
            .find(|d| d.parent == parent && d.name.eq_ignore_ascii_case(segment))
            .map(|d| d.id);
        let id = match existing {
            Some(id) => id,
            None => {
                if *created >= MAX_IMPORT_DECKS {
                    return Err(DataError::Csv(format!(
                        "import would create more than {MAX_IMPORT_DECKS} decks"
                    )));
                }
                *created += 1;
                lib.new_deck(parent, segment, None, now)?
            }
        };
        parent = Some(id);
    }
    parent.ok_or_else(|| DataError::Csv("empty deck path".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::library::Scope;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    /// An empty library plus the id of its single Default deck.
    fn library() -> (Library, DeckId) {
        let lib = Library::new("Import test", now());
        let deck = lib.decks[0].id;
        (lib, deck)
    }

    /// Import `text` and unwrap, keeping the test bodies terse.
    fn run(text: &str, lib: &mut Library, deck: DeckId, mapping: &ImportMapping) -> ImportReport {
        import_text_from_str(text, lib, deck, mapping, now()).unwrap()
    }

    #[test]
    fn default_mapping_is_front_back_tags() {
        let m = ImportMapping::default();
        assert_eq!((m.front, m.back, m.tags, m.deck), (0, 1, Some(2), None));
    }

    #[test]
    fn tab_separated_import_applies_tags() {
        let (mut lib, deck) = library();
        let text = "What is ATP?\tEnergy currency\tbio chem\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        assert_eq!(report.skipped_duplicates, 0);
        assert_eq!(report.skipped_invalid, 0);
        let card = &lib.cards[0];
        match &card.kind {
            CardKind::Basic { front, back } => {
                assert_eq!(front, "What is ATP?");
                assert_eq!(back, "Energy currency");
            }
            other => panic!("expected basic, got {other:?}"),
        }
        assert_eq!(card.tags, vec!["bio", "chem"]);
        assert_eq!(card.deck_id, deck);
    }

    #[test]
    fn tab_file_keeps_commas_inside_fields() {
        let (mut lib, deck) = library();
        // TAB wins detection, so the commas stay part of the fields.
        let text = "Paris, France\tCapital, of France\ttravel\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        match &lib.cards[0].kind {
            CardKind::Basic { front, back } => {
                assert_eq!(front, "Paris, France");
                assert_eq!(back, "Capital, of France");
            }
            other => panic!("expected basic, got {other:?}"),
        }
        assert_eq!(lib.cards[0].tags, vec!["travel"]);
    }

    #[test]
    fn comma_separated_quoted_fields_keep_commas() {
        let (mut lib, deck) = library();
        let text = "\"Paris, France\",\"Capital, of France\"\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        match &lib.cards[0].kind {
            CardKind::Basic { front, back } => {
                assert_eq!(front, "Paris, France");
                assert_eq!(back, "Capital, of France");
            }
            other => panic!("expected basic, got {other:?}"),
        }
        assert!(lib.cards[0].tags.is_empty());
    }

    #[test]
    fn cloze_markup_becomes_a_cloze_card() {
        let (mut lib, deck) = library();
        let text = "The {{c1::mitochondria}} is the powerhouse\t(see notes)\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        match &lib.cards[0].kind {
            CardKind::Cloze { text } => {
                assert_eq!(text, "The {{c1::mitochondria}} is the powerhouse");
            }
            other => panic!("expected cloze, got {other:?}"),
        }
    }

    #[test]
    fn duplicates_are_skipped_case_insensitively() {
        let (mut lib, deck) = library();
        lib.add_card(
            deck,
            CardKind::Basic {
                front: "What is ATP?".into(),
                back: "energy".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
        // First row duplicates the existing card; second repeats the first
        // row; third is genuinely new.
        let text = "  what is atp?  \tother answer\nWhat is ATP?\tyet another\nFresh\tanswer\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        assert_eq!(report.skipped_duplicates, 2);
        assert_eq!(lib.cards.len(), 2);
        assert_eq!(lib.cards[1].kind.prompt(), "Fresh");
    }

    #[test]
    fn header_row_is_skipped() {
        let (mut lib, deck) = library();
        let text = "Front\tBack\tTags\nWhat is 2+2?\t4\tmath\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        assert_eq!(lib.cards.len(), 1);
        assert_eq!(lib.cards[0].kind.prompt(), "What is 2+2?");

        // The `Question` spelling is recognised too.
        let text2 = "Question,Answer\nCapital of France?,Paris\n";
        let report2 = run(text2, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report2.imported, 1);
        assert_eq!(lib.cards.len(), 2);
    }

    #[test]
    fn deck_column_creates_decks_on_demand() {
        let (mut lib, deck) = library();
        let mapping = ImportMapping {
            front: 0,
            back: 1,
            tags: Some(2),
            deck: Some(3),
        };
        let text = "Kanji card\tanswer\ttag\tLanguages::Japanese\n";
        let report = run(text, &mut lib, deck, &mapping);
        assert_eq!(report.imported, 1);
        let card_deck = lib.cards[0].deck_id;
        assert_ne!(card_deck, deck);
        assert_eq!(
            lib.deck_path(card_deck).unwrap(),
            vec!["Languages", "Japanese"]
        );
        assert_eq!(lib.cards[0].tags, vec!["tag"]);

        // A second import reuses the decks instead of duplicating names.
        let text2 = "Another card\tanswer\t\tLanguages::Japanese\n";
        let report2 = run(text2, &mut lib, deck, &mapping);
        assert_eq!(report2.imported, 1);
        assert_eq!(lib.cards[1].deck_id, card_deck);
        assert_eq!(lib.decks.len(), 3); // Default + Languages + Japanese
    }

    #[test]
    fn deck_subtree_is_not_searched_for_duplicates_of_another_deck() {
        // A duplicate in the fallback deck must not block a row that targets
        // a different, as-yet-missing deck.
        let (mut lib, deck) = library();
        lib.add_card(
            deck,
            CardKind::Basic {
                front: "Same front".into(),
                back: "x".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
        let mapping = ImportMapping {
            tags: None,
            deck: Some(2),
            ..ImportMapping::default()
        };
        let text = "Same front\tanswer\tOtherDeck\n";
        let report = run(text, &mut lib, deck, &mapping);
        assert_eq!(report.imported, 1);
        assert_eq!(report.skipped_duplicates, 0);
    }

    #[test]
    fn html_is_sanitised_on_import() {
        let (mut lib, deck) = library();
        let text = "Line one<br>line two\t<b>bold</b> and <i>italic</i>\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        match &lib.cards[0].kind {
            CardKind::Basic { front, back } => {
                assert_eq!(front, "Line one\nline two");
                assert_eq!(back, "bold and italic");
            }
            other => panic!("expected basic, got {other:?}"),
        }
    }

    #[test]
    fn sanitize_html_handles_br_variants_and_keeps_unknown_tags() {
        assert_eq!(sanitize_html("a<br>b"), "a\nb");
        assert_eq!(sanitize_html("a<BR/>b"), "a\nb");
        assert_eq!(sanitize_html("a<br />b"), "a\nb");
        assert_eq!(sanitize_html("<b>x</b><i>y</i>"), "xy");
        // Unknown tags are left verbatim (V1 cards are plain text).
        assert_eq!(
            sanitize_html("keep <div>this</div>"),
            "keep <div>this</div>"
        );
        // A bare `<` without a closing `>` is not a tag.
        assert_eq!(sanitize_html("2 < 3 and 4 > 3"), "2 < 3 and 4 > 3");
    }

    #[test]
    fn invalid_rows_are_skipped_not_fatal() {
        let (mut lib, deck) = library();
        let text = "only a front\t\n\ngood\tanswer\n\tmissing front\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 1);
        assert_eq!(report.skipped_invalid, 2);
        assert_eq!(lib.cards.len(), 1);
    }

    #[test]
    fn crlf_and_trailing_newline_yield_no_phantom_rows() {
        let (mut lib, deck) = library();
        let text = "one\ttwo\r\nthree\tfour\r\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 2);
        assert_eq!(lib.cards.len(), 2);
    }

    #[test]
    fn report_summary_is_human_readable() {
        let report = ImportReport {
            imported: 120,
            skipped_duplicates: 8,
            skipped_invalid: 0,
        };
        assert_eq!(report.summary(), "imported 120, skipped 8 duplicate(s)");
        let clean = ImportReport {
            imported: 3,
            ..ImportReport::default()
        };
        assert_eq!(clean.summary(), "imported 3");
    }

    #[test]
    fn mapping_and_report_serde_roundtrip() {
        let mapping = ImportMapping {
            front: 1,
            back: 2,
            tags: Some(3),
            deck: Some(4),
        };
        let json = serde_json::to_string(&mapping).unwrap();
        assert_eq!(
            serde_json::from_str::<ImportMapping>(&json).unwrap(),
            mapping
        );

        let report = ImportReport {
            imported: 3,
            skipped_duplicates: 1,
            skipped_invalid: 2,
        };
        let json = serde_json::to_string(&report).unwrap();
        assert_eq!(serde_json::from_str::<ImportReport>(&json).unwrap(), report);
    }

    #[test]
    fn import_text_reads_a_file_and_rejects_non_utf8() {
        let (mut lib, deck) = library();
        let path = std::env::temp_dir().join(format!("engram_imp_{}", uuid::Uuid::new_v4()));
        fs::write(&path, "hello\tworld\tgreeting\n").unwrap();
        let report = import_text(&path, &mut lib, deck, &ImportMapping::default(), now()).unwrap();
        assert_eq!(report.imported, 1);
        fs::remove_file(&path).ok();

        let bad = std::env::temp_dir().join(format!("engram_bad_{}", uuid::Uuid::new_v4()));
        fs::write(&bad, [0xFF, 0xFE, 0x00]).unwrap();
        assert!(matches!(
            import_text(&bad, &mut lib, deck, &ImportMapping::default(), now()),
            Err(DataError::Csv(_))
        ));
        fs::remove_file(&bad).ok();
    }

    #[test]
    fn import_lands_cards_in_the_expected_scope() {
        let (mut lib, deck) = library();
        let text = "a\tb\tx\nc\td\ty\n";
        let report = run(text, &mut lib, deck, &ImportMapping::default());
        assert_eq!(report.imported, 2);
        assert_eq!(lib.cards_in_scope(Scope::Deck(deck)).len(), 2);
    }

    #[test]
    fn detect_separator_prefers_tab_then_comma() {
        assert_eq!(detect_separator("a\tb\n"), '\t');
        assert_eq!(detect_separator("a,b\n"), ',');
        assert_eq!(detect_separator("\na\tb\n"), '\t');
        assert_eq!(detect_separator(""), ',');
    }

    #[test]
    fn deck_paths_deeper_than_the_cap_are_skipped() {
        // Audit finding deck-explosion-frame-freeze: depth cap.
        let (mut lib, deck) = library();
        let spec = ["L1", "L2", "L3", "L4", "L5", "L6"].join("::");
        let mapping = ImportMapping {
            front: 0,
            back: 1,
            tags: None,
            deck: Some(2),
        };
        let text = format!("q1\tb1\t{spec}\n");
        let report = run(&text, &mut lib, deck, &mapping);
        assert_eq!(report.imported, 0);
        assert_eq!(report.skipped_invalid, 1);
        assert_eq!(lib.decks.len(), 1, "no decks created beyond the cap");
    }

    #[test]
    fn deck_creation_is_capped_per_import() {
        // Audit finding deck-explosion-frame-freeze: count cap.
        let (mut lib, deck) = library();
        let mapping = ImportMapping {
            front: 0,
            back: 1,
            tags: None,
            deck: Some(2),
        };
        let mut text = String::new();
        let rows = MAX_IMPORT_DECKS + 5;
        for i in 0..rows {
            text.push_str(&format!("card{i}\tx\tD{i}\n"));
        }
        let report = run(&text, &mut lib, deck, &mapping);
        assert_eq!(lib.decks.len(), 1 + MAX_IMPORT_DECKS);
        assert_eq!(report.imported, MAX_IMPORT_DECKS);
        assert_eq!(report.skipped_invalid, 5);
    }
}
