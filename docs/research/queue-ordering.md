# Research note — Queue interleaving / shuffle setting

**Status:** DRAFT — awaiting human approval (guide §2.3/§8.4).
**Milestone target:** v0.2.0.
**Sources (public, capability-level only):**
- Anki manual, new-card gather/spread options: <https://docs.ankiweb.net/deck-options.html#new-cards>
- Engram's own deterministic queue: DECISIONS D-008, `docs/data-model.md`

## Behaviour to implement

1. New setting `queue_order: QueueOrder` (library setting, serde default =
   current behaviour so 0.1.0 files load unchanged):
   - `DueFirst` (default, current): learning → review (soonest due) → new
     (creation order).
   - `Interleaved`: new cards are spread through the due cards
     (deterministic spread, e.g. every k-th position).
   - `Random`: seeded shuffle — seed = library seed + day stamp so a given
     day is reproducible (testable) while feeling random across days.
2. Setting lives in Settings dialog (radio group) with a one-line
   description of each mode.
3. Session summaries and daily-limit accounting are unaffected.

## Acceptance criteria

- [ ] `DueFirst` produces bit-identical queues to v0.1.0.
- [ ] `Interleaved` distributes new cards as evenly as possible between
      review cards (documented formula: position = floor(i * (R+N)/N) for
      the i-th new card), stable for the same inputs.
- [ ] `Random` uses a seeded PRNG (e.g. library seed) — same seed + same
      day + same library state → same queue; different days differ.
- [ ] Empty categories degrade gracefully (no due cards / no new cards).
- [ ] All three modes respect daily limits and the learning-first rule for
      learning steps.

## Edge cases

- Zero new cards / zero review cards: every mode equals DueFirst.
- More new cards than reviews: Interleaved still spreads (starts with new
  cards after the first due position).
- Random mode must never randomise *learning steps* (steps stay due-order
  — capping/stranding rules from D-014 unchanged).

## Design notes (original)

- Domain: pure functions `interleave(queue, new, ratio)` and a seeded
  shuffle in `session.rs`; the mode enum is serde-defaulted. Determinism
  is preserved (D-008 refined, not broken: "deterministic given the mode").
