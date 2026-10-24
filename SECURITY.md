# Security policy

Engram is a local-first desktop application. It has no server component and
embeds no privileged tokens, so the attack surface is small — but small is
not none, and security reports are taken seriously.

## Supported versions

Engram is pre-1.0. Security fixes are made on `main` and ship in the next
tagged release; only the latest release is supported. If you are on an older
release, please reproduce against the latest one first.

## Reporting a vulnerability

Please report security issues **privately**. Do not open a public issue,
pull request or discussion, and do not disclose the problem publicly until a
fix has been released.

Use GitHub's private vulnerability reporting:

- Open the repository's **Security** tab and choose **Report a
  vulnerability**, or
- start a draft advisory directly at
  <https://github.com/Perchemop/engram/security/advisories/new>.

A good report includes:

- The Engram version and the operating system and version.
- What an attacker can do, and the impact you believe it has.
- Step-by-step reproduction, or a minimal `.engram.json` / input that triggers
  the issue.
- Any proof-of-concept, logs or crash reports (with any personal paths
  removed).

## What to expect

- **Acknowledgement within seven days.** Engram is maintained by one person
  in a few hours a week; the seven-day window is a firm commitment, not a
  guarantee of an instant fix.
- **Coordinated disclosure.** The maintainer will confirm the issue, judge
  its severity, prepare a fix, and agree a disclosure date with you. Please
  keep the report private until a fixed release is published.
- **Credit.** Reporters are credited in the advisory unless they ask not to
  be.

If the issue turns out to be a design limitation or a duplicate, you will be
told, with the reasoning.

## Scope

In scope:

- The Engram desktop application on Windows, macOS and Linux.
- Parsing and loading of the `.engram.json` library file, CSV import and JSON
  backups.
- The optional update check and the crash-report submission path.
- The CI and release pipeline in `.github/workflows/`.

Out of scope (report these elsewhere):

- Operating-system, GPU-driver and window-manager bugs.
- Bugs in upstream crates, especially `egui` and `eframe`; please report
  those to the upstream project, and mention the Engram version so it can be
  tracked here if it affects us.
- Issues that require physical access to an unlocked machine, or that rely
  on social engineering.
- The security of GitHub itself, TLS/PKI, and the user's own machine and
  account.

## Lightweight threat model

What Engram does and what it trusts:

- **Local, plain data.** All user data is a single human-readable JSON file
  the user owns and can read, edit, back up or delete. There is no server,
  no account and no telemetry.
- **No embedded credentials.** The binary contains no tokens, API keys or
  secrets; there is nothing privileged to extract from it.
- **Two optional network paths, both opt-in and fail-soft.** The update check
  reads the public GitHub Releases API over HTTPS; the crash-report
  submission opens a pre-filled GitHub issue in the user's own browser, where
  the user signs in themselves. Neither path runs unless the user triggers
  it (or has left the daily update check enabled), and both fail silently
  when offline.

How the project could be misused, and the guarantees we intend to hold:

- **A malicious `.engram.json` must never cause code execution or a freeze.**
  The file is parsed with Rust's `serde`/`serde_json`; there is no
  evaluation, no scripting and no dynamic code loading. Malformed or hostile
  input should produce a clear error and a safe state, not a hang or a crash.
  A file that can freeze or crash the app is treated as a security-relevant
  robustness bug.
- **The update check must not be a code-execution path.** It reads a version
  string from the Releases API, compares it and shows a notification with a
  link. It does not download, install or execute anything.
- **Crash reports are local by default.** A report is written to disk and
  shown to the user for review; nothing is transmitted until the user
  chooses to submit, and home paths are scrubbed from the backtrace.
- **Contributor-supplied code is reviewed and CI-checked before it reaches
  `main`.** See `CONTRIBUTING.md` and `GOVERNANCE.md`.

## Incident response plan

1. **Triage.** The maintainer is the sole responder. A private report is
   acknowledged within seven days and reproduced.
2. **Severity.** Judged from impact and reach:
   - **Critical** — code execution, data loss without user action, or
     exposure of user data over the network.
   - **High** — a crash or freeze reachable from ordinary use, or a clear
     correctness/robustness failure on untrusted input.
   - **Medium** — a bug that degrades safety or privacy with some user
     interaction, or a hardening gap.
   - **Low** — defence-in-depth, best-practice and hardening findings with no
     direct exploit.
3. **Fix.** The fix is prepared privately (in a security-advisory fork or a
   private branch), reviewed, and verified by the CI gates described in
   `CONTRIBUTING.md`. Direct pushes to `main` are not used.
4. **Disclose.** A GitHub **Security Advisory** is published with the fix,
   and the fix ships in a tagged release whose notes reference the advisory.
   Users are notified through the advisory and the release; the reporter is
   credited unless they prefer otherwise.

## No secrets

The project holds a hard rule that no secrets, tokens or credentials are ever
committed (see `DECISIONS.md`, D-019). If you find a credential in the
repository or in a release artifact, report it privately as above and treat
it as high severity.
