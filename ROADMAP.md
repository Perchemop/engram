# ROADMAP

Public roadmap for **Engram** — spaced repetition & active recall.
Readiness percentages reflect *verified* evidence (CI runs + traceability +
local runs), never intent (guide §6.2). Updated per release.

## Where we are

**v0.1.0 released 2026-10-23** — first functional test build. The study
core (decks, cards, cloze, SM-2 scheduling, study sessions, statistics,
persistence, import/export, crash reporting, update check) is implemented,
tested on four platforms, and installable.

Feature breadth of the long-term product vision implemented so far:
**~55%**. Ready-for-work items in the next milestone: **100%** (research
notes drafted, human gate pending).

## Milestones

### v0.1.x — polish on the first functional build
*(no research dependency; fix-driven)*
- [ ] Manual keyboard/visual QC walk completed and recorded (human step)
- [ ] Dark/light contrast audit on real hardware
- [ ] Small fixes discovered by the QC walk
- Ready-for-work: **100%** · Target: as fixes land

### v0.2.0 — richer study workflow
*(research notes drafted — HUMAN GATE required before implementation,
guide §2.3/§8.4)*
- [ ] Multi-cloze-per-card study faces — one deletion per review
      (`docs/research/multi-cloze-faces.md`)
- [ ] Card history browser; suspend/bury cards
      (`docs/research/card-history-suspend-bury.md`)
- [ ] Filtered "cram" sessions that bypass the scheduler
      (`docs/research/cram-sessions.md`)
- [ ] Deck-local daily-limit overrides
      (`docs/research/deck-local-limits.md`)
- [ ] Queue interleaving/shuffle as a setting
      (`docs/research/queue-ordering.md`)
- Ready-for-work: **100%** (notes drafted: `multi-cloze-faces.md`,
  `card-history-suspend-bury.md`, `cram-sessions.md`, `deck-local-limits.md`,
  `queue-ordering.md`) · Human gate: **pending**

### v0.3.0 — data & interoperability
*(research notes drafted — HUMAN GATE required)*
- [ ] Anki text import (public TSV/CSV conventions)
      (`docs/research/anki-import.md`)
- [ ] FSRS scheduler option alongside SM-2 (public algorithm + reference
      implementations) (`docs/research/fsrs-scheduler.md`)
- [ ] Backup reminders / rotation of `.engram.json` snapshots
- Ready-for-work: **notes drafted** (`anki-import.md`, `fsrs-scheduler.md`)
  · Human gate: **pending**

### v1.0.0 — finished product (guide §9 gates)
- [ ] All §9 finished-product gates pass on the primary platform
- [ ] §2.5 legal checklist clean for a full release
- [ ] Web/WASM target evaluated (browser storage backend, assumption Q-1)
- [ ] Installers/packages per platform if demand warrants

## Explicitly out of scope (see `docs/scope.md`)
Cloud sync, accounts/subscriptions, real-time collaboration, native mobile.
