# Current Model Map

Which free model is assigned to which subagent role for the current session.
Updated whenever the available free models change (guide §5.2).

## Provider order (guide §5)

1. **OpenCode free models** (default).
2. **NVIDIA free models** — used automatically when OpenCode free is
   exhausted, via the configured failover integration.
3. **Paid API keys** (Groq / Google) — last resort only.

No local model hosting (Ollama or similar) is used. Only the provider
changes between tiers; the process, roles, vertical-slice discipline,
CI-first rule and tracking files stay identical.

## OpenCode free (active, discovered 2026-10-23)

Available at discovery: `opencode/big-pickle`, `opencode/beam-free`,
`opencode/deepseek-v4.1-flash-free`, `opencode/glm-5.3-flash-free`,
`opencode/mimo-v2.6-flash-free`, `opencode/nemotron-3.5-super-vl-free`,
`opencode/flint-40b-free`.

| Role | Model |
|---|---|
| Orchestrator / lead (plans & delegates, never edits) | `opencode/big-pickle` |
| Domain dev (pure Rust types/logic/tests) | `opencode/deepseek-v4.1-flash-free#high` |
| UI dev (egui/eframe slices) | `opencode/beam-free#high` |
| Fixer (build-gate failures) | `opencode/glm-5.3-flash-free#high` |
| Reviewer (read-only review) | `opencode/glm-5.3-flash-free#high` |
| Test writer | `opencode/mimo-v2.6-flash-free` |
| Docs writer | `opencode/big-pickle` |
| Explorer (fast search) | `opencode/mimo-v2.6-flash-free` |
| CI runner (GitHub interaction only) | `opencode/beam-free#high` |

Model IDs are not load-bearing: the orchestrator re-discovers the currently
available free models at session start and reassigns the best fit per role
(guide §5.2). The `model:` values in `.opencode/agents/*.md` are defaults the
orchestrator overrides, and delegation passes `model` explicitly.

## NVIDIA free (fallback, discovered 2026-10-23)

All zero-cost. Assign on exhaustion of the OpenCode tier:

- `nvidia/nvidia/nemotron-3.5-super-vl-120b-a12b` — strong reasoning/planning
- `nvidia/deepseek-ai/deepseek-v4.1-flash` — coding (domain/fixer)
- `nvidia/z-ai/glm-5.3-flash` — coding / fixer
- `nvidia/z-ai/glm-5.3` — heavier coding
- `nvidia/openai/gpt-oss-20b` — general / review
- `nvidia/stepfun-ai/step-5-preview` — large-context coding
- `nvidia/poolside/laguna-xs-2.1` — code completion
- `nvidia/meta/llama-guard-4-12b` — light/quick tasks

When switching: note the exhaustion in `PROGRESS.md`, re-discover NVIDIA
models, reassign per role, and continue the same slice pipeline unchanged.
