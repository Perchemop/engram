//! Dialog state machines: card editor, deck dialog, library settings.

use uuid::Uuid;

use crate::domain::{CardId, CardKind, DeckColor, DeckId, Library, LibrarySettings, SchedParams};

/// Create-or-edit state for the card editor window.
#[derive(Debug, Clone, PartialEq)]
pub struct CardEditor {
    /// `None` while creating a new card.
    pub card_id: Option<CardId>,
    pub is_cloze: bool,
    pub front: String,
    pub back: String,
    pub tags: String,
    pub deck_id: DeckId,
    pub error: Option<String>,
}

impl CardEditor {
    /// A blank editor for a new card in `deck_id`.
    #[must_use]
    pub fn new_card(deck_id: DeckId) -> Self {
        Self {
            card_id: None,
            is_cloze: false,
            front: String::new(),
            back: String::new(),
            tags: String::new(),
            deck_id,
            error: None,
        }
    }

    /// An editor pre-filled from an existing card.
    #[must_use]
    pub fn edit_card(card_id: CardId, lib: &Library) -> Option<Self> {
        let card = lib.card(card_id)?;
        let (is_cloze, front, back) = match &card.kind {
            CardKind::Basic { front, back } => (false, front.clone(), back.clone()),
            CardKind::Cloze { text } => (true, text.clone(), String::new()),
        };
        Some(Self {
            card_id: Some(card_id),
            is_cloze,
            front,
            back,
            tags: card.tags.join(", "),
            deck_id: card.deck_id,
            error: None,
        })
    }

    /// Build the card kind from the editor fields.
    #[must_use]
    pub fn build_kind(&self) -> CardKind {
        if self.is_cloze {
            CardKind::Cloze {
                text: self.front.clone(),
            }
        } else {
            CardKind::Basic {
                front: self.front.clone(),
                back: self.back.clone(),
            }
        }
    }

    /// Parse the tags field (comma or space separated).
    #[must_use]
    pub fn parse_tags(&self) -> Vec<String> {
        self.tags
            .split([',', ';'])
            .flat_map(|t| t.split_whitespace())
            .map(str::to_string)
            .collect()
    }
}

/// Create-or-edit state for the deck dialog.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckDialog {
    /// `None` while creating a deck.
    pub deck_id: Option<DeckId>,
    pub parent: Option<DeckId>,
    pub name: String,
    pub color: Option<DeckColor>,
    pub error: Option<String>,
}

impl DeckDialog {
    /// A blank dialog for a new deck under `parent`.
    #[must_use]
    pub fn new_deck(parent: Option<DeckId>) -> Self {
        Self {
            deck_id: None,
            parent,
            name: String::new(),
            color: None,
            error: None,
        }
    }

    /// A dialog pre-filled from an existing deck.
    #[must_use]
    pub fn edit_deck(deck_id: DeckId, lib: &Library) -> Option<Self> {
        let deck = lib.deck(deck_id)?;
        Some(Self {
            deck_id: Some(deck_id),
            parent: deck.parent,
            name: deck.name.clone(),
            color: deck.color,
            error: None,
        })
    }
}

/// Library settings dialog state (editable copy).
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsDialog {
    pub settings: LibrarySettings,
    /// Working copies for the step-list text inputs.
    pub learning_steps: String,
    pub relearn_steps: String,
    pub error: Option<String>,
}

impl SettingsDialog {
    /// Snapshot the current settings for editing.
    #[must_use]
    pub fn from_library(lib: &Library) -> Self {
        let s = &lib.settings;
        Self {
            settings: s.clone(),
            learning_steps: steps_to_string(&s.sched.learning_steps_minutes),
            relearn_steps: steps_to_string(&s.sched.relearn_steps_minutes),
            error: None,
        }
    }

    /// Parse the step lists; invalid input keeps the previous values.
    pub fn sync_steps(&mut self) {
        if let Some(v) = parse_steps(&self.learning_steps) {
            self.settings.sched.learning_steps_minutes = v;
        }
        if let Some(v) = parse_steps(&self.relearn_steps) {
            self.settings.sched.relearn_steps_minutes = v;
        }
    }
}

fn steps_to_string(steps: &[u32]) -> String {
    steps
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Parse "1, 10" style minute lists. Empty input yields `None` (keep old).
fn parse_steps(text: &str) -> Option<Vec<u32>> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for part in trimmed.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.parse::<u32>() {
            Ok(v) if v > 0 => out.push(v.min(1440)),
            _ => return None,
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Basic sanity check before applying settings.
#[must_use]
pub fn settings_valid(s: &SchedParams) -> bool {
    s.starting_ease.is_finite()
        && s.graduating_interval_days.is_finite()
        && s.easy_interval_days.is_finite()
        && s.interval_modifier.is_finite()
        && s.max_interval_days.is_finite()
}

/// Window id helper so egui state persists correctly per dialog instance.
#[must_use]
pub fn window_id(prefix: &str, id: Option<Uuid>) -> String {
    match id {
        Some(u) => format!("{prefix} {u}"),
        None => prefix.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn parse_steps_handles_lists() {
        assert_eq!(parse_steps("1, 10"), Some(vec![1, 10]));
        assert_eq!(parse_steps("5"), Some(vec![5]));
        assert_eq!(parse_steps(""), None);
        assert_eq!(parse_steps("abc"), None);
        assert_eq!(parse_steps("0"), None);
        assert_eq!(parse_steps("999999"), Some(vec![1440]));
    }

    #[test]
    fn card_editor_roundtrip_through_library() {
        let mut lib = Library::new("T", Utc::now());
        let deck = lib.decks[0].id;
        let cid = lib
            .add_card(
                deck,
                CardKind::Basic {
                    front: "f".into(),
                    back: "b".into(),
                },
                vec!["tag one".into()],
                Utc::now(),
            )
            .unwrap();
        let ed = CardEditor::edit_card(cid, &lib).unwrap();
        assert_eq!(ed.front, "f");
        assert_eq!(ed.parse_tags(), vec!["tag", "one"]);
        match ed.build_kind() {
            CardKind::Basic { front, .. } => assert_eq!(front, "f"),
            _ => panic!("wrong kind"),
        }
    }

    #[test]
    fn window_ids_are_stable() {
        let u = Uuid::new_v4();
        assert_eq!(window_id("card", Some(u)), format!("card {u}"));
        assert_eq!(window_id("deck", None), "deck");
    }
}
