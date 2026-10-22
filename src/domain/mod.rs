//! Domain module re-exports and common types.
//!
//! Everything in `domain` is pure Rust: no UI crates, no I/O, no panics.
//! Scheduling and library mutations are command-centric and deterministic;
//! time is always injected by the caller (never `Utc::now()` inside domain
//! logic) so every path is testable headlessly.

pub mod aigen;
pub mod card;
pub mod deck;
pub mod library;
pub mod scheduler;
pub mod session;
pub mod stats;
pub mod theme;

pub use aigen::{generate_drafts, DraftCard, DraftKind};
pub use card::{
    collect_cloze_numbers, count_cloze_groups, render_cloze_blank, render_cloze_revealed, Card,
    CardError, CardId, CardKind, CARD_ID_NIL,
};
pub use deck::{Deck, DeckColor, DeckError, DeckId, DeckSummary};
pub use library::{
    CardFilter, Library, LibraryError, LibrarySettings, QueueKind, ReviewLog, ReviewOutcome, Scope,
    DEFAULT_DAILY_NEW_LIMIT, DEFAULT_DAILY_REVIEWS_LIMIT, SCHEMA_VERSION,
};
pub use scheduler::{
    format_interval, preview_interval, schedule, Grade, LearnPhase, SchedParams, SchedulingState,
    MAX_EASE, MIN_EASE,
};
pub use session::{AnsweredCard, SessionSummary, StudySession};
pub use stats::{DailyCount, DailyDue, LibraryStats};
pub use theme::Theme;
