# Governance

This document explains how decisions are made in Engram, who can make them,
and how the project's AI-assisted build pipeline fits in. It is written to
be honest about a project that is currently small: one maintainer, a public
repository, and a team of AI build agents working under that maintainer's
direction.

## Model

Engram uses a **BDFL with consensus-seeking** model.

- One maintainer has the final say over the project: its direction, scope,
  releases and the contents of the repository.
- Decisions are reached by seeking consensus first. Ideas, proposals and
  design questions are raised in issues, discussed openly, and only become
  decisions when there is agreement — or, failing that, a reasoned ruling by
  the maintainer.
- The maintainer is a benevolent dictator in name, but the working
  expectation is that they explain their reasoning and prefer the outcome the
  community can live with over the one that merely wins an argument.

## Roles

Roles are defined broadly, because a spare-time project should not gatekeep
participation.

### Users

Anyone who runs Engram and reports a bug, a confusing interaction or an
accessibility problem. Users shape the project by describing real behaviour,
and their reports are the most valuable input it receives.

### Contributors

Anyone who improves the project: code, tests, documentation, translations,
triage, design feedback or research. Contributors work through the pull
request flow described in `CONTRIBUTING.md` and are credited in
`CONTRIBUTORS.md` as their contributions land.

### Maintainers

Maintainers are contributors who have taken on stewardship: review, merge
decisions, release preparation, issue triage and the responsibility of saying
"no" with a reason. Today the project has a single maintainer.

## How decisions are made

1. Discuss in the open: an issue for anything larger than a small fix, and a
   pull request for the change itself.
2. Seek consensus. Prefer the proposal with the broadest agreement, and
   record the trade-offs.
3. If consensus does not emerge and a decision is still needed, the
   maintainer decides and records the decision and its rationale in
   `DECISIONS.md`.

## The AI-agent build pipeline

Engram is largely built by AI build agents under the maintainer's direction,
and this is stated plainly rather than hidden:

- Agents **plan, implement and verify** bounded slices of work under the
  maintainer's instruction. The canonical roles are described in `AGENTS.md`
  and `.opencode/agents/`: an orchestrator plans, a coder implements, a
  reviewer reviews, and a dedicated runner handles GitHub interaction.
- All heavy verification runs in GitHub Actions (see `docs/development.md`),
  and every change goes through the same CI gates and pull request flow as a
  human contribution would.
- **Agents are not "contributors" in the governance sense.** They hold no
  role, no vote and no standing in this document. They act only under the
  maintainer's direction, and the maintainer remains accountable for every
  change, including those proposed by an agent.

## Becoming a maintainer

There is no formal entry exam. A contributor who has repeatedly shown good
judgement — sound pull requests, careful reviews, honest issue triage — may
be invited by the maintainer to help maintain the project, whether as a
second maintainer or as a member of a subteam with a narrower scope (for
example, accessibility, documentation or release packaging). The invitation
comes with the expectations, not the other way around.

## Commercial participation

Contributions are judged on their merits, not on the contributor's employer,
funding or commercial interest. Companies and individuals are welcome to
contribute on exactly the same terms as anyone else. There is no special
treatment, no paid fast lane, and no roadmap priority bought with money.
Anyone contributing in a commercial capacity must ensure they have the right
to do so, and every contribution is accepted under the project's MIT licence
(see `CONTRIBUTING.md`).

## Funding

Engram has no funding today, and none is required for it to function: it is
paid for in spare-time effort and free CI minutes. Should money ever appear,
the first step would be **GitHub Sponsors**, with any use of funds disclosed
publicly. Any arrangement would stay consistent with the non-goals in
`VISION.md` — no paywalled core features, no advertising, no selling of user
data.

## The tiebreaker

Stated explicitly: if consensus-seeking does not produce a decision and one
is still required, **the maintainer's ruling is the tiebreaker.** It is used
as a last resort, not as a matter of routine, and the ruling and its
reasoning are recorded in `DECISIONS.md`.
