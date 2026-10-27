# Research note — FSRS scheduler option

**Status:** APPROVED — human approval granted 2026-10-27 (guide §2.3/§8.4).
**Milestone target:** v0.3.0.
**Sources (public, capability-level only):**
- FSRS algorithm description (open-spaced-repetition project):
  <https://github.com/open-spaced-repetition/fsrs4anki/wiki/The-Algorithm>
- Reference implementations (MIT/BSD-style): <https://github.com/open-spaced-repetition/py-fsrs>
- SM-2 (current Engram scheduler, D-004): <https://supermemo.guru/wiki/SuperMemo_1.0_for_DOS_(1987)_-_SuperMemo_2_(1987)>

## What FSRS is (public description)

FSRS ("Free Spaced Repetition Scheduler") is a publicly documented,
openly implemented scheduler that models three memory variables per card:

- **R**etrievability — probability of recalling right now (decays with time).
- **D**ifficulty — how hard the card is for this user (moves with grades).
- **S**tability — how long until retrievability drops to a threshold.

The next interval is computed from stability and a **target retention**
parameter (default ~0.9): `interval ≈ stability * factor(R)`, instead of
SM-2's ease-factor multiplication. The default parameter weights are
public and can be used as-is; per-user optimisation is explicitly out of
scope for a first version.

## Behaviour to implement

1. **Scheduler becomes pluggable**: `enum SchedulerKind { Sm2, Fsrs }` in
   library settings; SM-2 stays the default and is unchanged.
2. FSRS cards carry FSRS state (D, S, R + due + phase) in
   `SchedulingState` (additive serde fields; SM-2 fields remain for SM-2
   cards and for migration).
3. Same four grades, same learning-step flow for new cards, same lapse
   handling into relearning steps — only the *graduated interval
   computation* differs.
4. Settings: target retention slider (0.7–0.95); the public default
   parameter weights are used.
5. Cards are scheduled by whatever kind their state uses; switching a
   library to FSRS re-derives FSRS state from existing SM-2 fields on
   first review (documented approximation), not retroactively.

## Acceptance criteria

- [ ] With the default parameters, a Good answer on a new card graduates
      after the learning steps exactly as in SM-2 (identical step flow).
- [ ] Review intervals increase with stability and decrease as target
      retention rises; target retention 0.9 with public defaults yields
      intervals in the publicly documented ballpark for the first reviews.
- [ ] SM-2 libraries behave bit-identically to v0.1.0 when the scheduler
      is left on SM-2 (pure addition, no regression).
- [ ] Round-trip: FSRS state serialises; 0.1.0 files load.
- [ ] Determinism: same state + grade + time → same interval (no clocks,
      no randomness in the scheduler).

## Edge cases

- First review of a card (no stability yet): standard FSRS initial
  stability per grade (public defaults).
- Very long lapses / very high intervals: clamp to the existing
  max-interval cap.
- Mixed libraries (some cards SM-2, some FSRS during a transition): fully
  supported — the kind lives with the card's state.

## Design notes (original)

- Domain: `scheduler::fsrs` module implementing the three-variable model
  with the public default weights as constants; a `Scheduler` trait shared
  with the SM-2 path so the queue/session/statistics code is
  scheduler-agnostic. Maturity threshold and preview strings keep working
  per scheduler.
- Attribution: FSRS is referenced as a public algorithm; Engram's
  implementation is written from the public description and remains
  original code (MIT OR Apache-2.0), credited in `ATTRIBUTION.md`.
