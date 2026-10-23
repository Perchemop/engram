//! The library: the root aggregate owning decks, cards and review history.
//!
//! Every mutation is a named command returning `Result`; queries are
//! read-only. Persistence (serialisation) is layered on top in `src/data`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::card::{normalise_tags, Card, CardError, CardId, CardKind};
use crate::domain::deck::{summarise_forest, Deck, DeckColor, DeckError, DeckId, DeckSummary};
use crate::domain::scheduler::{schedule, Grade, LearnPhase, SchedParams, SchedulingState};
use crate::domain::session::StudySession;

/// Schema version of the persisted library file.
pub const SCHEMA_VERSION: u32 = 1;

/// Default number of new cards introduced per day.
pub const DEFAULT_DAILY_NEW_LIMIT: u32 = 20;

/// Default number of reviews served per day.
pub const DEFAULT_DAILY_REVIEWS_LIMIT: u32 = 200;

/// Errors raised by library commands.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LibraryError {
    #[error(transparent)]
    Deck(#[from] DeckError),
    #[error(transparent)]
    Card(#[from] CardError),
    #[error("card not found: {0}")]
    CardNotFound(CardId),
    #[error("no cards are due in the selected scope")]
    NothingToStudy,
}

/// Library-wide settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LibrarySettings {
    pub sched: SchedParams,
    pub daily_new_limit: u32,
    pub daily_reviews_limit: u32,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            sched: SchedParams::default(),
            daily_new_limit: DEFAULT_DAILY_NEW_LIMIT,
            daily_reviews_limit: DEFAULT_DAILY_REVIEWS_LIMIT,
        }
    }
}

/// One recorded review. Append-only; drives the statistics view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewLog {
    pub id: Uuid,
    pub card_id: CardId,
    pub deck_id: DeckId,
    pub at: DateTime<Utc>,
    pub grade: Grade,
    pub interval_before_days: f64,
    pub interval_after_days: f64,
}

/// The result of a successful [`Library::record_review`].
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewOutcome {
    pub card_id: CardId,
    pub state_after: SchedulingState,
    pub log: ReviewLog,
}

/// The scope of a study session or count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Every deck in the library.
    All,
    /// One deck and all of its descendants.
    Deck(DeckId),
}

impl Scope {
    /// Map an optional deck selection onto a scope (`None` = all).
    #[must_use]
    pub fn from_option(deck: Option<DeckId>) -> Self {
        match deck {
            None => Self::All,
            Some(id) => Self::Deck(id),
        }
    }
}

/// Which part of the queue an entry belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueKind {
    /// Graduated or lapsed card past its due date.
    Review,
    /// Card inside a learning/relearning step.
    Learning,
    /// Brand-new card being introduced.
    New,
}

impl QueueKind {
    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Review => "Review",
            Self::Learning => "Learning",
            Self::New => "New",
        }
    }
}

/// Due/new counts for a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DueCounts {
    pub new_cards: usize,
    pub learning: usize,
    pub review: usize,
}

impl DueCounts {
    /// Total cards the queue will serve (respecting limits is the session
    /// builder's job).
    #[must_use]
    pub fn total(&self) -> usize {
        self.new_cards + self.learning + self.review
    }
}

/// A read-side filter for the card list.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CardFilter {
    /// Restrict to a deck subtree (`None` = all decks).
    pub deck: Option<DeckId>,
    /// Cards must carry *all* of these tags.
    pub tags: Vec<String>,
    /// Case-insensitive substring over card text.
    pub query: String,
}

impl CardFilter {
    /// True when the filter selects everything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.deck.is_none() && self.tags.is_empty() && self.query.trim().is_empty()
    }
}

/// The root aggregate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Library {
    pub schema_version: u32,
    pub name: String,
    pub settings: LibrarySettings,
    pub decks: Vec<Deck>,
    pub cards: Vec<Card>,
    pub reviews: Vec<ReviewLog>,
}

impl Library {
    /// An empty library with one top-level "Default" deck.
    #[must_use]
    pub fn new(name: &str, now: DateTime<Utc>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            name: name.to_string(),
            settings: LibrarySettings::default(),
            decks: vec![Deck::new(None, "Default".to_string(), None, now)],
            cards: Vec::new(),
            reviews: Vec::new(),
        }
    }

    // ------------------------------------------------------------------
    // Lookups
    // ------------------------------------------------------------------

    /// Find a deck by id.
    #[must_use]
    pub fn deck(&self, id: DeckId) -> Option<&Deck> {
        self.decks.iter().find(|d| d.id == id)
    }

    /// Find a card by id.
    #[must_use]
    pub fn card(&self, id: CardId) -> Option<&Card> {
        self.cards.iter().find(|c| c.id == id)
    }

    /// Mutable card lookup.
    #[must_use]
    pub fn card_mut(&mut self, id: CardId) -> Option<&mut Card> {
        self.cards.iter_mut().find(|c| c.id == id)
    }

    /// Path segments from the top level down to and including `id`.
    /// Guarded against corrupted parent cycles: the walk stops after
    /// at most one step per stored deck.
    ///
    /// # Errors
    /// [`DeckError::NotFound`] when the deck does not exist.
    pub fn deck_path(&self, id: DeckId) -> Result<Vec<String>, DeckError> {
        let mut segments = Vec::new();
        let mut current = Some(id);
        let mut steps = 0usize;
        let max_steps = self.decks.len() + 1;
        while let Some(cid) = current {
            let deck = self.deck(cid).ok_or(DeckError::NotFound(cid))?;
            segments.push(deck.name.clone());
            steps += 1;
            if steps > max_steps {
                break; // corrupted parent chain: return the partial path
            }
            current = deck.parent;
        }
        segments.reverse();
        Ok(segments)
    }

    /// Display name with ancestry, e.g. `Languages › Japanese › Kanji`.
    ///
    /// # Errors
    /// [`DeckError::NotFound`] when the deck does not exist.
    pub fn deck_display_name(&self, id: DeckId) -> Result<String, DeckError> {
        Ok(self.deck_path(id)?.join(" › "))
    }

    /// The deck plus every descendant id. Guarded against corrupted parent
    /// cycles (e.g. a hand-edited file): every id is pushed at most once.
    #[must_use]
    pub fn descendant_ids(&self, id: DeckId) -> Vec<DeckId> {
        let mut out = vec![id];
        let mut seen: std::collections::HashSet<DeckId> = std::collections::HashSet::new();
        seen.insert(id);
        let mut i = 0;
        while i < out.len() {
            let parent = out[i];
            for d in &self.decks {
                if d.parent == Some(parent) && seen.insert(d.id) {
                    out.push(d.id);
                }
            }
            i += 1;
        }
        out
    }

    /// All tag strings in use, sorted.
    #[must_use]
    pub fn all_tags(&self) -> Vec<String> {
        let mut tags: Vec<String> = Vec::new();
        for c in &self.cards {
            for t in &c.tags {
                if !tags.contains(t) {
                    tags.push(t.clone());
                }
            }
        }
        tags.sort();
        tags
    }

    /// Cards belonging to a scope (deck subtree or everything).
    #[must_use]
    pub fn cards_in_scope(&self, scope: Scope) -> Vec<&Card> {
        match scope {
            Scope::All => self.cards.iter().collect(),
            Scope::Deck(id) => {
                let ids = self.descendant_ids(id);
                self.cards
                    .iter()
                    .filter(|c| ids.contains(&c.deck_id))
                    .collect()
            }
        }
    }

    /// Whether `ancestor` is `id` itself or an ancestor of it. Guarded
    /// against corrupted parent cycles (bounded by the deck count).
    #[must_use]
    pub fn is_ancestor(&self, ancestor: DeckId, id: DeckId) -> bool {
        let mut current = Some(id);
        let mut steps = 0usize;
        let max_steps = self.decks.len() + 1;
        while let Some(cid) = current {
            if cid == ancestor {
                return true;
            }
            steps += 1;
            if steps > max_steps {
                return false;
            }
            current = self.deck(cid).and_then(|d| d.parent);
        }
        false
    }

    // ------------------------------------------------------------------
    // Deck commands
    // ------------------------------------------------------------------

    /// Create a deck under `parent` (`None` = top level).
    ///
    /// # Errors
    /// [`LibraryError::Deck`] for empty names, unknown parents or duplicate
    /// sibling names (case-insensitive).
    pub fn new_deck(
        &mut self,
        parent: Option<DeckId>,
        name: &str,
        color: Option<DeckColor>,
        now: DateTime<Utc>,
    ) -> Result<DeckId, LibraryError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DeckError::EmptyName.into());
        }
        if let Some(pid) = parent {
            if !self.decks.iter().any(|d| d.id == pid) {
                return Err(DeckError::ParentNotFound.into());
            }
        }
        let dup = self
            .decks
            .iter()
            .any(|d| d.parent == parent && d.name.eq_ignore_ascii_case(trimmed));
        if dup {
            return Err(DeckError::DuplicateName(trimmed.to_string()).into());
        }
        let deck = Deck::new(parent, trimmed.to_string(), color, now);
        let id = deck.id;
        self.decks.push(deck);
        Ok(id)
    }

    /// Rename a deck.
    ///
    /// # Errors
    /// [`LibraryError::Deck`] for empty names, unknown decks or duplicate
    /// sibling names.
    pub fn rename_deck(&mut self, id: DeckId, name: &str) -> Result<(), LibraryError> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err(DeckError::EmptyName.into());
        }
        let parent = self.deck(id).ok_or(DeckError::NotFound(id))?.parent;
        let dup = self
            .decks
            .iter()
            .any(|d| d.id != id && d.parent == parent && d.name.eq_ignore_ascii_case(trimmed));
        if dup {
            return Err(DeckError::DuplicateName(trimmed.to_string()).into());
        }
        let deck = self
            .decks
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or(DeckError::NotFound(id))?;
        deck.name = trimmed.to_string();
        Ok(())
    }

    /// Change a deck's colour token.
    ///
    /// # Errors
    /// [`LibraryError::Deck`] when the deck does not exist.
    pub fn set_deck_color(
        &mut self,
        id: DeckId,
        color: Option<DeckColor>,
    ) -> Result<(), LibraryError> {
        let deck = self
            .decks
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or(DeckError::NotFound(id))?;
        deck.color = color;
        Ok(())
    }

    /// Move a deck under a new parent (`None` = top level).
    ///
    /// # Errors
    /// [`LibraryError::Deck`] for unknown decks, unknown parents, or moves
    /// that would create a cycle (moving a deck into its own subtree).
    pub fn move_deck(
        &mut self,
        id: DeckId,
        new_parent: Option<DeckId>,
    ) -> Result<(), LibraryError> {
        if !self.decks.iter().any(|d| d.id == id) {
            return Err(DeckError::NotFound(id).into());
        }
        if let Some(pid) = new_parent {
            if !self.decks.iter().any(|d| d.id == pid) {
                return Err(DeckError::ParentNotFound.into());
            }
            if self.is_ancestor(id, pid) {
                return Err(DeckError::Cycle.into());
            }
        }
        let name = self.deck(id).ok_or(DeckError::NotFound(id))?.name.clone();
        let dup = self
            .decks
            .iter()
            .any(|d| d.id != id && d.parent == new_parent && d.name.eq_ignore_ascii_case(&name));
        if dup {
            return Err(DeckError::DuplicateName(name).into());
        }
        let deck = self
            .decks
            .iter_mut()
            .find(|d| d.id == id)
            .ok_or(DeckError::NotFound(id))?;
        deck.parent = new_parent;
        Ok(())
    }

    /// Delete a deck.
    ///
    /// Semantics (documented): the deck's cards are deleted **with** their
    /// review history entries for those cards; child decks are re-parented
    /// to the deleted deck's parent so no part of the tree is lost.
    ///
    /// # Errors
    /// [`LibraryError::Deck`] when the deck does not exist.
    pub fn delete_deck(&mut self, id: DeckId) -> Result<usize, LibraryError> {
        if !self.decks.iter().any(|d| d.id == id) {
            return Err(DeckError::NotFound(id).into());
        }
        let parent = self.deck(id).ok_or(DeckError::NotFound(id))?.parent;

        // Re-parent children first (a deck may be its own subtree root).
        for d in self.decks.iter_mut() {
            if d.parent == Some(id) {
                d.parent = parent;
            }
        }

        // Delete the deck and its cards + review logs.
        self.decks.retain(|d| d.id != id);
        let deleted_cards = self.cards.iter().filter(|c| c.deck_id == id).count();
        self.cards.retain(|c| c.deck_id != id);
        self.reviews.retain(|r| r.deck_id != id);
        Ok(deleted_cards)
    }

    // ------------------------------------------------------------------
    // Card commands
    // ------------------------------------------------------------------

    /// Add a card to a deck.
    ///
    /// # Errors
    /// [`LibraryError::Deck`] when the deck does not exist;
    /// [`LibraryError::Card`] when the content is invalid.
    pub fn add_card(
        &mut self,
        deck_id: DeckId,
        kind: CardKind,
        tags: Vec<String>,
        now: DateTime<Utc>,
    ) -> Result<CardId, LibraryError> {
        if !self.decks.iter().any(|d| d.id == deck_id) {
            return Err(DeckError::NotFound(deck_id).into());
        }
        let card = Card::new(deck_id, kind, tags, now, self.settings.sched.starting_ease)?;
        let id = card.id;
        self.cards.push(card);
        Ok(id)
    }

    /// Edit a card's content.
    ///
    /// # Errors
    /// [`LibraryError::CardNotFound`] or [`LibraryError::Card`] when the new
    /// content is invalid.
    pub fn set_card_kind(
        &mut self,
        id: CardId,
        kind: CardKind,
        now: DateTime<Utc>,
    ) -> Result<(), LibraryError> {
        let card = self.card_mut(id).ok_or(LibraryError::CardNotFound(id))?;
        card.set_kind(kind, now).map_err(LibraryError::Card)
    }

    /// Edit a card's tags.
    ///
    /// # Errors
    /// [`LibraryError::CardNotFound`].
    pub fn set_card_tags(
        &mut self,
        id: CardId,
        tags: Vec<String>,
        now: DateTime<Utc>,
    ) -> Result<(), LibraryError> {
        let card = self.card_mut(id).ok_or(LibraryError::CardNotFound(id))?;
        card.set_tags(tags, now);
        Ok(())
    }

    /// Move a card to another deck.
    ///
    /// # Errors
    /// [`LibraryError::CardNotFound`] / [`LibraryError::Deck`].
    pub fn move_card(&mut self, id: CardId, deck_id: DeckId) -> Result<(), LibraryError> {
        if !self.decks.iter().any(|d| d.id == deck_id) {
            return Err(DeckError::NotFound(deck_id).into());
        }
        let card = self.card_mut(id).ok_or(LibraryError::CardNotFound(id))?;
        card.deck_id = deck_id;
        Ok(())
    }

    /// Delete a card and its review history.
    ///
    /// # Errors
    /// [`LibraryError::CardNotFound`].
    pub fn delete_card(&mut self, id: CardId) -> Result<(), LibraryError> {
        if !self.cards.iter().any(|c| c.id == id) {
            return Err(LibraryError::CardNotFound(id));
        }
        self.cards.retain(|c| c.id != id);
        self.reviews.retain(|r| r.card_id != id);
        Ok(())
    }

    // ------------------------------------------------------------------
    // Reviews
    // ------------------------------------------------------------------

    /// Record one review: update the card's scheduling state and append to
    /// the review log.
    ///
    /// # Errors
    /// [`LibraryError::CardNotFound`].
    pub fn record_review(
        &mut self,
        card_id: CardId,
        grade: Grade,
        now: DateTime<Utc>,
    ) -> Result<ReviewOutcome, LibraryError> {
        let params = self.settings.sched.clone();
        let card = self
            .card_mut(card_id)
            .ok_or(LibraryError::CardNotFound(card_id))?;
        let interval_before = card.sched.interval_days;
        let after = schedule(now, &params, &card.sched, grade);
        card.sched = after.clone();
        card.modified = now;
        let log = ReviewLog {
            id: Uuid::new_v4(),
            card_id,
            deck_id: card.deck_id,
            at: now,
            grade,
            interval_before_days: interval_before,
            interval_after_days: after.interval_days,
        };
        self.reviews.push(log.clone());
        Ok(ReviewOutcome {
            card_id,
            state_after: after,
            log,
        })
    }

    /// Due/new counts for a scope at `now`.
    #[must_use]
    pub fn due_counts(&self, scope: Scope, now: DateTime<Utc>) -> DueCounts {
        let mut counts = DueCounts::default();
        for c in self.cards_in_scope(scope) {
            match c.sched.phase {
                LearnPhase::New => counts.new_cards += 1,
                LearnPhase::Learning | LearnPhase::Relearning => {
                    if c.sched.due <= now {
                        counts.learning += 1;
                    }
                }
                LearnPhase::Review => {
                    if c.sched.due <= now {
                        counts.review += 1;
                    }
                }
            }
        }
        counts
    }

    /// Flattened deck tree with card and due counts for the given scope.
    #[must_use]
    pub fn deck_summaries(&self, now: DateTime<Utc>) -> Vec<DeckSummary> {
        // Aggregate counts up the tree so a parent shows its subtree totals.
        let mut counts: HashMap<DeckId, (usize, usize)> = HashMap::new();
        let direct: HashMap<DeckId, DueCounts> = self
            .decks
            .iter()
            .map(|d| (d.id, self.due_counts(Scope::Deck(d.id), now)))
            .collect();
        for d in &self.decks {
            let scope_ids = self.descendant_ids(d.id);
            let card_count = self
                .cards
                .iter()
                .filter(|c| scope_ids.contains(&c.deck_id))
                .count();
            let due = scope_ids
                .iter()
                .filter_map(|id| direct.get(id))
                .map(|dc| dc.learning + dc.review)
                .sum::<usize>();
            counts.insert(d.id, (card_count, due));
        }
        summarise_forest(&self.decks, &counts)
    }

    // ------------------------------------------------------------------
    // Search
    // ------------------------------------------------------------------

    /// Search cards: scope + tag conjunction + case-insensitive substring.
    ///
    /// Returns matching card ids in creation order (oldest first).
    #[must_use]
    pub fn search(&self, filter: &CardFilter) -> Vec<CardId> {
        let scope_ids = filter.deck.map(|id| self.descendant_ids(id));
        let query = filter.query.trim().to_lowercase();
        let tags = normalise_tags(filter.tags.clone());
        self.cards
            .iter()
            .filter(|c| {
                if let Some(ids) = &scope_ids {
                    if !ids.contains(&c.deck_id) {
                        return false;
                    }
                }
                if !tags.iter().all(|t| c.tags.contains(t)) {
                    return false;
                }
                if !query.is_empty() && !c.kind.searchable_text().to_lowercase().contains(&query) {
                    return false;
                }
                true
            })
            .map(|c| c.id)
            .collect()
    }

    // ------------------------------------------------------------------
    // Sessions
    // ------------------------------------------------------------------

    /// Build a study session for `scope` at `now`.
    ///
    /// Ordering (documented, deterministic): due *learning* cards first
    /// (soonest due first), then due *review* cards (soonest due first),
    /// then new cards in creation order.
    ///
    /// Daily accounting (per calendar day, derived from the review log):
    /// - Learning/relearning steps are **always served** (they are minutes
    ///   long; capping them would strand cards mid-step).
    /// - Due review cards are capped by `daily_reviews_limit` minus the
    ///   non-introduction reviews already done today.
    /// - New cards are capped by `daily_new_limit` minus the cards already
    ///   *introduced* today (a card's first-ever review = its introduction).
    ///
    /// # Errors
    /// [`LibraryError::NothingToStudy`] when the queue is empty.
    #[allow(clippy::too_many_lines)]
    pub fn build_session(
        &self,
        scope: Scope,
        now: DateTime<Utc>,
    ) -> Result<StudySession, LibraryError> {
        let today = now.date_naive();
        // First-ever review per card, in log order: cards whose first
        // review happened today were introduced today.
        let mut seen_cards: std::collections::HashSet<CardId> = std::collections::HashSet::new();
        let mut introduced_today: std::collections::HashSet<CardId> =
            std::collections::HashSet::new();
        let mut reviews_today = 0usize;
        for r in &self.reviews {
            if seen_cards.insert(r.card_id) && r.at.date_naive() == today {
                introduced_today.insert(r.card_id);
            }
            if r.at.date_naive() == today && !introduced_today.contains(&r.card_id) {
                reviews_today += 1;
            }
        }
        let fresh_budget =
            (self.settings.daily_new_limit as usize).saturating_sub(introduced_today.len());
        let review_budget =
            (self.settings.daily_reviews_limit as usize).saturating_sub(reviews_today);
        let mut learning: Vec<(DateTime<Utc>, CardId)> = Vec::new();
        let mut review: Vec<(DateTime<Utc>, CardId)> = Vec::new();
        let mut fresh: Vec<CardId> = Vec::new();

        for c in self.cards_in_scope(scope) {
            match c.sched.phase {
                LearnPhase::New => fresh.push(c.id),
                LearnPhase::Learning | LearnPhase::Relearning => {
                    if c.sched.due <= now {
                        learning.push((c.sched.due, c.id));
                    }
                }
                LearnPhase::Review => {
                    if c.sched.due <= now {
                        review.push((c.sched.due, c.id));
                    }
                }
            }
        }

        learning.sort_by_key(|a| a.0);
        review.sort_by_key(|a| a.0);
        review.truncate(review_budget);
        fresh.truncate(fresh_budget);

        if learning.is_empty() && review.is_empty() && fresh.is_empty() {
            return Err(LibraryError::NothingToStudy);
        }

        let mut queue = Vec::with_capacity(learning.len() + review.len() + fresh.len());
        for (_, id) in learning {
            queue.push((id, QueueKind::Learning));
        }
        for (_, id) in review {
            queue.push((id, QueueKind::Review));
        }
        for id in fresh {
            queue.push((id, QueueKind::New));
        }

        Ok(StudySession::new(queue, now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    fn lib() -> Library {
        Library::new("Test", now())
    }

    #[test]
    fn fresh_library_has_one_default_deck() {
        let l = lib();
        assert_eq!(l.decks.len(), 1);
        assert!(l.decks[0].parent.is_none());
        assert_eq!(l.decks[0].name, "Default");
    }

    #[test]
    fn deck_crud_and_siblings() {
        let mut l = lib();
        let root = l.decks[0].id;
        let a = l.new_deck(Some(root), "Alpha", None, now()).unwrap();
        assert!(l.new_deck(Some(root), "alpha", None, now()).is_err());
        assert!(l.new_deck(Some(root), "  ", None, now()).is_err());
        assert!(l.new_deck(Some(Uuid::nil()), "X", None, now()).is_err());
        l.rename_deck(a, "Beta").unwrap();
        assert_eq!(l.deck(a).unwrap().name, "Beta");
        assert!(matches!(
            l.move_deck(root, Some(a)),
            Err(LibraryError::Deck(DeckError::Cycle))
        ));
        l.set_deck_color(a, Some(DeckColor::Amber)).unwrap();
        assert_eq!(l.deck(a).unwrap().color, Some(DeckColor::Amber));
    }

    #[test]
    fn descendants_and_paths() {
        let mut l = lib();
        let root = l.decks[0].id;
        let a = l.new_deck(Some(root), "A", None, now()).unwrap();
        let b = l.new_deck(Some(a), "B", None, now()).unwrap();
        let mut ids = l.descendant_ids(root);
        ids.sort();
        let mut expected = vec![root, a, b];
        expected.sort();
        assert_eq!(ids, expected);
        assert_eq!(l.deck_path(b).unwrap(), vec!["Default", "A", "B"]);
        assert_eq!(l.deck_display_name(b).unwrap(), "Default › A › B");
        assert!(l.is_ancestor(a, b));
        assert!(!l.is_ancestor(b, a));
    }

    #[test]
    fn delete_deck_keeps_grandchildren_and_cascades_cards() {
        let mut l = lib();
        let root = l.decks[0].id;
        let mid = l.new_deck(Some(root), "Mid", None, now()).unwrap();
        let leaf = l.new_deck(Some(mid), "Leaf", None, now()).unwrap();
        let card = l
            .add_card(
                mid,
                CardKind::Basic {
                    front: "f".into(),
                    back: "b".into(),
                },
                vec![],
                now(),
            )
            .unwrap();
        l.record_review(card, Grade::Good, now()).unwrap();
        assert_eq!(l.delete_deck(mid).unwrap(), 1);
        assert!(l.deck(mid).is_none());
        assert_eq!(l.deck(leaf).unwrap().parent, Some(root));
        assert!(l.card(card).is_none());
        assert!(l.reviews.is_empty());
    }

    #[test]
    fn card_commands_validate() {
        let mut l = lib();
        let root = l.decks[0].id;
        assert!(l
            .add_card(
                root,
                CardKind::Basic {
                    front: "".into(),
                    back: "b".into()
                },
                vec![],
                now()
            )
            .is_err());
        assert!(l
            .add_card(
                Uuid::nil(),
                CardKind::Basic {
                    front: "f".into(),
                    back: "b".into()
                },
                vec![],
                now()
            )
            .is_err());
        let c1 = l
            .add_card(
                root,
                CardKind::Basic {
                    front: "f".into(),
                    back: "b".into(),
                },
                vec![" Tag ".into()],
                now(),
            )
            .unwrap();
        assert_eq!(l.card(c1).unwrap().tags, vec!["tag"]);
        let other = l.new_deck(None, "Other", None, now()).unwrap();
        l.move_card(c1, other).unwrap();
        assert_eq!(l.card(c1).unwrap().deck_id, other);
        l.set_card_tags(c1, vec!["x".into(), "y".into()], now())
            .unwrap();
        assert_eq!(l.card(c1).unwrap().tags, vec!["x", "y"]);
        l.set_card_kind(
            c1,
            CardKind::Cloze {
                text: "no cloze".into(),
            },
            now(),
        )
        .unwrap_err();
        l.delete_card(c1).unwrap();
        assert!(l.delete_card(c1).is_err());
    }

    #[test]
    fn review_updates_state_and_appends_log() {
        let mut l = lib();
        let root = l.decks[0].id;
        let c = l
            .add_card(
                root,
                CardKind::Basic {
                    front: "f".into(),
                    back: "b".into(),
                },
                vec![],
                now(),
            )
            .unwrap();
        let out = l.record_review(c, Grade::Good, now()).unwrap();
        assert_eq!(out.state_after.phase, LearnPhase::Learning);
        assert_eq!(out.log.card_id, c);
        assert_eq!(out.log.interval_before_days, 0.0);
        assert_eq!(l.reviews.len(), 1);
        assert!(l.record_review(Uuid::nil(), Grade::Good, now()).is_err());
    }

    #[test]
    fn daily_new_limit_counts_introductions_per_day() {
        let mut l = lib();
        l.settings.daily_new_limit = 2;
        let root = l.decks[0].id;
        for i in 0..5 {
            l.add_card(
                root,
                CardKind::Basic {
                    front: format!("n{i}"),
                    back: "x".into(),
                },
                vec![],
                now(),
            )
            .unwrap();
        }

        // Session 1: exactly two new cards.
        let s1 = l.build_session(Scope::All, now()).unwrap();
        assert_eq!(s1.total(), 2);
        let answer_time = now() + chrono::Duration::minutes(1);
        for (cid, _) in s1.queue.clone() {
            l.record_review(cid, Grade::Good, answer_time).unwrap();
        }

        // Session 2, same day: the two learning steps are always served,
        // but no further new cards (introductions already used up).
        // Ten minutes later the learning steps come due: both are served
        // (steps are never capped), but no further new cards — today's
        // introductions are used up.
        let step_due = answer_time + chrono::Duration::minutes(10);
        let s2 = l.build_session(Scope::All, step_due).unwrap();
        assert_eq!(s2.total(), 2);
        assert!(s2.queue.iter().all(|(_, k)| *k == QueueKind::Learning));

        // Graduating both cards (last-step Good) still yields no new cards
        // today.
        for (cid, _) in s2.queue.clone() {
            l.record_review(cid, Grade::Good, step_due).unwrap();
        }
        let s3 = l.build_session(Scope::All, step_due);
        assert!(matches!(s3, Err(LibraryError::NothingToStudy)));

        // Tomorrow the budget resets: two more new cards.
        let tomorrow = now() + chrono::Duration::days(1);
        let s4 = l.build_session(Scope::All, tomorrow).unwrap();
        assert_eq!(s4.total(), 2);
        assert!(s4.queue.iter().all(|(_, k)| *k == QueueKind::New));
    }

    #[test]
    fn daily_review_limit_caps_due_reviews_per_day() {
        let mut l = lib();
        l.settings.daily_reviews_limit = 1;
        let root = l.decks[0].id;
        for i in 0..3 {
            let c = l
                .add_card(
                    root,
                    CardKind::Basic {
                        front: format!("r{i}"),
                        back: "x".into(),
                    },
                    vec![],
                    now(),
                )
                .unwrap();
            let card = l.card_mut(c).unwrap();
            card.sched.phase = LearnPhase::Review;
            card.sched.interval_days = 10.0;
            card.sched.due = now() - chrono::Duration::hours(1);
        }
        // Three due review cards, daily cap 1 → one served.
        let s1 = l.build_session(Scope::All, now()).unwrap();
        assert_eq!(s1.total(), 1);
        let (cid, _) = s1.current().unwrap();
        // Answer Again: the card lapses into relearning, and today's review
        // budget is spent.
        l.record_review(cid, Grade::Again, now()).unwrap();
        // Ten minutes later (step due): the relearning step is always served,
        // but no further review cards fit today's budget.
        let lapse_time = now() + chrono::Duration::minutes(10);
        let s2 = l.build_session(Scope::All, lapse_time).unwrap();
        assert_eq!(s2.total(), 1);
        assert_eq!(s2.queue[0].1, QueueKind::Learning);
        // Tomorrow: budget resets — relearning step + one review (cap 1).
        let s3 = l
            .build_session(Scope::All, now() + chrono::Duration::days(1))
            .unwrap();
        assert_eq!(s3.total(), 2);
    }

    #[test]
    fn due_counts_and_session_ordering() {
        let mut l = lib();
        l.settings.daily_new_limit = 2;
        let root = l.decks[0].id;
        // One due review card.
        let due_card = l
            .add_card(
                root,
                CardKind::Basic {
                    front: "due".into(),
                    back: "x".into(),
                },
                vec![],
                now(),
            )
            .unwrap();
        {
            let card = l.card_mut(due_card).unwrap();
            card.sched.phase = LearnPhase::Review;
            card.sched.interval_days = 10.0;
            card.sched.due = now() - chrono::Duration::hours(2);
        }
        // One learning card due later.
        let learn_card = l
            .add_card(
                root,
                CardKind::Basic {
                    front: "learn".into(),
                    back: "x".into(),
                },
                vec![],
                now(),
            )
            .unwrap();
        {
            let card = l.card_mut(learn_card).unwrap();
            card.sched.phase = LearnPhase::Learning;
            card.sched.due = now() - chrono::Duration::minutes(5);
        }
        // Three new cards (limit 2).
        let mut fresh_ids = Vec::new();
        for i in 0..3 {
            fresh_ids.push(
                l.add_card(
                    root,
                    CardKind::Basic {
                        front: format!("n{i}"),
                        back: "x".into(),
                    },
                    vec![],
                    now(),
                )
                .unwrap(),
            );
        }

        let counts = l.due_counts(Scope::All, now());
        assert_eq!(counts.review, 1);
        assert_eq!(counts.learning, 1);
        assert_eq!(counts.new_cards, 3);

        let session = l.build_session(Scope::All, now()).unwrap();
        assert_eq!(session.queue.len(), 4); // learning + review + 2 new
        assert_eq!(session.queue[0].1, QueueKind::Learning);
        assert_eq!(session.queue[1].1, QueueKind::Review);
        assert_eq!(session.queue[2].0, fresh_ids[0]);
        assert_eq!(session.queue[3].0, fresh_ids[1]);
    }

    #[test]
    fn session_on_empty_library_is_an_error() {
        let l = lib();
        assert!(matches!(
            l.build_session(Scope::All, now()),
            Err(LibraryError::NothingToStudy)
        ));
    }

    #[test]
    fn scope_respects_subtree() {
        let mut l = lib();
        let root = l.decks[0].id;
        let a = l.new_deck(Some(root), "A", None, now()).unwrap();
        let b = l.new_deck(Some(a), "B", None, now()).unwrap();
        l.add_card(
            b,
            CardKind::Basic {
                front: "in b".into(),
                back: "x".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
        l.add_card(
            root,
            CardKind::Basic {
                front: "in root".into(),
                back: "x".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
        assert_eq!(l.cards_in_scope(Scope::Deck(b)).len(), 1);
        assert_eq!(l.cards_in_scope(Scope::Deck(a)).len(), 1);
        assert_eq!(l.cards_in_scope(Scope::All).len(), 2);
    }

    #[test]
    fn search_matches_text_tags_and_scope() {
        let mut l = lib();
        let root = l.decks[0].id;
        let other = l.new_deck(None, "Other", None, now()).unwrap();
        let c1 = l
            .add_card(
                root,
                CardKind::Basic {
                    front: "What is ATP?".into(),
                    back: "energy".into(),
                },
                vec!["bio".into()],
                now(),
            )
            .unwrap();
        l.add_card(
            other,
            CardKind::Basic {
                front: "ATP synthase".into(),
                back: "enzyme".into(),
            },
            vec!["bio".into()],
            now(),
        )
        .unwrap();
        l.add_card(
            root,
            CardKind::Cloze {
                text: "The {{c1::hippocampus}} consolidates memory".into(),
            },
            vec!["neuro".into()],
            now(),
        )
        .unwrap();

        let f = CardFilter {
            deck: Some(root),
            tags: vec!["bio".into()],
            query: "atp".into(),
        };
        assert_eq!(l.search(&f), vec![c1]);

        let f2 = CardFilter {
            query: "hippo".into(),
            ..Default::default()
        };
        assert_eq!(l.search(&f2).len(), 1);

        let f3 = CardFilter {
            tags: vec!["bio".into(), "neuro".into()],
            ..Default::default()
        };
        assert!(l.search(&f3).is_empty()); // AND semantics
    }

    #[test]
    fn deck_summaries_aggregate_subtrees() {
        let mut l = lib();
        let root = l.decks[0].id;
        let sub = l.new_deck(Some(root), "Sub", None, now()).unwrap();
        l.add_card(
            sub,
            CardKind::Basic {
                front: "f".into(),
                back: "b".into(),
            },
            vec![],
            now(),
        )
        .unwrap();
        let summaries = l.deck_summaries(now());
        let root_summary = summaries.iter().find(|s| s.id == root).unwrap();
        assert_eq!(root_summary.card_count, 1);
        assert_eq!(root_summary.due_count, 0);
        let sub_summary = summaries.iter().find(|s| s.id == sub).unwrap();
        assert_eq!(sub_summary.card_count, 1);
    }

    #[test]
    fn corrupted_parent_cycles_terminate() {
        let mut l = lib();
        let root = l.decks[0].id;
        let a = l.new_deck(Some(root), "A", None, now()).unwrap();
        // Forge a self-parent cycle, as a hand-edited file could.
        l.decks.iter_mut().find(|d| d.id == a).unwrap().parent = Some(a);
        // All walks must terminate and stay sane.
        assert_eq!(l.descendant_ids(a), vec![a]);
        assert!(l.deck_path(a).is_ok());
        assert!(!l.is_ancestor(a, root));
        // The library is still repairable: re-parent the cycle away.
        assert!(l.move_deck(a, Some(root)).is_ok());
        assert_eq!(l.deck(a).unwrap().parent, Some(root));
    }

    #[test]
    fn library_serde_roundtrip() {
        let mut l = lib();
        let root = l.decks[0].id;
        let c = l
            .add_card(
                root,
                CardKind::Basic {
                    front: "f".into(),
                    back: "b".into(),
                },
                vec!["t".into()],
                now(),
            )
            .unwrap();
        l.record_review(c, Grade::Good, now()).unwrap();
        let json = serde_json::to_string(&l).unwrap();
        let back: Library = serde_json::from_str(&json).unwrap();
        assert_eq!(back, l);
        assert_eq!(back.schema_version, SCHEMA_VERSION);
    }
}
