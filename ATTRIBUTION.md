# Attribution

Engram's code, documentation, icon and visual design are original work.

## Runtime dependencies (crates.io)

| Crate | Licence | Purpose |
|---|---|---|
| eframe 0.36 | MIT OR Apache-2.0 | Native window / app shell |
| egui 0.36 | MIT OR Apache-2.0 | Immediate-mode UI toolkit |
| serde 1 | MIT OR Apache-2.0 | Serialisation framework |
| serde_json 1 | MIT OR Apache-2.0 | JSON persistence |
| uuid 1 | MIT OR Apache-2.0 | Identifiers (v4) |
| thiserror 2 | MIT OR Apache-2.0 | Error derive |
| tracing 0.1 | MIT OR Apache-2.0 | Structured logging facade |
| tracing-subscriber 0.3 | MIT | Logging output |
| directories 6 | MIT OR Apache-2.0 | OS data-directory lookup |
| chrono 0.4 | MIT OR Apache-2.0 | Dates, times, durations |
| rfd 0.15 | MIT | Native file dialogs |
| ureq 3 | MIT OR Apache-2.0 | HTTPS client for the optional update check |
| semver 1 | MIT OR Apache-2.0 | Version comparison for the update check |

Full transitive dependency licences can be audited with
`cargo license`, `cargo-deny`, or by inspecting `Cargo.lock` and the
CycloneDX software bill of materials produced in CI.

## Algorithms and conventions (public knowledge)

- **SuperMemo-2 (SM-2)** scheduling algorithm — described publicly by
  Piotr Wozniak (1990); Engram implements it from the public description
  with documented extensions (learning steps, ease clamp, easy bonus).
- **`{{cN::…}}` cloze markup** — a long-standing public interchange
  convention for cloze deletions in plain text.

## Non-code assets

All of the following are original to Engram and released under the project's
licence, MIT.

| Asset | Origin | Licence |
|---|---|---|
| Application icon | Procedurally generated at runtime in `src/ui/icon.rs`; master vector in `assets/engram.svg`; original design (see `docs/design/brand.md`) | MIT |
| Colour tokens | Original palette defined in `src/domain/deck.rs` and `src/ui/theme.rs`; not derived from any other product | MIT |
| Fonts | No font files are bundled. Text uses the default fonts shipped by egui (its proportional, monospace and emoji faces, e.g. `Ubuntu-Light`, `Hack` and the Noto/emoji fonts in egui 0.36), each under its own permissive licence in the egui distribution | MIT OR Apache-2.0 (upstream) |

## Project licence

Engram is licensed under **MIT only** — see `LICENSE`. It is deliberately
not dual-licensed as `MIT OR Apache-2.0` (the decision and rationale are
recorded in `DECISIONS.md`, D-023). This file and the CI software bill of
materials report the project's own licence as MIT; the dependency table above
lists each crate's own licence, which is often the crate's choice of
`MIT OR Apache-2.0`.

## Build provenance and model/service terms

- **Original work only.** All Engram code, documentation, strings, layouts and
  assets are original, written from public standards, this project's own
  specification, and capability-level inspiration (`docs/inspiration.md`). No
  proprietary source, decompiled material or copied expression was used as
  input.
- **AI model and API services used during the build were free tiers** — the
  free models available through OpenCode, and the free NVIDIA API as the
  fallback tier (`docs/current-model-map.md`). These providers' terms permit
  use of the outputs of the free tiers, and no model output whose terms
  forbid redistribution has been committed.
- **Every committed change is reviewed and CI-verified** under the same gates
  as human work (`docs/development.md`), and commits are authored under the
  project's build identity (`CONTRIBUTORS.md`).

## Inspiration

Other products influenced Engram only at the capability level; see
`docs/inspiration.md`. No source code, assets, strings, layouts or internal
algorithms from any commercial product were read, copied or derived.
