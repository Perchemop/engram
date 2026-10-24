//! Card entities: the atomic unit of active recall.
//!
//! Two card kinds ship in V1:
//! - [`CardKind::Basic`] — a prompt and an answer.
//! - [`CardKind::Cloze`] — a sentence with one or more `{{c1::…}}`
//!   deletions; each deletion number is studied as its own card face.
//!
//! The `{{cn::text}}` convention is a long-standing public interchange
//! format for cloze deletions; Engram reads and writes it so decks stay
//! portable as plain text.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::scheduler::SchedulingState;
use crate::domain::DeckId;

/// Public identifier of a card (UUIDv4).
pub type CardId = Uuid;

/// The id used by freshly-drafted cards that are not yet stored.
pub const CARD_ID_NIL: CardId = Uuid::nil();

/// Errors raised when creating or editing cards.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CardError {
    #[error("card content is empty")]
    EmptyContent,
    #[error("cloze card has no {{{{c1::…}}}} deletion")]
    ClozeWithoutDeletion,
    #[error("card not found: {0}")]
    NotFound(CardId),
}

/// The content of a card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardKind {
    /// A prompt on the front, the answer on the back.
    Basic { front: String, back: String },
    /// A sentence containing `{{c1::…}}` style deletions.
    Cloze { text: String },
}

impl CardKind {
    /// The recall prompt shown before the answer is revealed.
    ///
    /// For a cloze card every deletion is replaced by `[…]`; the first
    /// deletion group is the studied one in V1 (multi-group cards reveal
    /// all groups at once and are graded as a whole).
    #[must_use]
    pub fn prompt(&self) -> String {
        match self {
            Self::Basic { front, .. } => front.trim().to_string(),
            Self::Cloze { text } => render_cloze_blank(text),
        }
    }

    /// The answer revealed after recall.
    #[must_use]
    pub fn answer(&self) -> String {
        match self {
            Self::Basic { back, .. } => back.trim().to_string(),
            Self::Cloze { text } => text.trim().to_string(),
        }
    }

    /// Plain-text projection used for search and export.
    #[must_use]
    pub fn searchable_text(&self) -> String {
        match self {
            Self::Basic { front, back } => format!("{front}\n{back}"),
            Self::Cloze { text } => text.clone(),
        }
    }

    /// Human-readable kind label.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Basic { .. } => "Basic",
            Self::Cloze { .. } => "Cloze",
        }
    }

    /// Validate the content: nothing may be empty, and cloze cards must
    /// actually contain at least one deletion.
    pub fn validate(&self) -> Result<(), CardError> {
        match self {
            Self::Basic { front, back } => {
                if front.trim().is_empty() || back.trim().is_empty() {
                    Err(CardError::EmptyContent)
                } else {
                    Ok(())
                }
            }
            Self::Cloze { text } => {
                if text.trim().is_empty() {
                    Err(CardError::EmptyContent)
                } else if count_cloze_groups(text) == 0 {
                    Err(CardError::ClozeWithoutDeletion)
                } else {
                    Ok(())
                }
            }
        }
    }
}

/// Count the distinct `{{cN::…}}` deletion groups in `text`.
#[must_use]
pub fn count_cloze_groups(text: &str) -> usize {
    collect_cloze_numbers(text).len()
}

/// Every distinct cloze group number present in `text`, ascending.
#[must_use]
pub fn collect_cloze_numbers(text: &str) -> Vec<u32> {
    let mut numbers: Vec<u32> = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{c") {
        let after_prefix = &rest[start + 3..];
        let digits: String = after_prefix
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        if let Ok(n) = digits.parse::<u32>() {
            if !numbers.contains(&n) {
                numbers.push(n);
            }
        }
        rest = after_prefix;
    }
    numbers.sort_unstable();
    numbers
}

/// Replace every `{{cN::…}}` deletion with a blank placeholder.
#[must_use]
pub fn render_cloze_blank(text: &str) -> String {
    replace_cloze(text, |_, inner| format!("[{inner}]"))
}

/// Replace every `{{cN::…}}` deletion with its inner text (the answer view).
#[must_use]
pub fn render_cloze_revealed(text: &str) -> String {
    replace_cloze(text, |_, inner| inner.to_string())
}

fn replace_cloze(text: &str, f: impl Fn(u32, &str) -> String) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("{{c") {
        out.push_str(&rest[..start]);
        let after_prefix = &rest[start + 3..];
        let digits: String = after_prefix
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        let after_digits = &after_prefix[digits.len()..];
        let inner = if digits.is_empty() {
            None
        } else {
            after_digits
                .strip_prefix("::")
                .and_then(|s| s.split_once("}}").map(|(i, _)| i))
        };
        match inner {
            Some(inner) => {
                let n = digits.parse::<u32>().unwrap_or(1);
                out.push_str(&f(n, inner));
                rest = &after_digits[2 + inner.len() + 2..]; // skip "::inner}}"
            }
            None => {
                // Not a well-formed deletion: keep the raw bytes.
                out.push_str("{{c");
                out.push_str(&digits);
                rest = after_digits;
            }
        }
    }
    out.push_str(rest);
    out
}

/// A single flashcard with its scheduling state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub id: CardId,
    pub deck_id: DeckId,
    pub kind: CardKind,
    pub tags: Vec<String>,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub sched: SchedulingState,
    /// Paused by the user: never queued for study (`docs/research/
    /// card-history-suspend-bury.md`).
    #[serde(default)]
    pub suspended: bool,
    /// Postponed by the user until this instant passes; then eligible
    /// again without any mutation.
    #[serde(default)]
    pub buried_until: Option<DateTime<Utc>>,
}

impl Card {
    /// Create a new card in the *New* phase. Validates the content.
    ///
    /// # Errors
    /// Returns [`CardError`] when the content is empty or a cloze card has
    /// no deletion.
    pub fn new(
        deck_id: DeckId,
        kind: CardKind,
        tags: Vec<String>,
        now: DateTime<Utc>,
        starting_ease: f64,
    ) -> Result<Self, CardError> {
        kind.validate()?;
        let tags = normalise_tags(tags);
        Ok(Self {
            id: Uuid::new_v4(),
            deck_id,
            kind,
            tags,
            created: now,
            modified: now,
            sched: SchedulingState::new(now, starting_ease),
            suspended: false,
            buried_until: None,
        })
    }

    /// Replace the content of this card, bumping `modified`.
    ///
    /// # Errors
    /// See [`CardKind::validate`].
    pub fn set_kind(&mut self, kind: CardKind, now: DateTime<Utc>) -> Result<(), CardError> {
        kind.validate()?;
        self.kind = kind;
        self.modified = now;
        Ok(())
    }

    /// True when the card must not enter any study queue right now:
    /// suspended, or buried until a future instant.
    #[must_use]
    pub fn is_blocked(&self, now: DateTime<Utc>) -> bool {
        self.suspended || self.buried_until.is_some_and(|t| t > now)
    }

    /// Replace the tags with a normalised set, bumping `modified`.
    pub fn set_tags(&mut self, tags: Vec<String>, now: DateTime<Utc>) {
        self.tags = normalise_tags(tags);
        self.modified = now;
    }
}

/// Normalise a tag list: trim, lowercase, drop empties, dedupe, sort.
#[must_use]
pub fn normalise_tags(tags: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for t in tags {
        let t = t.trim().to_lowercase();
        if !t.is_empty() && !out.contains(&t) {
            out.push(t);
        }
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_validation_rejects_empty_sides() {
        let now = chrono::Utc::now();
        let deck = Uuid::new_v4();
        assert!(Card::new(
            deck,
            CardKind::Basic {
                front: "  ".into(),
                back: "b".into()
            },
            vec![],
            now,
            2.5
        )
        .is_err());
        assert!(Card::new(
            deck,
            CardKind::Basic {
                front: "f".into(),
                back: "".into()
            },
            vec![],
            now,
            2.5
        )
        .is_err());
        let card = Card::new(
            deck,
            CardKind::Basic {
                front: "f".into(),
                back: "b".into(),
            },
            vec![],
            now,
            2.5,
        )
        .unwrap();
        assert_eq!(card.kind.prompt(), "f");
        assert_eq!(card.kind.answer(), "b");
    }

    #[test]
    fn cloze_requires_at_least_one_deletion() {
        let now = chrono::Utc::now();
        let deck = Uuid::new_v4();
        assert!(Card::new(
            deck,
            CardKind::Cloze {
                text: "no deletions".into()
            },
            vec![],
            now,
            2.5
        )
        .is_err());
        let card = Card::new(
            deck,
            CardKind::Cloze {
                text: "The {{c1::mitochondria}} is the powerhouse".into(),
            },
            vec![],
            now,
            2.5,
        )
        .unwrap();
        assert_eq!(card.kind.prompt(), "The [mitochondria] is the powerhouse");
        assert_eq!(
            card.kind.answer(),
            "The {{c1::mitochondria}} is the powerhouse"
        );
    }

    #[test]
    fn cloze_numbers_are_collected_and_deduped() {
        let text = "{{c2::b}} and {{c1::a}} and {{c2::b}} again";
        assert_eq!(collect_cloze_numbers(text), vec![1, 2]);
        assert_eq!(count_cloze_groups(text), 2);
    }

    #[test]
    fn unterminated_cloze_is_kept_verbatim() {
        assert_eq!(render_cloze_blank("broken {{c1::oops"), "broken {{c1::oops");
        assert_eq!(
            render_cloze_blank("not a deletion {c1::x}"),
            "not a deletion {c1::x}"
        );
    }

    #[test]
    fn tags_are_normalised() {
        let tags = normalise_tags(vec![
            " Physics ".into(),
            "physics".into(),
            "".into(),
            "Biology".into(),
        ]);
        assert_eq!(tags, vec!["biology", "physics"]);
    }

    #[test]
    fn card_serde_roundtrip() {
        let now = chrono::Utc::now();
        let deck = Uuid::new_v4();
        let card = Card::new(
            deck,
            CardKind::Cloze {
                text: "{{c1::x}}".into(),
            },
            vec!["t".into()],
            now,
            2.5,
        )
        .unwrap();
        let json = serde_json::to_string(&card).unwrap();
        let back: Card = serde_json::from_str(&json).unwrap();
        assert_eq!(back, card);
    }
}
