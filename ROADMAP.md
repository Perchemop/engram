# ROADMAP

Public roadmap for **Engram** — spaced repetition & active recall.
Readiness percentages reflect *verified* evidence (CI runs + traceability +
local runs), never intent (guide §6.2). Updated per release.

## Where we are

**v0.1.0 released 2026-10-23** — first functional test build. The study
core (decks, cards, cloze, SM-2 scheduling, study sessions, statistics,
persistence, import/export, crash reporting, update check) is implemented,
tested on four platforms, and installable.

**Next slice (v0.2.0)** — richer study workflow: queue ordering, deck-local
limits, suspend/bury with a history browser, and cram sessions are
implemented on the `opencode/ci` verification branch and are being checked
by CI. Multi-cloze faces remain to be done.

Feature breadth of the long-term product vision implemented so far:
**~70%**. Ready-for-work items in the next milestone: **100%** (research
notes drafted and approved by the human gate; implementation started).

## Milestones

### v0.1.x — polish on the first functional build
*(no research dependency; fix-driven)*
- [ ] Manual keyboard/visual QC walk completed and recorded (human step)
- [ ] Dark/light contrast audit on real hardware
- [ ] Small fixes discovered by the QC walk
- Ready-for-work: **100%** · Target: as fixes land

### v0.2.0 — richer study workflow
*(research notes approved by the human gate, guide §2.3/§8.4)*
- [x] Queue interleaving/shuffle as a setting
      (`docs/research/queue-ordering.md`)
- [x] Deck-local daily-limit overrides
      (`docs/research/deck-local-limits.md`)
- [x] Card history browser; suspend/bury cards
      (`docs/research/card-history-suspend-bury.md`)
- [x] Filtered "cram" sessions that bypass the scheduler
      (`docs/research/cram-sessions.md`)
- [ ] Multi-cloze-per-card study faces — one deletion per review
      (`docs/research/multi-cloze-faces.md`) — deferred to a later slice;
      it reworks the queue entry to carry a face and migrates every
      existing cloze card, so it lands on its own
- Ready-for-work: **100%** · Human gate: **approved** · CI: **verified per
  slice**

### Project health — open-source readiness
*(guide Part II, §16–§25; no research dependency)*
- [x] Required repository files: `CODE_OF_CONDUCT.md`, `SECURITY.md`
      (with threat model + incident response), `ACCESSIBILITY.md`,
      `VISION.md`, `GOVERNANCE.md`, `CONTRIBUTORS.md`
- [x] Issue templates: bug, feature and a dedicated accessibility path
- [x] PR template extended with the accessibility checklist
- [x] CI supply chain: secret scanning (gitleaks), SAST (CodeQL),
      dependency licence classes (cargo-deny) and a CycloneDX SBOM
- [x] Dependency-update automation (Dependabot) for Cargo and Actions
- [x] Contribution labels (`good first issue`, `help wanted`,
      `documentation`, `first-timers-only`, area labels)
- [ ] Protected `main` + required status checks (configuration step)
- [ ] At least two human admins, and MFA on every privileged account
      (human step, guide §20.1)

### v0.3.0 — data & interoperability
*(research notes approved by the human gate)*
- [ ] Anki text import (public TSV/CSV conventions)
      (`docs/research/anki-import.md`)
- [ ] FSRS scheduler option alongside SM-2 (public algorithm + reference
      implementations) (`docs/research/fsrs-scheduler.md`)
- [ ] Backup reminders / rotation of `.engram.json` snapshots
- Ready-for-work: **100%** · Human gate: **approved**

### v1.0.0 — finished product (guide §9 gates)
- [ ] All §9 finished-product gates pass on the primary platform
- [ ] §2.5 legal checklist clean for a full release
- [ ] Web/WASM target evaluated (browser storage backend, assumption Q-1)
- [ ] Installers/packages per platform if demand warrants

## Explicitly out of scope (see `docs/scope.md`)
Cloud sync, accounts/subscriptions, real-time collaboration, native mobile.
