# ROADMAP

Public roadmap for **Engram** — spaced repetition & active recall.
Readiness percentages reflect *verified* evidence (CI + traceability), never
intent. Updated per release (guide §6.2).

## Current release: v0.1.0 — first functional test build

| Area | Ready | Evidence |
|---|---|---|
| Core domain (cards, cloze, SM-2 scheduler, sessions, stats) | 100% | 83 tests green on 4 platforms; `REQUIREMENTS_TRACEABILITY.md` |
| Deck & tag management | 100% | UI + domain tests |
| Study mode (keyboard-first) | 90% | Flow verified in tests + smoke run; human keyboard/visual walk pending |
| Statistics views | 100% | Domain tests + UI |
| Persistence & import/export | 100% | Atomic JSON, CSV/JSON round-trips tested |
| Crash reporting | 100% (local report + consent flow; GitHub submit via pre-filled issue URL) | `src/crash_report.rs` + tests |
| Update check | 100% (GitHub Releases API, 24h cache, fail-soft) | `src/update_check.rs` + tests |
| Documentation | 100% | README, ATTRIBUTION, known limitations, implementation report |

## Milestones

### v0.1.x — polish on the first functional build
- [ ] Manual keyboard/visual QC walk completed and recorded (human step)
- [ ] Dark/light theme contrast audit on real hardware
- [ ] Multi-cloze-per-card study faces (one deletion per review)

### v0.2.0 — richer study workflow
- [ ] Card history browser; suspend/bury cards
- [ ] Filtered "cram" sessions ignoring the scheduler
- [ ] Deck-local daily-limit overrides
- [ ] Queue interleaving/shuffle as a setting

### v0.3.0 — data & interoperability
- [ ] Anki `.apkg` import (public format research, human-gated per guide §2.3)
- [ ] FSRS scheduler option alongside SM-2 (public algorithm)
- [ ] Backup reminders / auto-rotation of backups

### v1.0.0 — finished product (guide §9 gates)
- [ ] All §9 finished-product gates pass
- [ ] §2.5 legal checklist clean
- [ ] Web/WASM target evaluated (browser storage backend)
- [ ] Installers/packages per platform if demand warrants

## Explicitly out of scope (see `docs/scope.md`)
Cloud sync, accounts/subscriptions, real-time collaboration, native mobile.
