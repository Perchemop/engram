# Repository radar

Periodic build-vs-adopt pass over public GitHub projects (guide.md §4A). One
section per pass; verdicts are `adopt`, `pilot`, `watch` or `reject`, and a
rejected candidate is not re-proposed unless its reason has changed.

## 2026-10-25 — first pass

Scope: the open v0.3.0 items (Anki import, FSRS option) plus release, testing
and supply-chain infrastructure.

Already covered in this repository (no action, do not re-propose): `cargo-deny`,
gitleaks secret scanning, CycloneDX SBOM, CodeQL, Dependabot, SHA-pinned
actions, checksummed release assets.

| Candidate | Verdict | Reason |
| --- | --- | --- |
| `axodotdev/cargo-dist` | **adopt** | Replaces the hand-maintained release matrix with a declarative target/installer list. Adopted incrementally: `dist-workspace.toml` + `dist plan` on every pull request; the release workflow stays hand-written and SHA-pinned (`allow-dirty = ["ci"]`). |
| `sourcefrog/cargo-mutants` | **adopt** | No mutation testing today, and coverage alone cannot show the scheduler assertions bite. Weekly, scoped to `src/domain/scheduler.rs` and `src/domain/session.rs`. |
| `open-spaced-repetition/fsrs-rs` | **reject (for now)** | `DECISIONS.md` already defers FSRS to the v0.3.0 slice: it needs a scheduler abstraction, a migration and a review-interval model, not a drop-in dependency. Revisit when the v0.3.0 scheduler slice starts, and then as a **pilot** behind the existing SM-2 path. |
| `tazz4843/whisper-rs` | **reject** | Archived (last push 2025-07-30); not relevant to this app. |

Re-check date for the radar as a whole: 2027-01-25.
