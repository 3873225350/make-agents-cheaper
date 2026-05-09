# Evaluation Metrics

`make-agents-cheaper` should prove two things at the same time:

1. The agent workflow becomes cheaper.
2. The agent does not become worse at the task.

The project should not claim that it reduces total context. The main target is lower paid uncached input.

Token usage is the primary metric. Provider price is useful context, but it is secondary and can change.

## Motivation From Tracing

The motivation did not start as "make agents cheaper." It started from trying to understand why DeepSeek v4-style systems make tokens cheaper, what cache hit means, and what part of that is reachable outside model training.

Agent trace and observability work such as `claude-trace` and `codex-trace` made that question concrete. Traces show that a coding-agent request is a harness product: stable instructions, tool schemas, repository policy, transport metadata, session routing, and dynamic task content are assembled into a single model call.

This makes prefix stability observable. It also makes cache-hit behavior something we can evaluate instead of only guess.

## Evaluation Positioning

This project evaluates what can be changed outside the model, not model-side training efficiency.

```text
model-side efficiency:
  training, distillation, sparse inference, serving kernels, batching

harness layer:
  stable provider, stable transport, stable session route, stable prompt prefix
```

The benchmark should therefore compare two agent harness configurations while keeping the model family and task semantics fixed.

## Implemented CLI Checks

The first implementation provides six executable pieces:

| Command | Purpose | Framework change needed? |
| --- | --- | --- |
| `init-experiment` | Create an append-only experiment log directory | No |
| `fingerprint` | Hash prompt/harness layers and compare drift | No, if traces or layer exports exist |
| `tool-schema` | Hash tools and detect schema order drift | No for detection; yes for full control |
| `breakpoints` | Inspect explicit `cache_control` breakpoint placement | No for trace analysis; yes to choose breakpoints |
| `eval` | Compare baseline vs cache-friendly JSONL runs, optionally with provider prices | No |
| `task-report` | Print per-task token usage and cache-hit rows | No |
| `compact-template` | Print a stable-first reactivation template | No |

The skill layer should start with detection and measurement. A native agent framework such as `cheapcode` can later move these policies into prompt assembly, tool registry, and session lifecycle.

## Prefix Fingerprint Input

`fingerprint` expects either a JSON object or an object with a `layers` object:

```json
{
  "layers": {
    "system_policy": "stable system text",
    "tool_schema": [{ "name": "read" }, { "name": "edit" }],
    "repo_rules": "AGENTS.md content",
    "active_state": "branch and open files"
  }
}
```

The command prints short SHA-256 fingerprints. These are fingerprints, not prompt text.

## Tool Schema Input

`tool-schema` expects a JSON array or an object with a `tools` array:

```json
{
  "tools": [
    { "name": "apply_patch", "description": "Apply edits" },
    { "name": "exec_command", "description": "Run commands" }
  ]
}
```

Tool execution order can vary. Tool definition order should stay stable.

## Breakpoint Input

`breakpoints` accepts a simplified `blocks` array or a traced request with `system`, `tools`, and `messages` arrays. It looks for direct `cache_control` markers and reports gaps between breakpoints.

## Primary Cost Metrics

| Metric | Definition | Why it matters |
| --- | --- | --- |
| `input_tokens` | Total provider-reported input tokens | Shows total context sent |
| `cached_input_tokens` | Provider-reported cached input tokens | Direct cache-hit signal |
| `cache_creation_input_tokens` | Provider-reported cache creation tokens when available | Separates cache write from cache read |
| `uncached_input_tokens` | `input_tokens - cached_input_tokens` | Main paid-input target |
| `cache_hit_rate` | `cached_input_tokens / input_tokens` | Normalized cache performance |
| `estimated_actual_cost` | Uncached input + cached input + output cost | Estimated billable cost |
| `estimated_full_cost` | Counterfactual cost if all input were uncached | Baseline for savings |
| `savings_percent` | `1 - actual_cost / full_cost` | Estimated cache savings |
| `actual_cost_usd` | Provider/CLI reported actual cost, when available | Best cost signal if the route exposes it |

Key wording:

```text
It reduces paid uncached input, not necessarily total input.
```

CLI cost estimation uses provider prices supplied at run time:

```bash
cargo run --quiet -- eval \
  --baseline baseline.jsonl \
  --candidate cache-friendly.jsonl \
  --uncached-input-per-mtok <USD> \
  --cached-input-per-mtok <USD> \
  --output-per-mtok <USD>
```

Do not hard-code stale provider prices in the repo. Fill these values from the current route/provider pricing page when running the experiment.

## Latency Metrics

| Metric | Definition |
| --- | --- |
| `ttft_ms` | Time to first token |
| `model_latency_ms` | Model-call wall time when separable |
| `total_latency_ms` | Full end-to-end turn latency |
| `task_wall_time_ms` | Complete task time including tools and tests |

Prompt cache hits should primarily improve repeated prefill cost, so `ttft_ms` is especially important.

## Quality Metrics

| Metric | Definition |
| --- | --- |
| `task_success` | Whether the task was completed under a fixed rubric |
| `validation_passed` | Whether the task-specific test command passed |
| `turns_to_completion` | Number of user/agent turns |
| `tool_calls` | Number of tool calls |
| `intended_files_changed` | Count of expected files changed |
| `unintended_files_changed` | Count of unrelated files changed |
| `review_score` | Human or LLM reviewer score for correctness and minimality |

A cache-friendly run is only a win if cost drops without a meaningful quality regression.

## A/B Conditions

Use the same task prompt, repository snapshot, model family, validation command, and success rubric.

Baseline:

```text
ordinary setup, no enforced cache-friendly invariants
```

Cache-friendly:

```text
stable provider
stable model
stable reasoning effort
Responses transport
WebSocket/session continuity where available
no prompt rewriting
```

## JSONL Run Record

```json
{
  "task_id": "rust-cli-add-version",
  "run_id": "2026-05-09-cache-friendly-01",
  "condition": "cache_friendly",
  "turn_index": 3,
  "agent": "codex",
  "model": "gpt-5.4",
  "transport": "responses_ws",
  "input_tokens": 82000,
  "cached_input_tokens": 76000,
  "cache_creation_input_tokens": 0,
  "output_tokens": 3000,
  "ttft_ms": 1200,
  "total_latency_ms": 24000,
  "actual_cost_usd": 0.014073,
  "tool_calls": 5,
  "validation_command": "cargo test --locked",
  "validation_passed": true,
  "task_success": true
}
```

## Minimum Result Table

| Metric | Baseline | Cache-friendly |
| --- | ---: | ---: |
| Prompt cache hit rate | 18.2% | 76.9% |
| Uncached input tokens | 1.00x | 0.43x |
| Estimated cost | 1.00x | 0.57x |
| Median time to first token | 1.00x | 0.69x |
| Task success | 17/20 | 17/20 |
| Median turns | 4 | 4 |

The core claim is valid only when:

```text
cached tokens go up
uncached input goes down
estimated cost goes down
latency does not regress
task success does not regress
```
