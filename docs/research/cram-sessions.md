# Research note — Filtered "cram" sessions

**Status:** DRAFT — awaiting human approval (guide §2.3/§8.4).
**Milestone target:** v0.2.0.
**Sources (public, capability-level only):**
- Anki manual, custom study / filtered decks: <https://docs.ankiweb.net/filtered-decks.html>
- Engram's own queue design: `docs/data-model.md`, DECISIONS D-008/D-014

## Behaviour to implement

1. **Cram session**: study a chosen scope **ignoring the scheduler** —
   all cards in scope are eligible regardless of phase or due date.
2. Scope selection reuses the existing picker (All / deck subtree) plus a
   tag filter (AND semantics, same as search).
3. Ordering: same deterministic rules as a normal session (learning →
   review → new is *not* applicable; crams order by due date where it
   exists, then creation order). Simplest documented rule: **creation
   order**, optionally "hardest first" by lowest ease in v2.
4. **Crams do not mutate scheduling by default** ("preview mode"): answers
   are recorded to the review log with queue kind `Cram` (new variant) but
   the card's `SchedulingState` is untouched. Rationale: cramming should
   not distort the schedule (standard SRS practice). A per-session toggle
   "apply grades" may come later.
5. Daily limits do not apply; sessions are unlimited in size.
6. Session summary identical to normal study (grades, retention, time).

## Acceptance criteria

- [ ] Cram of a scope with 500 cards yields a 500-card queue (cap only by
      an explicit user-entered limit, default 100).
- [ ] Cards due in the future and suspended-by-schedule cards (future-due
      review cards) are all included; suspended cards are excluded (they
      are excluded everywhere).
- [ ] Answering in a cram records a `ReviewLog` with `queue_kind = Cram`
      and leaves `card.sched` unchanged (bit-identical before/after).
- [ ] Statistics: cram reviews count in the activity heatmap/retention but
      NOT in the daily-limit accounting (D-014 unchanged).
- [ ] Starting a cram requires no cards to be due.

## Edge cases

- Empty scope → same honest "nothing to study" error path.
- Card edited/deleted mid-cram: skip gracefully (existing session
  behaviour).
- Cram of one card repeated: allowed (user choice), each answer logged.

## Design notes (original)

- Domain: `Library::build_cram_session(scope, tag_filter, limit)`; new
  `QueueKind::Cram`; `record_review` gains no new parameters (queue kind is
  passed as today) — but `record_review` must branch: for `Cram`, skip the
  scheduler update and log with `interval_before == interval_after`.
- `SchedParams`/statistics unaffected; stats gain a "crams" count.
