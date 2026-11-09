#!/usr/bin/env bash
# Personal-data scan (guide II.29, owner directive 2026-10-27): fails when a
# real-looking personal value is committed. It is a required status check
# ("Personal data scan") so a leak cannot merge.
#
# Usage:
#   bash scripts/scan-personal-data.sh              # scan git-tracked files
#   bash scripts/scan-personal-data.sh --path DIR   # scan a directory (self-test)
#
# Scans for:
#   * an absolute home path for a real user: /home/<name>, /Users/<name>
#     (macOS), C:\Users\<name>. The CI build users /home/runner, /Users/runner
#     and C:\Users\runner are permitted (guide II.29.6).
#   * an email address that is not obviously synthetic (@example.*, or any
#     domain under the RFC 2606 reserved TLDs .example/.invalid/.test/
#     .localhost) or a GitHub noreply address.
#
# Dependency notice files that legitimately carry an upstream author's address
# are third-party content (guide II.29.2); if such a file is ever added, the
# scan fails loudly and a human classifies the hit rather than the check
# guessing. This script contains the patterns themselves, so it is excluded
# from the tracked scan. The same script scans release artefacts via --path
# (release.yml, audit finding pii-scan-coverage-gap).
set -euo pipefail

cd "$(dirname "$0")/.."

MODE="git"
SCAN_ROOT=""
if [ "${1:-}" = "--path" ]; then
    MODE="path"
    SCAN_ROOT="${2:?usage: scan-personal-data.sh [--path <dir>]}"
fi

# Path lists. Home paths are scanned with -a because a build path can be
# embedded in a committed binary (a .pyc, for example); the literal prefix
# keeps false positives out. Emails are scanned the same way for the same
# reason.
HOME_RE='/home/[A-Za-z0-9_.-]+|/Users/[A-Za-z0-9_.-]+|C:\\Users\\[A-Za-z0-9_.-]+'
# /home/user, /Users/user and C:\Users\user are additionally allowed: the
# generic placeholder username reveals nothing personal, and it appears in
# path-scrubbing tests (guide II.29.6) and documentation examples.
# /home/runner, /Users/runner, C:\Users\runner and the -admin variants are
# the GitHub Actions runner users: they appear inside release binaries
# (build-machine paths baked in by the toolchain) and carry no personal
# data. Classified 2026-11-09 when the release-artifact scan (finding
# pii-scan-coverage-gap) first ran on dist/.
HOME_ALLOW='(/home/runner|/Users/runner|C:\\Users\\runner|/home/runneradmin|/Users/runneradmin|C:\\Users\\runneradmin|/home/user|/Users/user|C:\\Users\\user)$'
EMAIL_RE='[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}'
# Synthetic addresses and GitHub noreply are allowed everywhere. In
# --path (release-artifact) mode, upstream third-party notice addresses
# baked into vendored/static dependency code (e.g. OpenSSL author strings
# inside a vendored crate) are third-party content per guide II.29.2: the
# addresses below were classified on 2026-11-09 when the first artifact
# scan ran. A NEW address in an artifact still fails loudly and must be
# classified by a human - never grow this list casually.
EMAIL_OK_CORE='(example\.(com|org|net)|users\.noreply\.github\.com|noreply\.github\.com|[A-Za-z0-9.-]+\.(example|invalid|test|localhost))'
EMAIL_OK_ARTIFACTS_CORE='(appro@openssl\.org|info@e-szigno\.hu|dave@understandinglimited\.com|jimmac@gmail\.com)'

list_files() {
    if [ "$MODE" = path ]; then
        find "$SCAN_ROOT" -type f -print0
    else
        git ls-files -z -- ':!scripts/scan-personal-data.sh'
    fi
}

home_hits="$(
    list_files | while IFS= read -r -d '' f; do
        [ -f "$f" ] || continue
        grep -HnaoE "$HOME_RE" "$f" 2>/dev/null || true
    done | grep -vE "$HOME_ALLOW" || true
)"

email_allow="@$EMAIL_OK_CORE\$"
if [ "$MODE" = path ]; then
    # Release-artifact mode: upstream third-party notice addresses baked
    # into vendored dependency code are classified third-party content
    # (guide II.29.2); they are skipped here but must never appear in a
    # tracked tree file, where the plain allowlist still applies.
    email_allow="(@$EMAIL_OK_CORE|$EMAIL_OK_ARTIFACTS_CORE)\$"
fi

email_hits="$(
    list_files | while IFS= read -r -d '' f; do
        [ -f "$f" ] || continue
        grep -HnaoE "$EMAIL_RE" "$f" 2>/dev/null || true
    done | grep -vEi "$email_allow" || true
)"

fail=0
if [ -n "${home_hits//[[:space:]]/}" ]; then
    echo "ERROR: file(s) contain a non-CI absolute home path:" >&2
    echo "$home_hits" >&2
    fail=1
fi
if [ -n "${email_hits//[[:space:]]/}" ]; then
    echo "ERROR: file(s) contain a non-synthetic email address:" >&2
    echo "$email_hits" >&2
    fail=1
fi
if [ "$fail" -ne 0 ]; then
    echo "Classify each hit (guide II.29.2): synthetic/CI, or a finding to fix." >&2
    exit 1
fi

if [ "$MODE" = path ]; then
    echo "personal-data scan: clean ($SCAN_ROOT)"
else
    echo "personal-data scan: clean (no non-CI home path, no real email)"
fi
