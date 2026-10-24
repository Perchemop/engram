# DECISIONS

Architecture and process decisions, newest first. Each entry states the
decision, the rationale, and the rejected alternatives.

## 2026-10-23

### D-001 — Product name: **Engram**
"SynapseFlow" (working name) was judged too similar to the inspiration
product's name and replaced with **Engram** — the neuroscience term for the
physical trace a memory leaves. Original, short, and unrelated to any
existing SRS product name. *Rejected:* Loci, Rote, Memorable (weaker or more
generic), keeping SynapseFlow (clean-room naming hygiene).

### D-002 — Stack: pure Rust + egui/eframe 0.36, JSON persistence
Matches the guide's default recommendation for desktop tools; gives
cross-platform builds (Win/macOS/Linux) from one codebase and headless-testable
domain code. SQLite was rejected for V1 — human-readable JSON keeps the data
format auditable and diffable, and atomic writes plus small libraries make a
database unnecessary. WASM/web is deferred (phase 5) with browser-storage
concerns (see assumptions Q-1).

### D-003 — CI-first workflow; all heavy gates in GitHub Actions
The local machine is memory-constrained (guide §4). `cargo test/clippy/check`
and release builds run only in CI on Linux+Windows+macOS runners; locally only
`cargo fmt` and read-only git are used. Verification truth = the CI result.

### D-004 — Scheduling: SM-2 with learning/relearning steps
The publicly documented SuperMemo-2 algorithm, extended with short-interval
learning steps (1m, 10m defaults), an ease clamp (1.3–3.0), an easy bonus
(1.3×) and a hard factor (1.2×). All parameters are user-configurable and
sanitised at the boundary. *Rejected:* a from-scratch adaptive algorithm
(unproven, unreviewable), FSRS (heavier state, harder to test in one slice;
candidate for post-V1).

### D-005 — Time is injected, never read, inside the domain
Every scheduling/statistics/session function takes `now: DateTime<Utc>`.
Makes the whole domain deterministic and testable (fixed timestamps in
tests), and keeps real-clock reads at the UI boundary. *Rejected:* domain
calling `Utc::now()` (untestable, hides mutation of due dates).

### D-006 — Review log is append-only and central
Reviews are logged once per answer in `Library::reviews` (id, card, deck,
timestamp, grade, before/after interval) instead of per-card histories.
Simplifies statistics, keeps cards small, and makes deletion semantics clean:
logs die with their card or deck.

### D-007 — Delete-deck semantics: cards cascade, sub-decks re-parent
Deleting a deck deletes its cards (and their logs) but re-parents child decks
to the deleted deck's parent, so no part of the tree is silently lost. The UI
states this explicitly in the confirmation prompt. *Rejected:* refusing to
delete non-empty decks (annoying), full cascade of sub-decks (data loss).

### D-008 — Session queue order: learning → review → new, deterministic
Due learning steps first (soonest due), then due reviews (soonest due), then
new cards in creation order capped by the daily new limit. No randomisation
in V1 so tests are deterministic and users can reason about the queue.
*Rejected:* interleaving (deferred; noted as a future setting).

### D-009 — Offline heuristic generator instead of an LLM API
The "AI card generation" capability ships as a fully offline rule-based
generator (definition lines, Q/A pairs, copula sentences → cloze). It never
invents facts because it only quotes source text. *Rejected:* shipping an
HTTP LLM client in V1 (network dependency, key management, non-determinism,
untestable offline; explicitly deferred).

### D-010 — Auto-save on every mutation, atomically
The library is written to disk (temp file + rename) whenever a command
succeeds; no explicit save is required and no data can sit unsaved beyond one
frame. *Rejected:* manual-save-only (violates "never lose user data"), debounced
autosave (window for loss on crash).

### D-011 — Cloze syntax: `{{cN::…}}` public convention
Reading/writing the long-standing public cloze markup keeps card text
portable as plain text. V1 studies all deletion groups on one face and grades
the card as a whole; per-deletion faces are a post-V1 item (traceability
notes this honestly).

### D-012 — CSV as the exchange format, JSON for backups
CSV (with RFC-4180 quoting, `Deck,Type,Front,Back,Tags`) interchanges with
spreadsheets and other tools; whole-library JSON backups are exact copies.
*Rejected:* only-native-format (no interop), Anki's proprietary binary
`.anki2` (not a public format; clean-room rules forbid reading its internals).

---

## 2026-10-23

### D-013 — CTO.new-style flow adopted; PRs may be self-merged when green

The build guide was updated to the CTO.new operating model: **Chat (plan) →
Task (execute) → Pull Request (review) → merge**, where GitHub Actions is
the free "Task Runner" and every Task ends in a PR. The owner additionally
delegated merge authority: with CI green **and** the reviewer pass done, the
orchestrator may merge the `opencode/ci` PR itself instead of waiting for a
human merge. Human review remains possible (PRs stay open for reading), and
the reviewer subagent runs before every merge.

Agent definitions were aligned to the guide's canonical roles
(`orchestrator` = Chat planner, `coder` = Task executor, `explorer`,
`reviewer`), with the existing specialists retained as Task-mode helpers.
*Rejected:* pushing directly to `main` without a PR (loses the review
surface the guide now requires).

### D-014 — Daily limits are per-calendar-day, derived from the review log

Review of the V1 diff found that `daily_new_limit` / `daily_reviews_limit`
were applied per `build_session` call, so a second session the same day got
a fresh allotment. Fixed: card introductions (a card's first-ever review)
and non-introduction reviews are counted per calendar day from the review
log; learning/relearning steps are always served (capping them would strand
cards mid-step). No schema change was needed — first-review data already
exists in the log. *Rejected:* adding a `was_new` field (redundant; the log
already records first reviews; a compat default would add noise).

---

## 2026-10-24

### D-015 — Crash report + update check as in-app modules (guide §6.1/§7)
The updated guide makes a crash stub and update-check stub **v0.1.0 hard
gates** and prescribes shared suite crates (`crates/crash-report`,
`update-check`) for *suite* apps. Engram is a standalone app today, so both
capabilities ship as in-app modules — `src/crash_report.rs` and
`src/update_check.rs` — with clean, dependency-light APIs designed to be
lifted into shared crates if a suite forms. Crash reports stay local until
the user clicks; submission is a pre-filled GitHub issue URL (no embedded
tokens, ever). Update checks hit the public Releases API at most once per
24h and fail soft offline. *Rejected:* shipping only placeholder stubs
(the real behaviour is small, testable, and guide §7 specifies it);
embedding a bot token (§2.5 forbids); creating two extra crates for a
single app (premature).

### D-016 — v0.1.0 tagged from the guide's hard-gate checklist
Version stays `0.1.0` (Cargo.toml already matched). The tag is cut only
after: CI green on 4 platforms, CHANGELOG `## [0.1.0]` entry written,
crash + update-check implemented, legal baseline present (LICENSE,
ATTRIBUTION, no secrets), and `release.yml` verified to publish the GitHub
Release with per-platform binaries + SHA-256 checksums.

---

## 2026-10-24

### D-017 — cargo is banned on the local machine outright
Owner directive: **no `cargo` subcommand of any kind may run locally —
including `cargo fmt`, `cargo check`, and "small" tests.** This supersedes
the earlier "fmt is trivial" exception. Formatting is produced by hand
(rustfmt-clean style) and enforced by CI's `cargo fmt --all -- --check`
gate; any fmt/clippy/test failure is fixed by editing source and re-running
CI. Agent definitions and process docs were updated to remove the fmt
allowance. *Rejected:* keeping the fmt exception (the owner wants zero Rust
toolchain executions on the constrained host, removing all ambiguity about
what counts as "heavy").

---

## 2026-10-24

### D-018 — The local test restriction is a maintainer-machine constraint (guide §26.1)

The "cargo is banned locally" rule (D-017) applies to the maintainer's
resource-constrained machine, not to the project. Contributors are expected
and encouraged to run `cargo test --all`, `cargo fmt --all -- --check` and
`cargo clippy --all-targets -- -D warnings` locally before opening a pull
request. CI remains the authoritative verification for everyone.
*Rationale:* the rule exists to protect one machine from restarts under
load; presenting it as a project-wide norm would wrongly discourage
contributors from testing their own work. `CONTRIBUTING.md` and
`docs/development.md` state both halves explicitly. *Rejected:* telling all
contributors not to run cargo (worse quality, slower feedback); leaving the
wording ambiguous (contributors would not know which rule applies to them).

### D-019 — Repository is public from day one, with a no-secrets gate and branch protection (guide §26.2)

The repository is public from the first commit, which buys free CI minutes
and an open contribution surface. In exchange, a hard gate applies from the
start: **no secret, token or credential is ever committed**, verified by a
secrets scan (gitleaks, D-026) and by the pre-commit/tree checks, and branch
protection is enabled on `main` from the beginning. *Rationale:* going public
first is simpler than migrating a private history later, and the real risk of
public history is secrets, which a scan plus review closes. *Rejected:*
starting private (loses free CI and the open model, defers the hard problem);
public without a secrets gate (unacceptable history risk).

### D-020 — Pull-request flow for every change; branch protection requires the CI check on `main` (guide §26.3)

Every change — human or agent-authored — lands through a pull request. Branch
protection on `main` requires the authoritative CI check to pass before merge.
Direct pushes to `main` stop. The `.ci-required.txt` file names the checks
that decide the verdict (`check` on ubuntu-latest, `build` on windows-latest
and macos-latest). *Rationale:* the pull request is the review and audit
surface; requiring the check makes "CI is authoritative" mechanically true
rather than a convention. *Rejected:* direct pushes to `main` (no review, no
gate); advisory-only CI (nothing forces green before merge).

### D-021 — Agent automation uses a scoped credential and the project git identity (guide §26.4)

Automated build agents act through a scoped credential with the least
privilege needed for their task, and all agent-authored commits use the
project's git identity, **Engram Dev `<dev@example.com>`** — never a personal
account. If the project ever moves to a GitHub organisation, at least two
human administrators are recommended for account recovery and separation of
duties. *Rationale:* a scoped credential limits blast radius if it leaks, and
a single project identity keeps authorship honest and attributable.
*Rejected:* a personal-access token with broad scope (over-privileged);
committing under a personal name (misattributes agent work).

### D-022 — `guide.md` stays local; contributor-facing process lives in the repo (guide §26.5)

The internal build guide (`guide.md`) is not committed or published; it is
git-ignored. Everything a contributor needs lives in contributor-facing
documents: `README.md`, `CONTRIBUTING.md` and `GOVERNANCE.md` (with
`docs/development.md` for engineering workflow). *Rationale:* reference
material and process notes must not leak into the public repository, but
contributors must still have a complete, self-contained process.
*Rejected:* committing the guide (publishes internal reference material and
notes about other products); leaving process undocumented for contributors.

### D-023 — Licence is deliberately MIT-only, not `MIT OR Apache-2.0` (guide §26.6)

Engram is licensed under **MIT only**. This is a deliberate choice for a
blank-slate project: MIT is the simplest, most widely understood permissive
licence, and it satisfies the "use outputs freely" goal without the dual
`MIT OR Apache-2.0` ceremony. Relicensing remains possible later if a reason
appears. `Cargo.toml` (`license = "MIT"`), `LICENSE`, `ATTRIBUTION.md` and
the CI software bill of materials must all report MIT accurately.
*Rationale:* one clear licence reduces ambiguity for users and for
downstream packagers. *Rejected:* dual `MIT OR Apache-2.0` (extra ceremony
with no benefit here); copyleft licences (against the project's goals).

### D-024 — Employer/IP policy check recorded as a maintainer action item (guide §19.6)

The maintainer must confirm that their employer's intellectual-property
policy does not claim work on this personal, off-hours project, and record
the outcome. It is tracked as an open action item in
`docs/assumptions-and-questions.md`. *Rationale:* the check is a real legal
prerequisite for an unencumbered licence, and recording it makes it
verifiable rather than assumed. *Rejected:* treating it as obvious and
leaving no record (unverifiable if the licence is ever questioned).

### D-025 — Accessibility target: WCAG 2.2 AA where feasible

Engram adopts **WCAG 2.2 Level AA where feasible** as its accessibility
target, with documented exceptions where an immediate-mode native toolkit
cannot meet a web-content criterion exactly. Priorities are keyboard support,
AccessKit screen-reader semantics, colour contrast and reduced motion; the
custom-painted heatmap's lack of per-cell semantics and the absence of
automated assistive-technology tests are recorded as known limitations.
*Rationale:* a named, external standard makes "accessible" checkable instead
of aspirational, and "where feasible" keeps it honest for a native app.
*Rejected:* no stated standard (unverifiable); claiming full AA compliance
(untrue and would hide real gaps).

### D-026 — Security and quality scanning in CI: gitleaks, CodeQL, cargo-deny, CycloneDX SBOM

The CI pipeline gains four gates: **gitleaks** (secrets scan on history and
diff), **CodeQL** (static analysis / SAST), **cargo-deny** (dependency
licence-class and advisory checks), and a **CycloneDX SBOM** generated and
attached to releases. New dependencies must fall within the allowed licence
classes (MIT/Apache-2.0) and be recorded in `ATTRIBUTION.md`.
*Rationale:* these automate the checks the process already promised (no
secrets, permissive licences, no known advisories, a reproducible component
list) and make them evidence rather than intention. *Rejected:* manual-only
checks (easily forgotten, unverifiable); no SBOM (poor supply-chain
visibility).

### D-027 — Dependabot enabled for dependency updates

Dependabot is enabled for the repositories' Cargo and GitHub Actions
dependencies, opening update pull requests that run through the same CI and
review gates as any other change. *Rationale:* it keeps dependencies current
and surfaces security advisories early at no ongoing effort, and the normal
PR flow keeps each update reviewed rather than auto-merged.
*Rejected:* manual dependency tracking (easy to let drift); auto-merge of
Dependabot PRs (bypasses review).
