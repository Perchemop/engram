# Accessibility

Engram is a study tool. If it cannot be used comfortably, it fails at its
one job — so accessibility is treated as a correctness property of the UI,
not as a nice-to-have.

## Commitment

The project commits to:

- A keyboard-first interface: every core task is reachable without a mouse.
- Associating accessibility work with the same seriousness as other bugs,
  with its own reporting path and severity scale.
- Documenting known gaps honestly rather than claiming compliance it cannot
  evidence.
- Treating accessibility regressions as defects, not as documentation
  footnotes.

## Target standard

The target is **WCAG 2.2 Level AA where feasible** for a native desktop
application. "Where feasible" is deliberate: egui is an immediate-mode
toolkit, some WCAG success criteria are written for web content, and a few
cannot be met exactly by a native renderer. Where a criterion cannot be met,
the limitation is recorded below rather than glossed over.

## Primary priorities

1. **Keyboard support.** Every flow (create a deck, create a card, study,
   grade, open and dismiss dialogs) must be completable from the keyboard.
   Focus order must be sensible and focus must be visible.
2. **Screen-reader semantics.** egui integrates **AccessKit**, which exposes
   a semantic tree to screen readers on supported platforms (Windows and
   macOS today). Interactive widgets must carry meaningful roles, names and
   values.
3. **Colour contrast.** Text and meaningful UI must meet WCAG AA contrast in
   both the dark and light themes, and colour must never be the only signal
   for state (grades, due counts, deck colours).
4. **Reduced motion.** Where the platform reports a reduced-motion
   preference, animations and transitions must be minimised or disabled.

## Known limitations

These are real, current gaps. Each names the workaround or the plan.

- **The statistics heatmap is custom-painted and exposes no per-cell
  semantics.** The grid is drawn with immediate-mode shapes, so a screen
  reader cannot read individual day counts from it. Workaround: the same
  period is summarised in text tiles (retention, streak, due forecast) above
  the heatmap. Plan: expose an accessible summary or a table view of the
  same data.
- **No automated assistive-technology (AT) test runs in CI.** Screen-reader
  behaviour is verified by manual spot-checks, not by an automated suite.
  This means AT regressions rely on manual testing and on user reports; the
  dedicated accessibility issue template exists for exactly this reason.
- **AccessKit coverage depends on the platform.** Semantics are exposed where
  AccessKit has a bridge (notably Windows and macOS); on other platforms the
  semantic tree may not reach the system AT.
- **Native file dialogs are provided by the operating system** (`rfd`) and
  inherit their own accessibility, which is outside Engram's control.

## Supported environments

- Windows 10 and later, with Narrator or NVDA.
- macOS, with VoiceOver.
- Linux desktop, with a caveat: AccessKit bridging to AT-SPI is not something
  the project can currently verify, so Linux screen-reader support is
  best-effort. Keyboard-only use is supported on all platforms.

## How accessibility bugs are graded

- **Critical** — the app cannot be used at all with a screen reader or
  keyboard, or a core task is impossible for an assistive-technology user,
  with no workaround.
- **High** — a core task is very difficult or has only a cumbersome
  workaround; focus is lost or trap-prone; content is unreadable due to
  contrast.
- **Medium** — a non-core task is affected, or a core task has a reasonable
  workaround; missing labels on secondary controls.
- **Low** — polish: minor wording, ordering or annoyance that does not block a
  task.

## Reporting an accessibility bug

Use the dedicated accessibility template,
[`.github/ISSUE_TEMPLATE/accessibility.yml`](.github/ISSUE_TEMPLATE/accessibility.yml).
It takes a separate path from general bug reports because accessibility
issues are time-sensitive, need different context (assistive technology and
version, not just an OS), and are graded on the scale above.

When a reported accessibility issue is fixed, the reporter is told in the
issue. Accessibility acceptance criteria are required for any change that
touches the UI; see `docs/development.md` and the pull-request template.
