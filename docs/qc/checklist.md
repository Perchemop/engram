# QC Checklists (guide §9)

Run these in order per phase / release. Record each run (date, phase, CI
run ID, outcome) in `PROGRESS.md`; never claim completion without a green
CI run.

## Per slice (Definition of Done)

- [ ] Feature slice is vertical (domain + tests + UI + docs in one slice).
- [ ] Unit tests cover happy path, boundary, and failure/edge cases.
- [ ] Headless domain tests need no display; no CPU-heavy work runs locally.
- [ ] Code is rustfmt-clean (CI runs `cargo fmt --all -- --check`; cargo
      is banned locally).
- [ ] CI green on GitHub Actions: fmt → clippy `-D warnings` → test --all
      (Linux/Windows/macOS) → release build.
- [ ] No fake success states / dead controls / placeholder-only data.
- [ ] `CHANGELOG.md`, `PROGRESS.md` updated with **exact** CI results.
- [ ] `REQUIREMENTS_TRACEABILITY.md` statuses match reality.
- [ ] Change is small and reviewable.

## Per phase

- [ ] All phase requirement IDs traced → `DONE` with CI evidence.
- [ ] `REQUIREMENTS_TRACEABILITY.md` ↔ `IMPLEMENTATION_PLAN.md` consistent
      (no orphan IDs).
- [ ] Open assumptions reviewed in `docs/assumptions-and-questions.md`
      (answer or mark still-open with Q-IDs).
- [ ] DECISIONS.md has an entry for every user directive this phase.

## Release / packaging

- [ ] Cross-platform build artifacts verified (Linux/Windows/macOS) —
      every CI run attaches `engram-linux` / `engram-windows` /
      `engram-macos` plus the `Cargo.lock` used for the build.
- [ ] App launches on a clean machine; save → reopen → identical state.
- [ ] Version bumped; CHANGELOG has a release entry.
- [ ] No secrets/keys committed; no local-only files in the tree.
- [ ] LICENSE present; `ATTRIBUTION.md` complete and accurate.

## Release legal checklist (guide §2.5, run before every public release)

- [ ] No proprietary source, assets, or substantial copied text in tree
- [ ] No reverse-engineering output used as implementation input
- [ ] Name/logo original; no vendor marks in UI or installers
- [ ] UI is original expression (no slavish visual copy)
- [ ] No known patented techniques implemented where a public alternative exists
- [ ] Licence clear (MIT only — D-023); `ATTRIBUTION.md` complete and accurate
- [ ] No secrets in git or release binaries
- [ ] Crash reports sent only with user consent; privacy documented in README
- [ ] No embedded privileged tokens (issue submission is a pre-filled URL)
- [ ] Research notes are original summaries + URLs only
- [ ] README/ROADMAP make no false affiliation or "drop-in replacement" claims
- [ ] CONTRIBUTING.md states PR licence terms

## Originality attestation (every phase)

- [ ] All code, strings, icons, layouts and algorithms are original —
      derived from public standards, our own spec, and capability-level
      inspiration (`docs/inspiration.md`).
- [ ] No proprietary code or assets; dependencies listed in
      `ATTRIBUTION.md`.
- [ ] No absolute local paths or references to local-only files in
      code/docs.

## Manual QC spot-checks (when UI is touched)

### 1. Decks page
- [ ] Create/rename/move/delete deck works; duplicate sibling names are
      rejected with a visible error; deleting shows a confirm prompt that
      states the re-parenting semantics.
- [ ] Card table: search filters live; tag toggles AND correctly; row edit
      opens prefilled editor; delete removes card and its log rows.
- [ ] Counts shown on rows match the due queue.

### 2. Study page
- [ ] "Start studying" disabled when nothing is due; scope picker counts
      match the deck tree.
- [ ] Space/Enter reveals; 1–4 grades; buttons show the interval each grade
      produces; Esc ends early with an honest status.
- [ ] Progress bar and remaining count update every answer.
- [ ] Summary screen numbers match the session's answers.

### 3. Card editor
- [ ] Type toggle switches front/back ⇄ cloze; cloze preview blanks the
      deletion; empty content is rejected inline (no silent failures).
- [ ] Tags parse (comma/space); deck combo moves the card on save.

### 4. Settings
- [ ] Every DragValue range clamps; step lists parse or keep old values;
      Apply reports success only after a real write.

### 5. Persistence
- [ ] Quit without saving → reopen shows all changes (auto-save).
- [ ] Corrupting the file → app starts with a new library and says so
      (no panic).

### 6. Generate page
- [ ] Paste sample notes → drafts appear; adding N cards really adds N
      rows to the target deck; status never claims more than was added.

### 7. Theme & shell
- [ ] Dark/light switch applies immediately and survives restart (prefs).
- [ ] Every menu item performs its action.

### Run log

| Date | Scope | CI run | Outcome |
|---|---|---|---|
| 2026-10-23 | Phases 0–3 (full V1) | `39889290098` | ✅ Pass — check + Windows + macOS all green |
| 2026-10-22/23 | Compile bring-up (passes 1–6) | `39846764231` → `39889290098` | 5 red runs fixed via CI logs only (see PROGRESS.md) |
| 2026-10-23 | Review-fix slice (PR #1) | PR `39938361670` · main `39939371572` | ✅ Pass on all three runners; PR merged (D-013) |
| 2026-10-23 | Launch smoke test (arm64 binary, run `39949643789` artefact) | — | ✅ 8s session, no crash; wgpu OK; library-on-first-change behaviour confirmed; human keyboard/visual walk still pending |

## Accessibility smoke test (run when UI is touched)

Run this with the keyboard only, then once with a screen reader. Record the
result in the run log above. See `ACCESSIBILITY.md` for the severity scale.

- [ ] **Keyboard-only walk-through** of the main flows, mouse untouched:
      create a deck, create a Basic card and a Cloze card, start and study a
      session, and grade every card using `1`–`4`.
- [ ] **Dialogs:** every dialog can be opened from the keyboard and dismissed
      with `Esc`; focus is visible throughout and never gets trapped.
- [ ] **Screen-reader spot check:** with Narrator, NVDA or VoiceOver, the
      focusable controls announce a meaningful name and role, and the study
      reveal/grade flow is usable.
- [ ] **Reflow / zoom:** at 200% and in a narrow window, no control is
      clipped, overlapped or unreachable, and text does not truncate.
- [ ] **High contrast:** in both themes, text and essential state meet AA
      contrast, and state is not conveyed by colour alone.

## Launch readiness (guide §25.1)

Checked before a public launch or a major release. Items already covered by
the release/packaging and legal sections above are not repeated here.

- [ ] `README.md` answers, in order: what this does, why it is useful, how to
      get started, and where to get more help.
- [ ] `SECURITY.md`, `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`,
      `VISION.md`, `ACCESSIBILITY.md` and `CONTRIBUTORS.md` are present and
      current.
- [ ] The issue forms (`bug_report`, `feature_request`, `accessibility`) and
      the pull-request template are in place and render correctly on GitHub.
- [ ] Branch protection on `main` requires the authoritative CI checks
      (`.ci-required.txt`); direct pushes are disabled.
- [ ] The stated response expectation (a reply within seven days, maintained
      in spare time) is published and honest.
- [ ] Supported platforms and privacy claims in `README.md` match the built
      binaries and the code — no overstatement.
- [ ] No false affiliation or "drop-in replacement" claims anywhere in the
      public docs.
- [ ] Scanner gates are active and green: gitleaks, CodeQL, cargo-deny, SBOM
      generated and attached to the release (D-026).
- [ ] Dependabot is enabled for Cargo and GitHub Actions (D-027).
