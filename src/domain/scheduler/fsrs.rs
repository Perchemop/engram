//! FSRS: the three-variable spaced-repetition model.
//!
//! FSRS ("Free Spaced Repetition Scheduler") tracks three memory variables
//! per card:
//!
//! - **R** — retrievability: the probability the card is recalled right now.
//! - **D** — difficulty: how hard the card is, in `1.0..=10.0`.
//! - **S** — stability: the number of days until retrievability decays to
//!   the target retention (0.9 with the defaults).
//!
//! The forgetting curve is `R(t, S) = (1 + FACTOR * t / S)^DECAY` with
//! `DECAY = -0.5` and `FACTOR = 19 / 81`, so `R(S, S) = 0.9`. The interval
//! that brings a card back at a chosen target retention is the inverse
//! curve, `I(r, S) = S * (r^(1 / DECAY) - 1) / FACTOR`; at `r = 0.9` the
//! interval equals the stability.
//!
//! This is Engram's original implementation, written from the public
//! algorithm description (`docs/research/fsrs-scheduler.md`) using the
//! public default weight vector below. No reference-implementation source
//! was copied.
//!
//! Only *graduated interval computation* differs from SM-2: new-card
//! introduction, learning steps and lapse re-entry are shared, so a library
//! can switch algorithms without changing how cards are taught.

use chrono::{DateTime, Utc};

use super::{
    cap_interval, days, minutes, transition, FsrsParams, Grade, LearnPhase, PolicyContext,
    ReviewPolicy, ScheduleParams, SchedulingState, MAX_EASE, MIN_EASE,
};

/// Public default FSRS-4.5 weight vector (17 parameters), used verbatim.
///
/// Layout: `w[0..=3]` initial stability per grade (Again, Hard, Good, Easy),
/// `w[4..=7]` initial difficulty and its update/mean reversion,
/// `w[8..=10]` the recall-stability curve, `w[11..=14]` the lapse-stability
/// curve, `w[15]` the hard penalty and `w[16]` the easy bonus.
pub const DEFAULT_WEIGHTS: [f64; 17] = [
    0.4872, 1.4003, 3.7145, 13.8206, // initial stability per grade
    5.1618, 1.2298, 0.8975, 0.031, // initial difficulty and updates
    1.6474, 0.1367, 1.0461, // recall stability
    2.1072, 0.0793, 0.3246, 1.587, // lapse stability
    0.2272, 2.8755, // hard penalty, easy bonus
];

/// Default target retention: the probability of recall when a card is due.
pub const DEFAULT_TARGET_RETENTION: f64 = 0.9;

/// Decay constant of the forgetting curve.
pub const DECAY: f64 = -0.5;

/// Forgetting-curve scale, chosen so `R(S, S) = 0.9`.
pub const FACTOR: f64 = 19.0 / 81.0;

/// Lowest FSRS difficulty.
pub const MIN_DIFFICULTY: f64 = 1.0;

/// Highest FSRS difficulty.
pub const MAX_DIFFICULTY: f64 = 10.0;

/// Shortest interval FSRS may grant (days), matching the SM-2 floor.
pub const MIN_INTERVAL_DAYS: f64 = 0.1;

/// Zero-based position of `grade` in the public formulas.
fn grade_index(grade: Grade) -> usize {
    match grade {
        Grade::Again => 0,
        Grade::Hard => 1,
        Grade::Good => 2,
        Grade::Easy => 3,
    }
}

/// The public rating for `grade`: `Again = 1 … Easy = 4`.
fn rating(grade: Grade) -> f64 {
    (grade_index(grade) + 1) as f64
}

/// Initial stability in days for the first memory of a card graded `grade`.
#[must_use]
pub fn initial_stability(grade: Grade) -> f64 {
    DEFAULT_WEIGHTS[grade_index(grade)]
}

/// Initial difficulty for a card graded `grade`.
#[must_use]
pub fn initial_difficulty(grade: Grade) -> f64 {
    let w = &DEFAULT_WEIGHTS;
    let d = w[4] - (rating(grade) - 3.0) * w[5];
    d.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
}

/// Probability of recall after `elapsed_days` for a memory of `stability`.
///
/// Returns `0.0` when the stability is not positive (no memory yet).
#[must_use]
pub fn retrievability(elapsed_days: f64, stability: f64) -> f64 {
    if stability <= 0.0 || !stability.is_finite() {
        return 0.0;
    }
    let elapsed = if elapsed_days.is_finite() {
        elapsed_days.max(0.0)
    } else {
        0.0
    };
    (1.0 + FACTOR * elapsed / stability).powf(DECAY)
}

/// Interval in days that brings `stability` back at `target_retention`.
///
/// The result is at least [`MIN_INTERVAL_DAYS`]; the caller applies the
/// library's maximum-interval cap.
#[must_use]
pub fn next_interval_days(stability: f64, target_retention: f64) -> f64 {
    let interval = stability.max(0.0) * interval_factor(target_retention);
    if interval.is_finite() {
        interval.max(MIN_INTERVAL_DAYS)
    } else {
        MIN_INTERVAL_DAYS
    }
}

/// Multiplier turning a stability into an interval for `target_retention`.
///
/// At the default 0.9 the factor is 1.0; higher retention shortens the
/// interval and lower retention lengthens it.
fn interval_factor(target_retention: f64) -> f64 {
    let target = if target_retention.is_finite() {
        target_retention.clamp(FsrsParams::MIN_TARGET_RETENTION, FsrsParams::MAX_TARGET_RETENTION)
    } else {
        DEFAULT_TARGET_RETENTION
    };
    let factor = (target.powf(1.0 / DECAY) - 1.0) / FACTOR;
    if factor.is_finite() && factor > 0.0 {
        factor
    } else {
        1.0
    }
}

/// Difficulty after a review at `grade` (FSRS difficulty update with mean
/// reversion towards the easy-card starting point).
fn next_difficulty(difficulty: f64, grade: Grade) -> f64 {
    let w = &DEFAULT_WEIGHTS;
    let next = difficulty - w[6] * (rating(grade) - 3.0);
    let reverted = w[7] * initial_difficulty(Grade::Easy) + (1.0 - w[7]) * next;
    reverted.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
}

/// Stability after a successful recall (`grade != Again`).
fn recall_stability(difficulty: f64, stability: f64, retrievability: f64, grade: Grade) -> f64 {
    let w = &DEFAULT_WEIGHTS;
    let hard_penalty = if grade == Grade::Hard { w[15] } else { 1.0 };
    let easy_bonus = if grade == Grade::Easy { w[16] } else { 1.0 };
    let growth = w[8].exp()
        * (11.0 - difficulty)
        * stability.powf(-w[9])
        * ((1.0 - retrievability) * w[10]).exp_m1();
    let next = stability * (1.0 + growth * hard_penalty * easy_bonus);
    if next.is_finite() {
        next.max(MIN_INTERVAL_DAYS)
    } else {
        stability.max(MIN_INTERVAL_DAYS)
    }
}

/// Stability after a lapse. A lapse never lengthens the memory trace.
fn forget_stability(difficulty: f64, stability: f64, retrievability: f64) -> f64 {
    let w = &DEFAULT_WEIGHTS;
    let next = w[11]
        * difficulty.powf(-w[12])
        * ((stability + 1.0).powf(w[13]) - 1.0)
        * ((1.0 - retrievability) * w[14]).exp();
    let ceiling = stability.max(MIN_INTERVAL_DAYS);
    if next.is_finite() {
        next.clamp(MIN_INTERVAL_DAYS, ceiling)
    } else {
        ceiling
    }
}

/// Days elapsed since the card was last reviewed.
///
/// A review-phase card's `due` is `last_review + interval_days`, so the
/// elapsed time is exactly `interval_days + (now - due)`: this stays right
/// whether the card is answered early or late, and needs no extra clock
/// field in [`SchedulingState`].
fn elapsed_days(now: DateTime<Utc>, prev: &SchedulingState) -> f64 {
    let lateness = (now - prev.due).num_seconds() as f64 / 86_400.0;
    let elapsed = prev.interval_days + lateness;
    if elapsed.is_finite() {
        elapsed.max(0.0)
    } else {
        0.0
    }
}

/// The memory trace to start from: existing FSRS state, or a documented
/// derivation from the card's SM-2 fields on its first FSRS review.
fn memory(prev: &SchedulingState, target_retention: f64) -> (f64, f64) {
    if prev.fsrs_stability > 0.0 && prev.fsrs_stability.is_finite() {
        let difficulty = if prev.fsrs_difficulty.is_finite() {
            prev.fsrs_difficulty.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
        } else {
            initial_difficulty(Grade::Good)
        };
        (difficulty, prev.fsrs_stability)
    } else {
        seed_from_sm2(prev, target_retention)
    }
}

/// Approximate FSRS state for a card that was scheduled by SM-2.
///
/// SM-2 ease maps linearly onto FSRS difficulty (ease 1.3 → 10, ease 3.0 →
/// 1) and the current interval maps onto stability through the inverse
/// forgetting curve, so the card keeps its due date on the first FSRS
/// review (`docs/research/fsrs-scheduler.md`, "documented approximation").
fn seed_from_sm2(prev: &SchedulingState, target_retention: f64) -> (f64, f64) {
    let ease = if prev.ease.is_finite() {
        prev.ease.clamp(MIN_EASE, MAX_EASE)
    } else {
        MAX_EASE
    };
    let position = (MAX_EASE - ease) / (MAX_EASE - MIN_EASE);
    let difficulty =
        (MIN_DIFFICULTY + position * (MAX_DIFFICULTY - MIN_DIFFICULTY)).clamp(1.0, 10.0);
    let interval = if prev.interval_days.is_finite() {
        prev.interval_days.max(MIN_INTERVAL_DAYS)
    } else {
        MIN_INTERVAL_DAYS
    };
    let stability = (interval / interval_factor(target_retention)).max(MIN_INTERVAL_DAYS);
    (difficulty, stability)
}

/// The FSRS interval policy.
struct FsrsPolicy;

impl ReviewPolicy for FsrsPolicy {
    fn graduation(
        &self,
        ctx: &PolicyContext<'_>,
        next: &mut SchedulingState,
        grade: Grade,
    ) -> f64 {
        let target = ctx.fsrs.target_retention;
        let (difficulty, stability) =
            if next.fsrs_stability > 0.0 && next.fsrs_stability.is_finite() {
                // Graduating from relearning: the memory trace already exists.
                // Relearning steps are minutes long, so retrievability hardly
                // moves and stability is carried over; only difficulty follows
                // the graduating grade.
                let base = if next.fsrs_difficulty.is_finite() {
                    next.fsrs_difficulty.clamp(MIN_DIFFICULTY, MAX_DIFFICULTY)
                } else {
                    initial_difficulty(Grade::Good)
                };
                (next_difficulty(base, grade), next.fsrs_stability)
            } else {
                // First graduation: standard FSRS initial values for the grade.
                (initial_difficulty(grade), initial_stability(grade))
            };
        next.fsrs_difficulty = difficulty;
        next.fsrs_stability = stability;
        next.fsrs_retrievability = 1.0;
        cap_interval(next_interval_days(stability, target), ctx.sm2)
    }

    fn review(
        &self,
        now: DateTime<Utc>,
        ctx: &PolicyContext<'_>,
        prev: &SchedulingState,
        next: &mut SchedulingState,
        grade: Grade,
    ) {
        let params = ctx.sm2;
        let target = ctx.fsrs.target_retention;

        // 1. Memory before the grade, plus the recall probability the card
        //    had when it was answered.
        let (difficulty, stability) = memory(prev, target);
        let recall = retrievability(elapsed_days(now, prev), stability);

        // 2. Difficulty follows the grade; stability follows the outcome.
        next.fsrs_difficulty = next_difficulty(difficulty, grade);
        next.fsrs_stability = if grade == Grade::Again {
            forget_stability(difficulty, stability, recall)
        } else {
            recall_stability(difficulty, stability, recall, grade)
        };
        next.fsrs_retrievability = recall;

        // 3. Keep the SM-2 ease in step so a switch back to SM-2 remains
        //    sensible; FSRS itself does not read it.
        match grade {
            Grade::Again => next.ease -= params.ease_delta_again,
            Grade::Hard => next.ease -= params.ease_delta_hard,
            Grade::Good => {}
            Grade::Easy => next.ease += params.ease_delta_easy,
        }
        next.ease = next.ease.clamp(MIN_EASE, MAX_EASE);

        // 4. Same lapse flow as SM-2: relearning steps, then graduation.
        if grade == Grade::Again {
            next.lapses += 1;
            next.interval_days = params.graduating_interval_days;
            next.phase = LearnPhase::Relearning;
            next.step = 0;
            next.due = now + minutes(params.relearn_steps_minutes.first().copied().unwrap_or(10));
        } else {
            next.interval_days =
                cap_interval(next_interval_days(next.fsrs_stability, target), params);
            next.phase = LearnPhase::Review;
            next.step = 0;
            next.due = now + days(next.interval_days);
        }
    }
}

/// Apply one review grade with the FSRS model.
///
/// Pure and deterministic: `now` is injected and the public default weights
/// are constants, so the same inputs always produce the same state.
#[must_use]
pub fn schedule(
    now: DateTime<Utc>,
    params: &ScheduleParams,
    state: &SchedulingState,
    grade: Grade,
) -> SchedulingState {
    let sm2 = params.sm2.clone().sanitised();
    let fsrs = params.fsrs.sanitised();
    let ctx = PolicyContext {
        sm2: &sm2,
        fsrs: &fsrs,
    };
    transition(now, &ctx, state, grade, &FsrsPolicy)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::scheduler::{schedule as dispatch, SchedParams};
    use chrono::TimeZone;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    fn sm2() -> SchedParams {
        SchedParams::default()
    }

    fn params(target_retention: f64) -> ScheduleParams {
        ScheduleParams::fsrs(&sm2(), FsrsParams { target_retention })
    }

    fn new_state() -> SchedulingState {
        SchedulingState::new(now(), 2.5)
    }

    /// A graduated FSRS card with an existing memory trace, due at `now`.
    fn review_state(difficulty: f64, stability: f64, interval_days: f64) -> SchedulingState {
        SchedulingState {
            phase: LearnPhase::Review,
            step: 0,
            ease: 2.5,
            interval_days,
            due: now(),
            reps: 5,
            lapses: 0,
            fsrs_difficulty: difficulty,
            fsrs_stability: stability,
            fsrs_retrievability: 0.0,
        }
    }

    #[test]
    fn new_card_good_follows_the_sm2_learning_steps() {
        let sm2_params = ScheduleParams::from(&sm2());
        let fsrs_params = params(0.9);

        let first_sm2 = dispatch(now(), &sm2_params, &new_state(), Grade::Good);
        let first_fsrs = dispatch(now(), &fsrs_params, &new_state(), Grade::Good);
        assert_eq!(first_fsrs.phase, first_sm2.phase);
        assert_eq!(first_fsrs.step, first_sm2.step);
        assert_eq!(first_fsrs.due, first_sm2.due);

        let second_sm2 = dispatch(now(), &sm2_params, &first_sm2, Grade::Good);
        let second_fsrs = dispatch(now(), &fsrs_params, &first_fsrs, Grade::Good);
        assert_eq!(second_fsrs.phase, LearnPhase::Review);
        assert_eq!(second_fsrs.phase, second_sm2.phase);
        assert_eq!(second_fsrs.step, second_sm2.step);
        assert_eq!(
            second_fsrs.fsrs_stability,
            initial_stability(Grade::Good),
            "a first graduation uses the FSRS initial stability"
        );
    }

    #[test]
    fn new_card_easy_graduates_with_initial_easy_stability() {
        let s = schedule(now(), &params(0.9), &new_state(), Grade::Easy);
        assert_eq!(s.phase, LearnPhase::Review);
        assert!((s.fsrs_stability - DEFAULT_WEIGHTS[3]).abs() < 1e-9);
        assert!((s.fsrs_difficulty - initial_difficulty(Grade::Easy)).abs() < 1e-9);
        assert!(s.fsrs_retrievability > 0.99);
        assert!(
            (s.interval_days - DEFAULT_WEIGHTS[3]).abs() < 1e-6,
            "at 0.9 retention the interval equals the stability"
        );
    }

    #[test]
    fn initial_values_match_the_public_defaults() {
        let expected = [0.4872, 1.4003, 3.7145, 13.8206];
        for (grade, want) in Grade::ALL.iter().zip(expected) {
            assert!((initial_stability(*grade) - want).abs() < 1e-12);
            let d = initial_difficulty(*grade);
            assert!((MIN_DIFFICULTY..=MAX_DIFFICULTY).contains(&d));
        }
        assert!(initial_difficulty(Grade::Again) > initial_difficulty(Grade::Easy));
    }

    #[test]
    fn default_target_yields_documented_first_review_ballpark() {
        // The public description's first-interval examples with the default
        // weights: Easy ≈ 13.8 d, Good ≈ 3.7 d at 0.9 retention.
        assert!((next_interval_days(initial_stability(Grade::Easy), 0.9) - 13.82).abs() < 0.01);
        assert!((next_interval_days(initial_stability(Grade::Good), 0.9) - 3.71).abs() < 0.01);

        let easy = schedule(now(), &params(0.9), &new_state(), Grade::Easy);
        assert!((10.0..=15.0).contains(&easy.interval_days));

        let stepped = schedule(now(), &params(0.9), &new_state(), Grade::Good);
        let graduated = schedule(now(), &params(0.9), &stepped, Grade::Good);
        assert!((3.0..=5.0).contains(&graduated.interval_days));
    }

    #[test]
    fn review_intervals_increase_with_stability() {
        let p = params(0.9);
        let low = schedule(now(), &p, &review_state(5.0, 2.0, 2.0), Grade::Good);
        let high = schedule(now(), &p, &review_state(5.0, 10.0, 10.0), Grade::Good);
        assert!(high.interval_days > low.interval_days);
        assert!(high.fsrs_stability > low.fsrs_stability);
    }

    #[test]
    fn higher_target_retention_shortens_intervals() {
        let state = review_state(5.0, 5.0, 5.0);
        let relaxed = schedule(now(), &params(0.70), &state, Grade::Good);
        let strict = schedule(now(), &params(0.95), &state, Grade::Good);
        assert!(relaxed.interval_days > strict.interval_days);
        assert!(next_interval_days(5.0, 0.7) > next_interval_days(5.0, 0.9));
        assert!(next_interval_days(5.0, 0.95) < next_interval_days(5.0, 0.9));
    }

    #[test]
    fn retrievability_decays_and_is_0_9_at_stability() {
        assert!((retrievability(0.0, 5.0) - 1.0).abs() < 1e-12);
        assert!((retrievability(5.0, 5.0) - 0.9).abs() < 1e-12);
        assert!(retrievability(10.0, 5.0) < retrievability(5.0, 5.0));
        assert_eq!(retrievability(1.0, 0.0), 0.0);
        assert_eq!(retrievability(f64::NAN, 5.0), 1.0);
    }

    #[test]
    fn lapse_enters_relearning_like_sm2() {
        let before = review_state(5.0, 10.0, 10.0);
        let after = schedule(now(), &params(0.9), &before, Grade::Again);
        assert_eq!(after.phase, LearnPhase::Relearning);
        assert_eq!(after.step, 0);
        assert_eq!(after.lapses, before.lapses + 1);
        assert_eq!((after.due - now()).num_minutes(), 10);
        assert!(after.fsrs_stability < before.fsrs_stability);
        assert!(after.fsrs_retrievability > 0.0 && after.fsrs_retrievability < 1.0);
    }

    #[test]
    fn relearning_graduates_on_the_post_lapse_stability() {
        let lapsed = schedule(now(), &params(0.9), &review_state(5.0, 10.0, 10.0), Grade::Again);
        let stability = lapsed.fsrs_stability;
        let graduated = schedule(now(), &params(0.9), &lapsed, Grade::Good);
        assert_eq!(graduated.phase, LearnPhase::Review);
        assert!(
            (graduated.fsrs_stability - stability).abs() < 1e-9,
            "minutes-long relearning steps keep the post-lapse stability"
        );
        assert!((graduated.interval_days - next_interval_days(stability, 0.9)).abs() < 1e-6);
        assert!(graduated.interval_days < initial_stability(Grade::Good));
    }

    #[test]
    fn first_fsrs_review_of_an_sm2_card_derives_the_memory() {
        let before = review_state(0.0, 0.0, 10.0);
        let after = schedule(now(), &params(0.9), &before, Grade::Good);
        assert!(after.has_fsrs_memory());
        assert!((MIN_DIFFICULTY..=MAX_DIFFICULTY).contains(&after.fsrs_difficulty));
        assert!(after.fsrs_retrievability > 0.85 && after.fsrs_retrievability < 0.95);
        assert!(after.interval_days > before.interval_days);
    }

    #[test]
    fn intervals_are_capped_by_the_library_maximum() {
        let params = ScheduleParams::fsrs(
            &SchedParams {
                max_interval_days: 30.0,
                ..SchedParams::default()
            },
            FsrsParams::default(),
        );
        let s = schedule(now(), &params, &review_state(5.0, 10_000.0, 10_000.0), Grade::Easy);
        assert_eq!(s.interval_days, 30.0);
    }

    #[test]
    fn scheduling_is_deterministic() {
        let p = params(0.9);
        let state = review_state(4.0, 6.0, 6.0);
        let a = schedule(now(), &p, &state, Grade::Good);
        let b = schedule(now(), &p, &state, Grade::Good);
        assert_eq!(a, b);
        // Same state, different time -> different (but repeatable) result.
        let later = now() + chrono::Duration::days(3);
        let c = schedule(later, &p, &state, Grade::Good);
        let d = schedule(later, &p, &state, Grade::Good);
        assert_eq!(c, d);
        assert!(c.fsrs_retrievability < a.fsrs_retrievability);
    }

    #[test]
    fn fsrs_state_serde_roundtrip() {
        let state = schedule(now(), &params(0.9), &review_state(5.0, 5.0, 5.0), Grade::Good);
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("fsrs_stability"));
        let back: SchedulingState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state);
        assert_eq!(back.fsrs_difficulty, state.fsrs_difficulty);
        assert_eq!(back.fsrs_stability, state.fsrs_stability);
        assert_eq!(back.fsrs_retrievability, state.fsrs_retrievability);
    }

    #[test]
    fn fsrs_state_defaults_for_legacy_files() {
        let state = review_state(5.0, 5.0, 5.0);
        let mut value: serde_json::Value = serde_json::to_value(&state).unwrap();
        for key in ["fsrs_difficulty", "fsrs_stability", "fsrs_retrievability"] {
            value.as_object_mut().unwrap().remove(key);
        }
        let back: SchedulingState = serde_json::from_value(value).unwrap();
        assert_eq!(back.fsrs_difficulty, 0.0);
        assert_eq!(back.fsrs_stability, 0.0);
        assert_eq!(back.fsrs_retrievability, 0.0);
        assert!(!back.has_fsrs_memory());
    }

    #[test]
    fn interval_factor_is_one_at_default_retention() {
        assert!((interval_factor(0.9) - 1.0).abs() < 1e-9);
        assert!(interval_factor(0.7) > 1.0);
        assert!(interval_factor(0.95) < 1.0);
    }
}
