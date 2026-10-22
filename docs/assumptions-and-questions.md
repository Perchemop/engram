# Facts, Assumptions, and Decisions

## Facts
- **F-1:** SM-2 is a publicly documented algorithm (Wozniak, 1990); its
  parameters and update rules are public knowledge.
- **F-2:** `{{cN::…}}` cloze markup is a long-standing public interchange
  convention for plain-text cloze cards.
- **F-3:** egui/eframe 0.36 supports Windows, macOS and Linux from one
  codebase; the domain layer has no platform dependencies.
- **F-4:** The local machine is resource-constrained; heavy compilation is
  CI-only (user directive, guide §4).
- **F-5:** Atomic write (temp + rename) prevents partial-file corruption on
  POSIX and Windows filesystems.

## Assumptions
- **A-1:** Users prefer local-first file control over forced cloud accounts.
  *Risk:* no cross-device sync in V1; export/import is the bridge.
- **A-2:** JSON storage stays fast enough for libraries of ~10k cards.
  *Risk:* very large libraries may slow the card table; mitigated by the
  400-row render cap; SQLite remains a fallback (not needed yet).
- **A-3:** A 1–4 self-rating scale (Again/Hard/Good/Easy) is the right
  granularity for V1 self-assessment.
- **A-4:** All-deletion-groups-on-one-face is acceptable for V1 cloze study;
  per-deletion faces are post-V1.

## Open questions
- **Q-1 (open):** Web/WASM target — browser persistence backend
  (localStorage vs OPFS) and eframe web maturity. *Owner direction: desktop
  first; WASM deferred to phase 5.*
- **Q-2 (open):** Deck-local daily limits (per-deck new/review caps) vs the
  current library-wide limits. Deferred; single limits are simpler and
  cover the core workflow.
- **Q-3 (open):** Remote LLM card generation. The offline heuristic
  generator shipped in V1 (D-009); an optional BYO-API-key generator is a
  candidate for a later slice and was never promised in V1 scope.

## Resolved during discovery
- Product name → **Engram** (D-001; "SynapseFlow" rejected as too close to
  the inspiration product).
- Target platforms → desktop three (Win/macOS/Linux) first; WASM deferred.
- V1 scope → deck/tag management, SM-2 scheduling, Basic+Cloze cards,
  study mode, analytics, import/export, offline generator. Out of scope:
  sync servers, accounts, mobile-native, real-time collaboration.
