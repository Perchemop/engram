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

## Open maintainer actions

- **Employer/IP-policy check (guide §19.6, D-024).** Confirm that the
  maintainer's employer's intellectual-property policy does not claim this
  off-hours, personal-account project, and record the outcome here. The MIT
  licence is only clean if this is settled. *Owner: maintainer. Status:
  open.*
- **Account security (guide §20.1, D-021).** Enable multi-factor
  authentication on the GitHub account that owns the repository, and if the
  project ever moves to a GitHub organisation, add at least a second human
  administrator for account recovery and separation of duties. *Owner:
  maintainer. Status: open.*

## Metrics policy

- **Measurement is intentionally minimal (guide §23.5).** Engram has no
  in-app telemetry by design. Discovery, usage and retention are judged only
  from public signals — GitHub stars and forks, release download counts and,
  if consulted at all, GitHub Insights — and never from user data. There is
  no server to collect anything, so any future metric would have to be added
  deliberately; if that ever happens it must be aggregate, public and
  opt-in, and it must be recorded in this document. *Status: policy; revisit
  only with a concrete reason.*

## Resolved during the 2026-10-24 guide update
- **Q4 (suite?)** → Engram is a standalone app; crash-report and
  update-check ship as in-app modules designed for later extraction
  (D-015). Revisit if a second app joins a suite.
- **Q10 (primary local platform)** → Linux aarch64 (the owner's daily
  machine); CI builds it natively (PR #4).
- **Q16–Q18 (releases, roadmap, shared crates)** → confirmed: semver GitHub
  Releases with detailed notes (`release.yml`), public `ROADMAP.md` with
  evidence-based readiness, crash/update capabilities implemented (D-015).
- **Q8 (public research sources)** → official docs and public algorithm
  descriptions only; research notes live in `docs/research/` (created when
  first needed) with the human gate before dependent implementation.

## Resolved during discovery
- Product name → **Engram** (D-001; "SynapseFlow" rejected as too close to
  the inspiration product).
- Target platforms → desktop three (Win/macOS/Linux) first; WASM deferred.
- V1 scope → deck/tag management, SM-2 scheduling, Basic+Cloze cards,
  study mode, analytics, import/export, offline generator. Out of scope:
  sync servers, accounts, mobile-native, real-time collaboration.
