//! Study sessions: an ordered queue of cards for one sitting.
//!
//! A session is pure state: the UI pulls the current card, shows it, and
//! records answers. Session state is *not* persisted — the review log in
//! the library is the durable record.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::card::CardId;
use crate::domain::library::QueueKind;
use crate::domain::scheduler::Grade;

/// One answered card within a session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnsweredCard {
    pub card_id: CardId,
    pub queue_kind: QueueKind,
    pub grade: Grade,
    pub at: DateTime<Utc>,
}

/// End-of-session summary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionSummary {
    pub total: usize,
    pub again: usize,
    pub hard: usize,
    pub good: usize,
    pub easy: usize,
    /// Seconds from session start to the last answer.
    pub elapsed_seconds: i64,
}

impl SessionSummary {
    /// Share of answers that were not `Again`.
    #[must_use]
    pub fn retention(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.total - self.again) as f64 / self.total as f64
    }
}

/// An active study session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StudySession {
    /// `(card, queue kind)` in serve order.
    pub queue: Vec<(CardId, QueueKind)>,
    pub position: usize,
    pub answers: Vec<AnsweredCard>,
    pub started: DateTime<Utc>,
    pub finished: bool,
}

impl StudySession {
    /// Create a session over an ordered queue. Empty queues finish
    /// immediately.
    #[must_use]
    pub fn new(queue: Vec<(CardId, QueueKind)>, now: DateTime<Utc>) -> Self {
        let finished = queue.is_empty();
        Self {
            queue,
            position: 0,
            answers: Vec::new(),
            started: now,
            finished,
        }
    }

    /// Number of cards still to be answered.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.queue.len() - self.position
    }

    /// Total queue length.
    #[must_use]
    pub fn total(&self) -> usize {
        self.queue.len()
    }

    /// The card currently being studied.
    #[must_use]
    pub fn current(&self) -> Option<(CardId, QueueKind)> {
        self.queue.get(self.position).copied()
    }

    /// Record the grade for the current card and advance.
    ///
    /// Returns `false` when the session is already finished.
    pub fn answer(&mut self, grade: Grade, now: DateTime<Utc>) -> bool {
        if self.finished {
            return false;
        }
        let Some((card_id, queue_kind)) = self.current() else {
            self.finished = true;
            return false;
        };
        self.answers.push(AnsweredCard {
            card_id,
            queue_kind,
            grade,
            at: now,
        });
        self.position += 1;
        if self.position >= self.queue.len() {
            self.finished = true;
        }
        true
    }

    /// End the session early, keeping the answers recorded so far.
    pub fn end(&mut self) {
        self.finished = true;
    }

    /// Summary of the answers recorded so far.
    #[must_use]
    pub fn summary(&self) -> SessionSummary {
        let mut s = SessionSummary {
            total: self.answers.len(),
            again: 0,
            hard: 0,
            good: 0,
            easy: 0,
            elapsed_seconds: 0,
        };
        for a in &self.answers {
            match a.grade {
                Grade::Again => s.again += 1,
                Grade::Hard => s.hard += 1,
                Grade::Good => s.good += 1,
                Grade::Easy => s.easy += 1,
            }
        }
        if let Some(last) = self.answers.last() {
            s.elapsed_seconds = (last.at - self.started).num_seconds().max(0);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::library::QueueKind;
    use chrono::TimeZone;
    use uuid::Uuid;

    fn t(minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, minute).unwrap()
    }

    fn queue(n: usize) -> Vec<(Uuid, QueueKind)> {
        (0..n)
            .map(|i| (Uuid::from_u128(i as u128), QueueKind::Review))
            .collect()
    }

    #[test]
    fn session_walks_the_queue() {
        let mut s = StudySession::new(queue(3), t(0));
        assert_eq!(s.remaining(), 3);
        assert_eq!(s.current().unwrap().0, Uuid::from_u128(0));
        assert!(s.answer(Grade::Good, t(1)));
        assert_eq!(s.remaining(), 2);
        assert!(s.answer(Grade::Again, t(2)));
        assert!(s.answer(Grade::Easy, t(3)));
        assert!(s.finished);
        assert!(!s.answer(Grade::Good, t(4)));
        let sum = s.summary();
        assert_eq!(sum.total, 3);
        assert_eq!(sum.again, 1);
        assert!((sum.retention() - 2.0 / 3.0).abs() < 1e-9);
        assert_eq!(sum.elapsed_seconds, 180);
    }

    #[test]
    fn empty_queue_finishes_immediately() {
        let s = StudySession::new(vec![], t(0));
        assert!(s.finished);
        assert_eq!(s.total(), 0);
    }

    #[test]
    fn end_early_keeps_summary() {
        let mut s = StudySession::new(queue(5), t(0));
        s.answer(Grade::Hard, t(1));
        s.end();
        assert!(s.finished);
        assert_eq!(s.summary().total, 1);
    }

    #[test]
    fn session_serde_roundtrip() {
        let mut s = StudySession::new(queue(2), t(0));
        s.answer(Grade::Good, t(1));
        let json = serde_json::to_string(&s).unwrap();
        let back: StudySession = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }
}
