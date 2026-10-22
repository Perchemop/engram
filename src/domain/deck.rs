//! Deck entities: hierarchical organisation for cards.
//!
//! Decks form a forest (a deck may have at most one parent). The library
//! owns the list; this module defines the node type and its helpers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Public identifier of a deck (UUIDv4).
pub type DeckId = Uuid;

/// Errors raised by deck operations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeckError {
    #[error("deck not found: {0}")]
    NotFound(DeckId),
    #[error("deck name is empty")]
    EmptyName,
    #[error("a sibling deck named “{0}” already exists")]
    DuplicateName(String),
    #[error("cannot move a deck into its own subtree")]
    Cycle,
    #[error("parent deck not found")]
    ParentNotFound,
}

/// A named colour token for a deck (original palette; UI maps it to RGB).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeckColor {
    Violet,
    Blue,
    Teal,
    Green,
    Amber,
    Orange,
    Red,
    Slate,
}

impl DeckColor {
    /// All tokens in palette order.
    pub const ALL: [DeckColor; 8] = [
        DeckColor::Violet,
        DeckColor::Blue,
        DeckColor::Teal,
        DeckColor::Green,
        DeckColor::Amber,
        DeckColor::Orange,
        DeckColor::Red,
        DeckColor::Slate,
    ];

    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Violet => "Violet",
            Self::Blue => "Blue",
            Self::Teal => "Teal",
            Self::Green => "Green",
            Self::Amber => "Amber",
            Self::Orange => "Orange",
            Self::Red => "Red",
            Self::Slate => "Slate",
        }
    }

    /// The RGB triple the UI renders for this token.
    #[must_use]
    pub fn rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Violet => (124, 92, 255),
            Self::Blue => (66, 133, 244),
            Self::Teal => (32, 178, 170),
            Self::Green => (76, 175, 80),
            Self::Amber => (240, 180, 41),
            Self::Orange => (255, 138, 61),
            Self::Red => (235, 87, 87),
            Self::Slate => (130, 142, 160),
        }
    }
}

/// A deck node. Cards reference decks by [`DeckId`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Deck {
    pub id: DeckId,
    pub parent: Option<DeckId>,
    pub name: String,
    pub color: Option<DeckColor>,
    /// UI-only collapse state, persisted for convenience.
    pub collapsed: bool,
    pub created: DateTime<Utc>,
}

impl Deck {
    /// Create a deck node. Callers enforce sibling-name uniqueness.
    #[must_use]
    pub fn new(
        parent: Option<DeckId>,
        name: String,
        color: Option<DeckColor>,
        now: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            parent,
            name: name.trim().to_string(),
            color,
            collapsed: false,
            created: now,
        }
    }
}

/// A flattened view entry of the deck tree, in display order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckSummary {
    pub id: DeckId,
    pub parent: Option<DeckId>,
    pub depth: usize,
    pub name: String,
    pub color: Option<DeckColor>,
    pub card_count: usize,
    pub due_count: usize,
}

/// Order decks as a depth-first forest: parents before children, siblings
/// alphabetically. Deterministic and stable for tests.
///
/// `counts` maps deck id → (card_count, due_count) for the summaries.
#[must_use]
pub fn summarise_forest(
    decks: &[Deck],
    counts: &std::collections::HashMap<DeckId, (usize, usize)>,
) -> Vec<DeckSummary> {
    let mut children: std::collections::HashMap<Option<DeckId>, Vec<&Deck>> =
        std::collections::HashMap::new();
    for d in decks {
        children.entry(d.parent).or_default().push(d);
    }
    for siblings in children.values_mut() {
        siblings.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    let mut out = Vec::with_capacity(decks.len());
    let mut stack: Vec<(&Deck, usize)> = children
        .get(&None)
        .map(|roots| roots.iter().rev().map(|d| (*d, 0)).collect())
        .unwrap_or_default();
    while let Some((deck, depth)) = stack.pop() {
        out.push(DeckSummary {
            id: deck.id,
            parent: deck.parent,
            depth,
            name: deck.name.clone(),
            color: deck.color,
            card_count: counts.get(&deck.id).copied().unwrap_or((0, 0)).0,
            due_count: counts.get(&deck.id).copied().unwrap_or((0, 0)).1,
        });
        if let Some(kids) = children.get(&Some(deck.id)) {
            for kid in kids.iter().rev() {
                stack.push((kid, depth + 1));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck(parent: Option<DeckId>, name: &str, now: DateTime<Utc>) -> Deck {
        Deck::new(parent, name.to_string(), None, now)
    }

    #[test]
    fn forest_orders_parents_before_children_alphabetically() {
        let now = Utc::now();
        let zeta = deck(None, "Zeta", now);
        let alpha = deck(None, "alpha", now);
        let child = deck(Some(alpha.id), "child", now);
        let grand = deck(Some(child.id), "grand", now);
        let decks = vec![zeta.clone(), grand.clone(), alpha.clone(), child.clone()];
        let counts = std::collections::HashMap::new();
        let summary = summarise_forest(&decks, &counts);
        let names: Vec<&str> = summary.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["alpha", "child", "grand", "Zeta"]);
        assert_eq!(summary[1].depth, 1);
        assert_eq!(summary[2].depth, 2);
    }

    #[test]
    fn deck_serde_roundtrip() {
        let d = Deck::new(None, "Test".into(), Some(DeckColor::Teal), Utc::now());
        let json = serde_json::to_string(&d).unwrap();
        let back: Deck = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
    }
}
