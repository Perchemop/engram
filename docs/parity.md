# Feature Parity Tracker

`REQUIREMENTS_TRACEABILITY.md` is the authoritative capability tracker for
Engram (requirement → implementation → tests → status). This file records
the high-level parity posture against the inspiration product so the two
views stay connected without duplicating detail.

| Capability (inspiration level) | Engram V1 | Where |
|---|---|---|
| Flashcard decks with organisation | ✅ hierarchical deck tree + colours | `domain/deck.rs`, `ui/decks.rs` |
| Tags | ✅ normalised tags + AND-filtering | `domain/card.rs`, `domain/library.rs` |
| Basic cards | ✅ | `domain/card.rs` |
| Cloze cards | ✅ single-face (all groups) | `domain/card.rs` (multi-face deferred) |
| Adaptive scheduling | ✅ SM-2 + steps + params | `domain/scheduler.rs` |
| Guided study sessions | ✅ keyboard-first flow + summary | `ui/study.rs` |
| Statistics / progress | ✅ heatmap, forecast, retention, streak | `domain/stats.rs`, `ui/stats.rs` |
| Card generation from notes | ✅ offline heuristic (no network) | `domain/aigen.rs`, `ui/generate.rs` |
| Import/export | ✅ CSV + JSON backup + native format | `data/export.rs` |
| Cloud sync | ❌ out of scope V1 | scope.md |
| Accounts / subscriptions | ❌ none, by design | scope.md |

Status vocabulary: ✅ implemented · 🔶 partial (noted) · ❌ not planned/deferred.
Every ✅ must have a trace row in `REQUIREMENTS_TRACEABILITY.md`; 🔶 rows
must say what is deferred and where.
