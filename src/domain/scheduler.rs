//! Spaced-repetition schedulers.
//!
//! Engram ships two interchangeable policies behind the [`Scheduler`] trait:
//!
//! - **SuperMemo-2 (SM-2)**, the default: the publicly documented algorithm
//!   published by Piotr Wozniak in 1990, extended with learning steps, an
//!   ease clamp, an easy bonus and a maximum interval.
//! - **FSRS**, the three-variable model implemented in [`fsrs`]:
//!   retrievability, difficulty and stability
//!   (`docs/research/fsrs-scheduler.md`).
//!
//! Both policies share the same four grades, the same learning/relearning
//! step flow and the same lapse handling; only the graduated interval
//! computation differs. Every scheduler is a pure function of
//! `(state, grade, now, params)`: it never reads the clock and never uses
//! randomness, so the same inputs always produce the same schedule.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

pub mod fsrs;

use fsrs::DEFAULT_TARGET_RETENTION;

/// Lowest allowed ease factor.
pub const MIN_EASE: f64 = 1.3;
/// Highest allowed ease factor.
pub const MAX_EASE: f64 = 3.0;

/// Self-assessed recall quality for one review.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Grade {
    /// Complete blackout — the card lapses.
    Again,
    /// Recalled with serious difficulty.
    Hard,
    /// Recalled correctly, with effort.
    Good,
    /// Recalled instantly.
    Easy,
}

impl Grade {
    /// All grades in keyboard order (1–4).
    pub const ALL: [Grade; 4] = [Grade::Again, Grade::Hard, Grade::Good, Grade::Easy];

    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Again => "Again",
            Self::Hard => "Hard",
            Self::Good => "Good",
            Self::Easy => "Easy",
        }
    }

    /// Map a keyboard digit (1–4) to a grade.
    #[must_use]
    pub fn from_digit(n: u32) -> Option<Self> {
        match n {
            1 => Some(Self::Again),
            2 => Some(Self::Hard),
            3 => Some(Self::Good),
            4 => Some(Self::Easy),
            _ => None,
        }
    }
}

/// Where a card sits in the learning lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LearnPhase {
    /// Not yet studied; the queue decides when it is introduced.
    New,
    /// Short-interval learning steps after (re)introduction.
    Learning,
    /// Graduated; intervals measured in days.
    Review,
    /// Relearning steps after a lapse; graduates back to [`LearnPhase::Review`].
    Relearning,
}

impl LearnPhase {
    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::New => "New",
            Self::Learning => "Learning",
            Self::Review => "Review",
            Self::Relearning => "Relearning",
        }
    }
}

/// Which spaced-repetition algorithm a library uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SchedulerKind {
    /// SuperMemo-2 with learning steps — the 0.1.0 behaviour (default).
    #[default]
    Sm2,
    /// FSRS three-variable model (retrievability, difficulty, stability).
    Fsrs,
}

impl SchedulerKind {
    /// Human-readable label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Sm2 => "SM-2",
            Self::Fsrs => "FSRS",
        }
    }
}

/// FSRS tuning parameters.
///
/// Per-user weight optimisation is out of scope for this version: the public
/// default weight vector ([`fsrs::DEFAULT_WEIGHTS`]) is always used.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FsrsParams {
    /// Desired probability of recall when a review card becomes due
    /// (0.70–0.95).
    pub target_retention: f64,
}

impl Default for FsrsParams {
    fn default() -> Self {
        Self {
            target_retention: DEFAULT_TARGET_RETENTION,
        }
    }
}

impl FsrsParams {
    /// Lowest selectable target retention.
    pub const MIN_TARGET_RETENTION: f64 = 0.70;
    /// Highest selectable target retention.
    pub const MAX_TARGET_RETENTION: f64 = 0.95;

    /// Clamp into the supported 0.70–0.95 band. Non-finite input falls back
    /// to the default so a corrupt file can never break scheduling.
    #[must_use]
    pub fn sanitised(self) -> Self {
        let target_retention = if self.target_retention.is_finite() {
            self.target_retention
                .clamp(Self::MIN_TARGET_RETENTION, Self::MAX_TARGET_RETENTION)
        } else {
            DEFAULT_TARGET_RETENTION
        };
        Self { target_retention }
    }
}

/// Tunable SM-2 scheduler parameters (also drives the shared learning steps).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SchedParams {
    /// Minutes between learning steps for new cards.
    pub learning_steps_minutes: Vec<u32>,
    /// Minutes between relearning steps after a lapse.
    pub relearn_steps_minutes: Vec<u32>,
    /// Interval granted when a card graduates from learning.
    pub graduating_interval_days: f64,
    /// Interval granted when a card graduates via an *Easy* grade.
    pub easy_interval_days: f64,
    /// Ease factor assigned to new cards.
    pub starting_ease: f64,
    /// Multiplier applied on top of the computed interval for *Easy*.
    pub easy_bonus: f64,
    /// Multiplier applied to the interval for *Hard*.
    pub hard_factor: f64,
    /// Global multiplier on computed review intervals.
    pub interval_modifier: f64,
    /// Ease penalty for a lapse.
    pub ease_delta_again: f64,
    /// Ease penalty for *Hard*.
    pub ease_delta_hard: f64,
    /// Ease reward for *Easy*.
    pub ease_delta_easy: f64,
    /// Interval after which a card counts as mature (statistics only).
    pub mature_threshold_days: f64,
    /// Upper bound on scheduled intervals (days).
    pub max_interval_days: f64,
}

impl Default for SchedParams {
    fn default() -> Self {
        Self {
            learning_steps_minutes: vec![1, 10],
            relearn_steps_minutes: vec![10],
            graduating_interval_days: 1.0,
            easy_interval_days: 4.0,
            starting_ease: 2.5,
            easy_bonus: 1.3,
            hard_factor: 1.2,
            interval_modifier: 1.0,
            ease_delta_again: 0.20,
            ease_delta_hard: 0.15,
            ease_delta_easy: 0.15,
            mature_threshold_days: 21.0,
            max_interval_days: 36_500.0,
        }
    }
}

impl SchedParams {
    /// Clamp every parameter into a sane range so corrupt files or wild UI
    /// input can never produce non-positive intervals.
    #[must_use]
    pub fn sanitised(mut self) -> Self {
        self.graduating_interval_days = clampf(self.graduating_interval_days, 1.0, 365.0);
        self.easy_interval_days = clampf(self.easy_interval_days, 1.0, 365.0);
        self.starting_ease = clampf(self.starting_ease, MIN_EASE, MAX_EASE);
        self.easy_bonus = clampf(self.easy_bonus, 1.0, 3.0);
        self.hard_factor = clampf(self.hard_factor, 1.0, 2.0);
        self.interval_modifier = clampf(self.interval_modifier, 0.1, 10.0);
        self.ease_delta_again = clampf(self.ease_delta_again, 0.0, 1.0);
        self.ease_delta_hard = clampf(self.ease_delta_hard, 0.0, 1.0);
        self.ease_delta_easy = clampf(self.ease_delta_easy, 0.0, 1.0);
        self.mature_threshold_days = clampf(self.mature_threshold_days, 1.0, 365.0);
        self.max_interval_days = clampf(self.max_interval_days, 30.0, 100_000.0);
        self
    }
}

fn clampf(v: f64, lo: f64, hi: f64) -> f64 {
    if v.is_finite() {
        v.clamp(lo, hi)
    } else {
        lo
    }
}

/// Complete scheduler configuration for one library.
///
/// Bundles the algorithm selection with the parameters of both engines so
/// queue, session and statistics code can stay scheduler-agnostic. Build it
/// from [`crate::domain::LibrarySettings::schedule_params`] in application
/// code, or with [`ScheduleParams::sm2_only`] / [`ScheduleParams::fsrs`] in
/// tests.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ScheduleParams {
    /// Which algorithm is active.
    pub kind: SchedulerKind,
    /// SuperMemo-2 parameters. The learning-step lists are shared by both
    /// engines, so they live here.
    pub sm2: SchedParams,
    /// FSRS parameters (only used when `kind` is [`SchedulerKind::Fsrs`]).
    pub fsrs: FsrsParams,
}

impl ScheduleParams {
    /// Configuration that schedules with SM-2 only (the 0.1.0 behaviour).
    #[must_use]
    pub fn sm2_only(params: &SchedParams) -> Self {
        Self {
            kind: SchedulerKind::Sm2,
            sm2: params.clone(),
            fsrs: FsrsParams::default(),
        }
    }

    /// Configuration that schedules with FSRS.
    #[must_use]
    pub fn fsrs(params: &SchedParams, fsrs: FsrsParams) -> Self {
        Self {
            kind: SchedulerKind::Fsrs,
            sm2: params.clone(),
            fsrs,
        }
    }
}

impl From<&SchedParams> for ScheduleParams {
    fn from(params: &SchedParams) -> Self {
        Self::sm2_only(params)
    }
}

/// A pluggable spaced-repetition policy.
///
/// Implementations must be pure: the same `(params, state, grade, now)`
/// always yields the same next state — no clocks, no randomness.
pub trait Scheduler {
    /// Apply one review grade to `state`, returning the next state.
    #[must_use]
    fn schedule(
        &self,
        now: DateTime<Utc>,
        params: &ScheduleParams,
        state: &SchedulingState,
        grade: Grade,
    ) -> SchedulingState;
}

/// The SuperMemo-2 policy (Engram's 0.1.0 behaviour).
#[derive(Debug, Clone, Copy, Default)]
pub struct Sm2Scheduler;

/// The FSRS policy.
#[derive(Debug, Clone, Copy, Default)]
pub struct FsrsScheduler;

static SM2_SCHEDULER: Sm2Scheduler = Sm2Scheduler;
static FSRS_SCHEDULER: FsrsScheduler = FsrsScheduler;

/// The implementation for `kind`.
#[must_use]
pub fn scheduler_for(kind: SchedulerKind) -> &'static dyn Scheduler {
    match kind {
        SchedulerKind::Sm2 => &SM2_SCHEDULER,
        SchedulerKind::Fsrs => &FSRS_SCHEDULER,
    }
}

impl Scheduler for Sm2Scheduler {
    fn schedule(
        &self,
        now: DateTime<Utc>,
        params: &ScheduleParams,
        state: &SchedulingState,
        grade: Grade,
    ) -> SchedulingState {
        let sm2 = params.sm2.clone().sanitised();
        let ctx = PolicyContext {
            sm2: &sm2,
            fsrs: &params.fsrs,
        };
        transition(now, &ctx, state, grade, &Sm2Policy)
    }
}

impl Scheduler for FsrsScheduler {
    fn schedule(
        &self,
        now: DateTime<Utc>,
        params: &ScheduleParams,
        state: &SchedulingState,
        grade: Grade,
    ) -> SchedulingState {
        fsrs::schedule(now, params, state, grade)
    }
}

/// The scheduling state of a single card.
///
/// The SM-2 fields (`ease`, `interval_days`, …) are always maintained; the
/// `fsrs_*` fields carry the three-variable model for FSRS libraries and
/// stay at `0.0` for SM-2 cards. Migration is therefore lossless in both
/// directions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchedulingState {
    pub phase: LearnPhase,
    /// Index into the current step list (learning / relearning).
    pub step: usize,
    pub ease: f64,
    /// Current interval for graduated cards (days).
    pub interval_days: f64,
    pub due: DateTime<Utc>,
    pub reps: u32,
    pub lapses: u32,
    /// FSRS difficulty `D` in `1.0..=10.0`; `0.0` means "no FSRS memory".
    #[serde(default)]
    pub fsrs_difficulty: f64,
    /// FSRS stability `S` in days; `0.0` means "no FSRS memory".
    #[serde(default)]
    pub fsrs_stability: f64,
    /// FSRS retrievability `R` estimated at the last transition (0–1);
    /// `0.0` means "no FSRS memory".
    #[serde(default)]
    pub fsrs_retrievability: f64,
}

impl SchedulingState {
    /// Fresh state for a brand-new card.
    #[must_use]
    pub fn new(now: DateTime<Utc>, starting_ease: f64) -> Self {
        Self {
            phase: LearnPhase::New,
            step: 0,
            ease: starting_ease.clamp(MIN_EASE, MAX_EASE),
            interval_days: 0.0,
            due: now,
            reps: 0,
            lapses: 0,
            fsrs_difficulty: 0.0,
            fsrs_stability: 0.0,
            fsrs_retrievability: 0.0,
        }
    }

    /// True when a graduated/learning card's due time has passed.
    #[must_use]
    pub fn is_due(&self, now: DateTime<Utc>) -> bool {
        self.phase != LearnPhase::New && self.due <= now
    }

    /// True when the card counts as mature (statistics only).
    #[must_use]
    pub fn is_mature(&self, params: &SchedParams) -> bool {
        self.phase == LearnPhase::Review && self.interval_days >= params.mature_threshold_days
    }

    /// True when the card carries FSRS memory (stability has been computed).
    /// SM-2 cards and cards that have never graduated return `false`.
    #[must_use]
    pub fn has_fsrs_memory(&self) -> bool {
        self.fsrs_stability > 0.0
    }
}

/// Apply one review grade with the scheduler selected by `params.kind`.
///
/// Pure and deterministic: `now` is injected.
#[must_use]
pub fn schedule(
    now: DateTime<Utc>,
    params: &ScheduleParams,
    state: &SchedulingState,
    grade: Grade,
) -> SchedulingState {
    scheduler_for(params.kind).schedule(now, params, state, grade)
}

/// The parameters shared by both engines during one transition.
struct PolicyContext<'a> {
    sm2: &'a SchedParams,
    fsrs: &'a FsrsParams,
}

/// The scheduler-specific half of one transition.
///
/// New-card introduction, learning steps, graduation *timing* and lapse
/// re-entry are identical for every scheduler; a policy only decides which
/// interval a graduating or reviewed card receives, and may record
/// scheduler-specific state on the way.
trait ReviewPolicy {
    /// Interval (days) granted when a card graduates out of learning or
    /// relearning with `grade`.
    fn graduation(&self, ctx: &PolicyContext<'_>, next: &mut SchedulingState, grade: Grade) -> f64;

    /// Apply a review-phase grade. `prev` is the state before the grade and
    /// `next` an incremental clone of it.
    fn review(
        &self,
        now: DateTime<Utc>,
        ctx: &PolicyContext<'_>,
        prev: &SchedulingState,
        next: &mut SchedulingState,
        grade: Grade,
    );
}

/// Run the shared part of one review transition.
fn transition(
    now: DateTime<Utc>,
    ctx: &PolicyContext<'_>,
    state: &SchedulingState,
    grade: Grade,
    policy: &dyn ReviewPolicy,
) -> SchedulingState {
    let params = ctx.sm2;
    let mut s = state.clone();
    s.reps += 1;

    match state.phase {
        LearnPhase::New => match grade {
            Grade::Again | Grade::Hard => {
                enter_steps(&mut s, now, &params.learning_steps_minutes, 0);
            }
            Grade::Good => {
                if params.learning_steps_minutes.len() > 1 {
                    enter_steps(&mut s, now, &params.learning_steps_minutes, 1);
                } else {
                    graduate(&mut s, now, ctx, policy, grade);
                }
            }
            Grade::Easy => graduate(&mut s, now, ctx, policy, grade),
        },
        LearnPhase::Learning | LearnPhase::Relearning => {
            let steps = if state.phase == LearnPhase::Learning {
                &params.learning_steps_minutes
            } else {
                &params.relearn_steps_minutes
            };
            match grade {
                Grade::Again => {
                    // A failed learning step is not a lapse.
                    enter_steps(&mut s, now, steps, 0);
                }
                Grade::Hard => {
                    // Repeat the current step.
                    if steps.is_empty() {
                        graduate(&mut s, now, ctx, policy, grade);
                    } else {
                        let idx = state.step.min(steps.len() - 1);
                        enter_steps(&mut s, now, steps, idx);
                    }
                }
                Grade::Good => {
                    if state.step + 1 < steps.len() {
                        enter_steps(&mut s, now, steps, state.step + 1);
                    } else {
                        graduate(&mut s, now, ctx, policy, grade);
                    }
                }
                Grade::Easy => graduate(&mut s, now, ctx, policy, grade),
            }
        }
        LearnPhase::Review => policy.review(now, ctx, state, &mut s, grade),
    }
    s
}

fn enter_steps(s: &mut SchedulingState, now: DateTime<Utc>, steps: &[u32], idx: usize) {
    s.phase = if s.phase == LearnPhase::Relearning {
        LearnPhase::Relearning
    } else {
        LearnPhase::Learning
    };
    s.step = idx;
    let minutes_now = steps.get(idx).copied().unwrap_or(10);
    s.due = now + minutes(minutes_now);
}

fn graduate(
    s: &mut SchedulingState,
    now: DateTime<Utc>,
    ctx: &PolicyContext<'_>,
    policy: &dyn ReviewPolicy,
    grade: Grade,
) {
    let interval = policy.graduation(ctx, s, grade);
    s.phase = LearnPhase::Review;
    s.step = 0;
    s.interval_days = interval.max(0.1);
    s.due = now + days(s.interval_days);
}

fn cap_interval(d: f64, params: &SchedParams) -> f64 {
    d.min(params.max_interval_days).max(0.1)
}

fn days(d: f64) -> Duration {
    Duration::seconds((d * 24.0 * 3600.0).round() as i64)
}

fn minutes(m: u32) -> Duration {
    Duration::seconds(i64::from(m) * 60)
}

/// The SM-2 interval policy.
struct Sm2Policy;

impl ReviewPolicy for Sm2Policy {
    fn graduation(
        &self,
        ctx: &PolicyContext<'_>,
        _next: &mut SchedulingState,
        grade: Grade,
    ) -> f64 {
        match grade {
            Grade::Easy => ctx.sm2.easy_interval_days,
            _ => ctx.sm2.graduating_interval_days,
        }
    }

    fn review(
        &self,
        now: DateTime<Utc>,
        ctx: &PolicyContext<'_>,
        prev: &SchedulingState,
        s: &mut SchedulingState,
        grade: Grade,
    ) {
        let params = ctx.sm2;
        match grade {
            Grade::Again => s.ease -= params.ease_delta_again,
            Grade::Hard => s.ease -= params.ease_delta_hard,
            Grade::Good => {}
            Grade::Easy => s.ease += params.ease_delta_easy,
        }
        s.ease = s.ease.clamp(MIN_EASE, MAX_EASE);

        match grade {
            Grade::Again => {
                s.lapses += 1;
                s.interval_days = params.graduating_interval_days;
                s.phase = LearnPhase::Relearning;
                s.step = 0;
                s.due = now + minutes(params.relearn_steps_minutes.first().copied().unwrap_or(10));
            }
            Grade::Hard => {
                s.interval_days = cap_interval(
                    (prev.interval_days * params.hard_factor * params.interval_modifier)
                        .max(prev.interval_days + 1.0),
                    params,
                );
                s.phase = LearnPhase::Review;
                s.step = 0;
                s.due = now + days(s.interval_days);
            }
            Grade::Good | Grade::Easy => {
                let mut next = prev.interval_days * s.ease * params.interval_modifier;
                if grade == Grade::Easy {
                    next *= params.easy_bonus;
                }
                s.interval_days = cap_interval(next.max(prev.interval_days + 1.0), params);
                s.phase = LearnPhase::Review;
                s.step = 0;
                s.due = now + days(s.interval_days);
            }
        }
    }
}

/// Format an interval in days for display: `8m`, `1.5d`, `3.2mo`, `1.4y`.
#[must_use]
pub fn format_interval(days_value: f64) -> String {
    let d = days_value.max(0.0);
    if d < 1.0 / 24.0 {
        format!("{}m", (d * 24.0 * 60.0).round() as i64)
    } else if d < 1.0 {
        format!("{}h", (d * 24.0).round() as i64)
    } else if d < 31.0 {
        format!("{:.1}d", d).replace(".0d", "d")
    } else if d < 365.0 {
        format!("{:.1}mo", d / 30.44).replace(".0mo", "mo")
    } else {
        format!("{:.1}y", d / 365.25).replace(".0y", "y")
    }
}

/// The interval a grade *would* produce, as a display string. Used for the
/// grading buttons during review ("Again · 10m", "Good · 3.5d").
#[must_use]
pub fn preview_interval(
    now: DateTime<Utc>,
    params: &ScheduleParams,
    state: &SchedulingState,
    grade: Grade,
) -> String {
    let next = schedule(now, params, state, grade);
    match next.phase {
        LearnPhase::Learning | LearnPhase::Relearning => {
            let mins = (next.due - now).num_minutes().max(0);
            format!("{mins}m")
        }
        _ => format_interval(next.interval_days),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn params() -> SchedParams {
        SchedParams::default()
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 15, 12, 0, 0).unwrap()
    }

    fn new_state() -> SchedulingState {
        SchedulingState::new(now(), 2.5)
    }

    /// Schedule with the SM-2 parameters and the default (SM-2) selection.
    fn schedule_sm2(
        now: DateTime<Utc>,
        params: &SchedParams,
        state: &SchedulingState,
        grade: Grade,
    ) -> SchedulingState {
        schedule(now, &ScheduleParams::from(params), state, grade)
    }

    fn review_state(ease: f64, interval_days: f64, lapses: u32) -> SchedulingState {
        SchedulingState {
            phase: LearnPhase::Review,
            step: 0,
            ease,
            interval_days,
            due: now() - Duration::days(1),
            reps: 5,
            lapses,
            fsrs_difficulty: 0.0,
            fsrs_stability: 0.0,
            fsrs_retrievability: 0.0,
        }
    }

    #[test]
    fn new_card_good_enters_second_learning_step() {
        let p = params();
        let s = schedule_sm2(now(), &p, &new_state(), Grade::Good);
        assert_eq!(s.phase, LearnPhase::Learning);
        assert_eq!(s.step, 1);
        assert_eq!((s.due - now()).num_minutes(), 10);
    }

    #[test]
    fn new_card_easy_graduates_to_easy_interval() {
        let p = params();
        let s = schedule_sm2(now(), &p, &new_state(), Grade::Easy);
        assert_eq!(s.phase, LearnPhase::Review);
        assert_eq!(s.interval_days, p.easy_interval_days);
    }

    #[test]
    fn new_card_again_enters_first_learning_step() {
        let p = params();
        let s = schedule_sm2(now(), &p, &new_state(), Grade::Again);
        assert_eq!(s.phase, LearnPhase::Learning);
        assert_eq!(s.step, 0);
        assert_eq!((s.due - now()).num_minutes(), 1);
    }

    #[test]
    fn graduating_from_learning_uses_graduating_interval() {
        let p = params();
        // New → step 0 (Again) → step 1 (Good) → graduate (last-step Good).
        let stepped = schedule_sm2(now(), &p, &new_state(), Grade::Again);
        let stepped2 = schedule_sm2(now(), &p, &stepped, Grade::Good);
        let graduated = schedule_sm2(now(), &p, &stepped2, Grade::Good);
        assert_eq!(graduated.phase, LearnPhase::Review);
        assert_eq!(graduated.interval_days, p.graduating_interval_days);
        assert!((graduated.due - now()).num_hours() <= 24);
    }

    #[test]
    fn review_good_multiplies_by_ease() {
        let p = params();
        let base = review_state(2.5, 10.0, 0);
        let s = schedule_sm2(now(), &p, &base, Grade::Good);
        assert_eq!(s.phase, LearnPhase::Review);
        assert!((s.interval_days - 25.0).abs() < 1e-9);
        assert_eq!((s.due - now()).num_days(), 25);
    }

    #[test]
    fn review_easy_applies_easy_bonus() {
        let p = params();
        let base = review_state(2.5, 10.0, 0);
        let s = schedule_sm2(now(), &p, &base, Grade::Easy);
        // Ease is bumped before the interval multiply (SM-2 order), so the
        // effective ease for this interval is base.ease + delta.
        let expected = base.interval_days * (base.ease + p.ease_delta_easy) * p.easy_bonus;
        assert!((s.interval_days - expected).abs() < 1e-9);
    }

    #[test]
    fn review_hard_grows_by_hard_factor() {
        let p = params();
        let base = review_state(2.5, 10.0, 0);
        let s = schedule_sm2(now(), &p, &base, Grade::Hard);
        assert!((s.interval_days - 12.0).abs() < 1e-9);
        assert_eq!(s.ease, 2.5 - p.ease_delta_hard);
    }

    #[test]
    fn review_again_lapses_into_relearning() {
        let p = params();
        let base = review_state(2.5, 100.0, 0);
        let s = schedule_sm2(now(), &p, &base, Grade::Again);
        assert_eq!(s.phase, LearnPhase::Relearning);
        assert_eq!(s.lapses, 1);
        assert!((s.ease - 2.3).abs() < 1e-9);
        assert!((s.interval_days - p.graduating_interval_days).abs() < 1e-9);
        assert_eq!((s.due - now()).num_minutes(), 10);
    }

    #[test]
    fn ease_is_clamped() {
        let p = params();
        let base = review_state(MIN_EASE, 10.0, 3);
        let s = schedule_sm2(now(), &p, &base, Grade::Again);
        assert_eq!(s.ease, MIN_EASE);
        let mut maxed = base;
        maxed.ease = MAX_EASE;
        let s2 = schedule_sm2(now(), &p, &maxed, Grade::Easy);
        assert_eq!(s2.ease, MAX_EASE);
    }

    #[test]
    fn interval_is_capped() {
        let p = SchedParams {
            max_interval_days: 100.0,
            ..SchedParams::default()
        };
        let base = review_state(2.5, 90.0, 0);
        let s = schedule_sm2(now(), &p, &base, Grade::Good);
        assert_eq!(s.interval_days, 100.0);
    }

    #[test]
    fn sanitising_repairs_insane_parameters() {
        let p = SchedParams {
            graduating_interval_days: -5.0,
            starting_ease: 99.0,
            interval_modifier: f64::NAN,
            ..SchedParams::default()
        }
        .sanitised();
        assert_eq!(p.graduating_interval_days, 1.0);
        assert_eq!(p.starting_ease, MAX_EASE);
        assert_eq!(p.interval_modifier, 0.1);
    }

    #[test]
    fn format_interval_is_human_readable() {
        assert_eq!(format_interval(10.0 / (24.0 * 60.0)), "10m");
        assert_eq!(format_interval(0.5), "12h");
        assert_eq!(format_interval(1.0), "1d");
        assert_eq!(format_interval(3.5), "3.5d");
        assert_eq!(format_interval(60.0), "2mo");
        assert_eq!(format_interval(400.0), "1.1y");
    }

    #[test]
    fn preview_interval_matches_schedule() {
        let p = params();
        let sp = ScheduleParams::from(&p);
        let s = schedule(now(), &sp, &new_state(), Grade::Again);
        let preview = preview_interval(now(), &sp, &new_state(), Grade::Again);
        assert_eq!(preview, format!("{}m", (s.due - now()).num_minutes()));
    }

    #[test]
    fn state_serde_roundtrip() {
        let s = schedule_sm2(now(), &params(), &new_state(), Grade::Good);
        let json = serde_json::to_string(&s).unwrap();
        let back: SchedulingState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn scheduler_kind_defaults_to_sm2() {
        assert_eq!(SchedulerKind::default(), SchedulerKind::Sm2);
        assert_eq!(SchedulerKind::Sm2.label(), "SM-2");
        assert_eq!(SchedulerKind::Fsrs.label(), "FSRS");
    }

    #[test]
    fn schedule_params_from_sm2_params_keeps_sm2_selected() {
        let p = params();
        let sp = ScheduleParams::from(&p);
        assert_eq!(sp.kind, SchedulerKind::Sm2);
        assert_eq!(sp.sm2, p);
        assert_eq!(sp.fsrs.target_retention, 0.9);

        let next = schedule(now(), &sp, &new_state(), Grade::Good);
        let mut expected = new_state();
        expected.reps = 1;
        expected.phase = LearnPhase::Learning;
        expected.step = 1;
        expected.due = now() + Duration::minutes(10);
        assert_eq!(next, expected);
    }

    #[test]
    fn fsrs_dispatch_changes_the_graduated_interval_only() {
        let p = params();
        let sm2 = ScheduleParams::from(&p);
        let fsrs = ScheduleParams::fsrs(&p, FsrsParams::default());
        let a = schedule(now(), &sm2, &new_state(), Grade::Good);
        let b = schedule(now(), &fsrs, &new_state(), Grade::Good);
        // Identical learning-step flow.
        assert_eq!(a.phase, b.phase);
        assert_eq!(a.step, b.step);
        assert_eq!(a.due, b.due);
        // Different graduation interval.
        let a = schedule(now(), &sm2, &a, Grade::Good);
        let b = schedule(now(), &fsrs, &b, Grade::Good);
        assert_eq!(a.phase, b.phase);
        assert_eq!(a.interval_days, p.graduating_interval_days);
        assert!(b.interval_days > a.interval_days);
        assert!(b.has_fsrs_memory());
        assert!(!a.has_fsrs_memory());
    }

    #[test]
    fn scheduler_for_returns_the_selected_policy() {
        let sm2 = scheduler_for(SchedulerKind::Sm2).schedule(
            now(),
            &ScheduleParams::default(),
            &new_state(),
            Grade::Again,
        );
        assert_eq!(sm2.phase, LearnPhase::Learning);

        let fsrs_params = ScheduleParams::fsrs(&params(), FsrsParams::default());
        let fsrs = scheduler_for(SchedulerKind::Fsrs).schedule(
            now(),
            &fsrs_params,
            &new_state(),
            Grade::Easy,
        );
        assert_eq!(fsrs.phase, LearnPhase::Review);
    }

    #[test]
    fn preview_interval_uses_the_selected_scheduler() {
        let p = params();
        let sm2 = ScheduleParams::from(&p);
        let fsrs = ScheduleParams::fsrs(&p, FsrsParams::default());
        let sm2_preview = preview_interval(now(), &sm2, &new_state(), Grade::Easy);
        let fsrs_preview = preview_interval(now(), &fsrs, &new_state(), Grade::Easy);
        assert_eq!(sm2_preview, "4d");
        assert_ne!(sm2_preview, fsrs_preview);
        assert_eq!(
            fsrs_preview,
            format_interval(fsrs::initial_stability(Grade::Easy))
        );
    }

    #[test]
    fn schedule_params_serde_roundtrip() {
        let sp = ScheduleParams::fsrs(
            &params(),
            FsrsParams {
                target_retention: 0.85,
            },
        );
        let json = serde_json::to_string(&sp).unwrap();
        let back: ScheduleParams = serde_json::from_str(&json).unwrap();
        assert_eq!(back, sp);
    }

    #[test]
    fn schedule_params_missing_fields_default() {
        let back: ScheduleParams = serde_json::from_str("{}").unwrap();
        assert_eq!(back.kind, SchedulerKind::Sm2);
        assert_eq!(back.sm2, SchedParams::default());
        assert_eq!(back.fsrs.target_retention, 0.9);
    }

    #[test]
    fn fsrs_params_are_sanitised_and_defaulted() {
        assert_eq!(FsrsParams::default().target_retention, 0.9);
        let low = FsrsParams {
            target_retention: 0.1,
        };
        let high = FsrsParams {
            target_retention: 0.99,
        };
        let nan = FsrsParams {
            target_retention: f64::NAN,
        };
        assert_eq!(
            low.sanitised().target_retention,
            FsrsParams::MIN_TARGET_RETENTION
        );
        assert_eq!(
            high.sanitised().target_retention,
            FsrsParams::MAX_TARGET_RETENTION
        );
        assert_eq!(nan.sanitised().target_retention, 0.9);
        let back: FsrsParams = serde_json::from_str("{}").unwrap();
        assert_eq!(back, FsrsParams::default());
    }

    #[test]
    fn scheduling_state_without_fsrs_fields_still_deserialises() {
        let s = new_state();
        let mut v: serde_json::Value = serde_json::to_value(&s).unwrap();
        for key in ["fsrs_difficulty", "fsrs_stability", "fsrs_retrievability"] {
            v.as_object_mut().unwrap().remove(key);
        }
        let back: SchedulingState = serde_json::from_value(v).unwrap();
        assert_eq!(back.fsrs_difficulty, 0.0);
        assert_eq!(back.fsrs_stability, 0.0);
        assert_eq!(back.fsrs_retrievability, 0.0);
        assert!(!back.has_fsrs_memory());
    }
}
