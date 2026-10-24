//! Library statistics: activity heatmap, retention, maturity and forecast.
//!
//! All computations are pure functions of the library plus an injected
//! `now`, so they are deterministic and testable.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::library::Library;
use crate::domain::scheduler::{Grade, LearnPhase};

/// Reviews recorded on one calendar day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyCount {
    pub date: NaiveDate,
    pub reviews: u32,
    /// Reviews graded better than *Again*.
    pub correct: u32,
}

/// Cards due on one upcoming day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyDue {
    pub date: NaiveDate,
    pub due: u32,
}

/// Aggregated statistics over the whole library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_cards: usize,
    pub new_cards: usize,
    pub learning_cards: usize,
    pub review_cards: usize,
    pub mature_cards: usize,
    pub reviews_today: u32,
    pub correct_today: u32,
    /// Reviews answered better than *Again* / all reviews, over the last
    /// 30 days (`None` when no reviews exist).
    pub retention_30d: Option<f64>,
    /// Consecutive days with at least one review, ending today or yesterday.
    pub streak_days: u32,
    /// The last `window` days of review activity, oldest first.
    pub activity: Vec<DailyCount>,
    /// The next `forecast_days` days of due work, starting today.
    pub forecast: Vec<DailyDue>,
}

impl LibraryStats {
    /// Window length of the activity series.
    pub const ACTIVITY_WINDOW_DAYS: usize = 35;
    /// Forecast horizon.
    pub const FORECAST_DAYS: usize = 14;
}

/// Compute full-library statistics.
#[must_use]
pub fn compute(lib: &Library, now: DateTime<Utc>, params_mature_days: f64) -> LibraryStats {
    let today = now.date_naive();

    let mut new_cards = 0usize;
    let mut learning_cards = 0usize;
    let mut review_cards = 0usize;
    let mut mature_cards = 0usize;
    for c in &lib.cards {
        match c.sched.phase {
            LearnPhase::New => new_cards += 1,
            LearnPhase::Learning | LearnPhase::Relearning => learning_cards += 1,
            LearnPhase::Review => {
                review_cards += 1;
                if c.sched.interval_days >= params_mature_days {
                    mature_cards += 1;
                }
            }
        }
    }

    // Reviews per day: build a date → (total, correct) map.
    let mut by_day: std::collections::HashMap<NaiveDate, (u32, u32)> =
        std::collections::HashMap::new();
    for r in &lib.reviews {
        let day = r.at.date_naive();
        let e = by_day.entry(day).or_insert((0, 0));
        e.0 += 1;
        if r.grade != Grade::Again {
            e.1 += 1;
        }
    }

    let reviews_today = by_day.get(&today).copied().unwrap_or((0, 0));

    // 30-day retention.
    let cutoff = today - chrono::Duration::days(30);
    let (mut recent, mut recent_correct) = (0u32, 0u32);
    for (&day, &(total, correct)) in &by_day {
        if day > cutoff {
            recent += total;
            recent_correct += correct;
        }
    }
    let retention_30d = if recent == 0 {
        None
    } else {
        Some(f64::from(recent_correct) / f64::from(recent))
    };

    // Activity window, oldest → newest, zero-filled.
    let mut activity = Vec::with_capacity(LibraryStats::ACTIVITY_WINDOW_DAYS);
    for i in (0..LibraryStats::ACTIVITY_WINDOW_DAYS).rev() {
        let day = today - chrono::Duration::days(i as i64);
        let (reviews, correct) = by_day.get(&day).copied().unwrap_or((0, 0));
        activity.push(DailyCount {
            date: day,
            reviews,
            correct,
        });
    }

    // Streak: walk back from today (or yesterday if today is empty).
    let mut streak = 0u32;
    let mut cursor = if by_day.contains_key(&today) {
        today
    } else {
        today - chrono::Duration::days(1)
    };
    while by_day.contains_key(&cursor) {
        streak += 1;
        cursor -= chrono::Duration::days(1);
    }

    // Forecast: due dates of non-new cards bucketed per day within the
    // horizon (overdue cards count today). New cards are not forecast —
    // they enter the queue via the daily new limit instead.
    let horizon_end = today + chrono::Duration::days(LibraryStats::FORECAST_DAYS as i64 - 1);
    let mut buckets: std::collections::HashMap<NaiveDate, u32> = std::collections::HashMap::new();
    for c in &lib.cards {
        if c.sched.phase == LearnPhase::New {
            continue;
        }
        if c.is_blocked(now) {
            continue; // suspended / buried cards are not upcoming work
        }
        let due = c.sched.due.date_naive();
        if due > horizon_end {
            continue;
        }
        let day = due.max(today);
        *buckets.entry(day).or_insert(0) += 1;
    }
    let mut forecast = Vec::with_capacity(LibraryStats::FORECAST_DAYS);
    for i in 0..LibraryStats::FORECAST_DAYS {
        let day = today + chrono::Duration::days(i as i64);
        forecast.push(DailyDue {
            date: day,
            due: buckets.get(&day).copied().unwrap_or(0),
        });
    }

    LibraryStats {
        total_cards: lib.cards.len(),
        new_cards,
        learning_cards,
        review_cards,
        mature_cards,
        reviews_today: reviews_today.0,
        correct_today: reviews_today.1,
        retention_30d,
        streak_days: streak,
        activity,
        forecast,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::CardKind;
    use crate::domain::library::QueueKind;
    use crate::domain::scheduler::SchedulingState;
    use chrono::TimeZone;

    fn at(day: u32, hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, day, hour, 0, 0).unwrap()
    }

    fn lib_with_card(now: DateTime<Utc>) -> (Library, uuid::Uuid) {
        let mut l = Library::new("S", now);
        let deck = l.decks[0].id;
        let c = l
            .add_card(
                deck,
                CardKind::Basic {
                    front: "f".into(),
                    back: "b".into(),
                },
                vec![],
                now,
            )
            .unwrap();
        (l, c)
    }

    #[test]
    fn empty_library_has_zero_counts_and_no_streak() {
        let l = Library::new("S", at(15, 12));
        let s = compute(&l, at(15, 12), 21.0);
        assert_eq!(s.total_cards, 0);
        assert_eq!(s.activity.len(), LibraryStats::ACTIVITY_WINDOW_DAYS);
        assert_eq!(s.forecast.len(), LibraryStats::FORECAST_DAYS);
        assert_eq!(s.streak_days, 0);
        assert_eq!(s.retention_30d, None);
    }

    #[test]
    fn phases_are_counted() {
        let (mut l, c) = lib_with_card(at(15, 12));
        l.card_mut(c).unwrap().sched.phase = LearnPhase::Review;
        l.card_mut(c).unwrap().sched.interval_days = 30.0;
        let s = compute(&l, at(15, 12), 21.0);
        assert_eq!(s.review_cards, 1);
        assert_eq!(s.mature_cards, 1);
        assert_eq!(s.new_cards, 0);

        l.card_mut(c).unwrap().sched = SchedulingState::new(at(15, 12), 2.5);
        let s2 = compute(&l, at(15, 12), 21.0);
        assert_eq!(s2.new_cards, 1);
        assert_eq!(s2.mature_cards, 0);
    }

    #[test]
    fn streak_counts_consecutive_days() {
        let (mut l, c) = lib_with_card(at(15, 12));
        // Reviews on the 13th, 14th and 15th → streak 3 ending today.
        for day in [13u32, 14, 15] {
            l.record_review(c, Grade::Good, at(day, 9), QueueKind::Review)
                .unwrap();
        }
        let s = compute(&l, at(15, 12), 21.0);
        assert_eq!(s.streak_days, 3);
        assert_eq!(s.reviews_today, 1);
        assert_eq!(s.correct_today, 1);
    }

    #[test]
    fn streak_survives_an_empty_today() {
        let (mut l, c) = lib_with_card(at(15, 12));
        for day in [13u32, 14] {
            l.record_review(c, Grade::Good, at(day, 9), QueueKind::Review)
                .unwrap();
        }
        let s = compute(&l, at(15, 12), 21.0);
        assert_eq!(s.streak_days, 2);
    }

    #[test]
    fn retention_excludes_again_grades() {
        let (mut l, c) = lib_with_card(at(15, 12));
        l.record_review(c, Grade::Again, at(14, 9), QueueKind::Review)
            .unwrap();
        l.record_review(c, Grade::Good, at(14, 10), QueueKind::Review)
            .unwrap();
        l.record_review(c, Grade::Good, at(14, 11), QueueKind::Review)
            .unwrap();
        let s = compute(&l, at(15, 12), 21.0);
        assert_eq!(s.retention_30d, Some(2.0 / 3.0));
    }

    #[test]
    fn forecast_buckets_due_cards_by_day() {
        let (mut l, c) = lib_with_card(at(15, 12));
        {
            let card = l.card_mut(c).unwrap();
            card.sched.phase = LearnPhase::Review;
            card.sched.interval_days = 10.0;
            card.sched.due = at(20, 8);
        }
        let s = compute(&l, at(15, 12), 21.0);
        let day20 = s
            .forecast
            .iter()
            .find(|d| d.date == NaiveDate::from_ymd_opt(2026, 1, 20).unwrap())
            .unwrap();
        assert_eq!(day20.due, 1);
        let day16 = s
            .forecast
            .iter()
            .find(|d| d.date == NaiveDate::from_ymd_opt(2026, 1, 16).unwrap())
            .unwrap();
        assert_eq!(day16.due, 0);
    }

    #[test]
    fn stats_serde_roundtrip() {
        let (mut l, c) = lib_with_card(at(15, 12));
        l.record_review(c, Grade::Good, at(15, 12), QueueKind::Review)
            .unwrap();
        let s = compute(&l, at(15, 12), 21.0);
        let json = serde_json::to_string(&s).unwrap();
        let back: LibraryStats = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }
}
