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

Full transitive dependency licences can be audited with
`cargo license` or by inspecting `Cargo.lock`.

## Algorithms and conventions (public knowledge)

- **SuperMemo-2 (SM-2)** scheduling algorithm — described publicly by
  Piotr Wozniak (1990); Engram implements it from the public description
  with documented extensions (learning steps, ease clamp, easy bonus).
- **`{{cN::…}}` cloze markup** — a long-standing public interchange
  convention for cloze deletions in plain text.

## CI tooling (GitHub-hosted actions)

| Action | Licence |
|---|---|
| actions/checkout | MIT |
| dtolnay/rust-toolchain | MIT OR Apache-2.0 |
| Swatinem/rust-cache | MIT |

## Inspiration

Other products influenced Engram only at the capability level; see
`docs/inspiration.md`. No source code, assets, strings, layouts or internal
algorithms from any commercial product were read, copied or derived.
