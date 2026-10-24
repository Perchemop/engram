# Research note — Card history browser, suspend & bury

**Status:** DRAFT — awaiting human approval (guide §2.3/§8.4).
**Milestone target:** v0.2.0.
**Sources (public, capability-level only):**
- Anki manual, pausing (suspend/bury): <https://docs.ankiweb.net/studying.html#editing-and-more>
- Review-log design: local `docs/data-model.md` (Engram's own append-only log)

## Behaviour to implement

### 1. Card history browser
1. A per-card view (opened from the card table's Edit row or a History
   button) listing every `ReviewLog` entry for that card, newest first:
   timestamp, grade, queue kind, interval before → after.
2. The card's totals: review count, lapse count, current ease, current
   interval, introduced-on date (first log entry).
3. Read-only in V1 (no log editing — the log is append-only, D-006).

### 2. Suspend (pause a card)
1. `Card.suspended: bool` (serde default false). Suspended cards:
   - never enter study queues (any phase),
   - are excluded from due counts and forecasts,
   - remain fully visible/editable in the card table (badge "suspended").
2. Suspend/resume from the card table (bulk action on selection) and the
   card editor. Persisted state — survives restart.
3. Resume keeps the old due date (card may be immediately due).

### 3. Bury (postpone within a day)
1. "Bury until tomorrow": `Card.buried_until: Option<DateTime<Utc>>`.
2. Buried cards are excluded from queues until that instant passes, then
   return automatically (no user action).
3. Use case: "show me this again tomorrow" without grading.

## Acceptance criteria

- [ ] History view lists exactly the card's log rows, newest first, with
      the five fields above.
- [ ] Suspended card never appears in `build_session` output or
      `due_counts`; appears in the table with a badge; resume restores its
      previous due state exactly.
- [ ] `buried_until` in the future excludes the card; once past, the card
      is eligible again without mutation.
- [ ] Deleting a card removes its history (existing cascade rule).
- [ ] All three fields are serde-defaulted (0.1.0 files load unchanged).

## Edge cases

- Suspend during an open study session: the card is dropped from the
  remaining queue (session keeps already-answered entries).
- Bury + suspend together: suspend wins (never queued).
- Statistics: suspended cards excluded from "due/forecast"; included in
  total counts with their own bucket; streak/retention unchanged.

## Design notes (original)

- Domain: add `suspended`/`buried_until` to `Card`; a single
  `Card::is_blocked(now)` predicate used by `build_session`,
  `due_counts`, `forecast`.
- UI: badge in the table row; actions in bulk bar and editor; history view
  as a modal reading `library.reviews` filtered by card id.
