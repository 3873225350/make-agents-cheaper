# Evaluation Protocol

This protocol verifies whether a cache-friendly agent setup is actually cheaper without making the agent worse.

Current target scenario:

```text
Claude Code -> mimo-v2.5-pro
```

The protocol is deliberately strict. Do not claim savings unless usage records show cached input or the route exposes enough cache accounting to support the claim.

## Claims To Test

Primary claim:

```text
The cache-friendly condition reduces paid uncached input while preserving task success.
```

Secondary claim:

```text
The cache-friendly condition reduces time to first token or total latency.
```

Do not claim:

```text
total input tokens went down
```

unless that also happens. This project is about reducing paid uncached input, not removing useful context.

## Experiment Directory

Create one directory per experiment:

```bash
cargo run --quiet -- init-experiment --dir runs/2026-05-09-claude-mimo-cache
```

This creates:

```text
runs/2026-05-09-claude-mimo-cache/
  manifest.json
  baseline.jsonl
  cache-friendly.jsonl
  notes.md
  traces/
  requests/
  layers/
  tools/
```

Fill `manifest.json` before running tasks.

## Fixed Variables

Keep these fixed across baseline and cache-friendly runs:

- agent: `claude_code`
- model: `mimo-v2.5-pro`
- repository snapshot / commit
- task prompt
- validation command
- provider or router path
- temperature / generation settings when observable

If any fixed variable changes, record it in `notes.md`.

## Conditions

Baseline:

```text
ordinary setup
```

Cache-friendly:

```text
same model
same provider / route
do not change MCP or hooks mid-session
keep tool schema stable
keep session route stable
structure stable components first and dynamic components later
use cache-aware compact / reactivation when needed
```

## Warm-State Pairing

Do not compare a warm prefix against a cold prefix. For every condition and every task prompt shape:

```text
1. run warm-up calls;
2. verify cache_read or cached_input_tokens is non-zero and stable;
3. exclude warm-up calls from the main result;
4. only then record measured paired A/B calls.
```

If the experiment changes prompt structure, model settings, tools, MCP servers, hooks, repo state, or dynamic system prompt sections, treat the next call as a possible cold start unless the trace proves otherwise.

## Task Suite

Use at least five task families:

| Family | Example | Validation |
| --- | --- | --- |
| docs edit | Add an evaluation section | Markdown diff review |
| small bug fix | Fix one failing unit test | Project test command |
| single-file feature | Add one CLI flag | Unit tests |
| multi-file feature | Add schema + docs + test | Unit tests + review |
| long iterative task | 4-6 turns improving one feature | Same validation each turn |

For each task, run at least:

```text
baseline: 3 runs
cache-friendly: 3 runs
```

For the standardized paired ablation flow, see:

```text
docs/paired-ablation-runbook.md
```

For the first small task-suite dataset, see:

```text
docs/task-suites/claude-cache-ablation-v1.md
```

For the running interpretation of observed phenomena, see:

```text
docs/phenomena-analysis.md
```

## Per-Run Logging

Every model call should append one JSON object to either `baseline.jsonl` or `cache-friendly.jsonl`.

Required fields:

```json
{
  "task_id": "docs-cache-hit-section",
  "run_id": "2026-05-09-cache-friendly-01",
  "condition": "cache_friendly",
  "turn_index": 1,
  "agent": "claude_code",
  "model": "mimo-v2.5-pro",
  "transport": "anthropic_messages",
  "input_tokens": 82000,
  "cached_input_tokens": 76000,
  "output_tokens": 3000,
  "ttft_ms": 1200,
  "total_latency_ms": 24000,
  "tool_calls": 5,
  "validation_command": "cargo test --locked",
  "validation_passed": true,
  "task_success": true,
  "trace_path": "traces/cache-friendly-01.json",
  "request_path": "requests/cache-friendly-01.request.json",
  "layers_path": "layers/cache-friendly-01.layers.json",
  "tools_path": "tools/cache-friendly-01.tools.json"
}
```

If `mimo-v2.5-pro` or the router does not expose cached token accounting, set:

```json
"cached_input_tokens": 0,
"cache_accounting_observable": false
```

and do not claim cost savings from cache hit. You may still report prefix stability and latency as exploratory evidence.

## Trace Artifacts

Save these when available:

```text
traces/{run_id}.json
requests/{run_id}.request.json
layers/{run_id}.layers.json
tools/{run_id}.tools.json
```

Use the CLI checks:

```bash
cargo run --quiet -- breakpoints --input runs/.../requests/{run_id}.request.json
cargo run --quiet -- fingerprint --input runs/.../layers/{run_id}.layers.json --previous runs/.../layers/{previous_run_id}.layers.json
cargo run --quiet -- tool-schema --input runs/.../tools/{run_id}.tools.json --previous runs/.../tools/{previous_run_id}.tools.json
```

## Breakpoint Verification

For Claude Code traces, inspect:

- explicit `cache_control` markers;
- number of breakpoints;
- whether stable anchors appear near `system` / tool prefix regions;
- whether latest user message acts as a moving cursor;
- whether breakpoint gaps exceed the 20-block heuristic.

Run:

```bash
cargo run --quiet -- breakpoints --input request.json
```

## Prefix Stability Verification

Expected healthy pattern:

```text
system_policy: stable
tool_schema: stable
repo_rules: stable
project_memory: stable or slowly changing
active_state: changed
latest_tool_results: changed
user_request: changed
```

Run:

```bash
cargo run --quiet -- fingerprint --input current-layers.json --previous previous-layers.json
```

## Tool Schema Verification

Expected healthy pattern:

```text
tool schema hash stable
tool definition order stable
tool execution order free to vary
```

Run:

```bash
cargo run --quiet -- tool-schema --input current-tools.json --previous previous-tools.json
```

## Final Comparison

After collecting runs:

```bash
cargo run --quiet -- eval \
  --baseline runs/2026-05-09-claude-mimo-cache/baseline.jsonl \
  --candidate runs/2026-05-09-claude-mimo-cache/cache-friendly.jsonl
```

Always also run the per-task token report:

```bash
cargo run --quiet -- task-report \
  --baseline runs/2026-05-09-claude-mimo-cache/baseline.jsonl \
  --candidate runs/2026-05-09-claude-mimo-cache/cache-friendly.jsonl
```

If the route exposes current prices, also run:

```bash
cargo run --quiet -- eval \
  --baseline runs/2026-05-09-claude-mimo-cache/baseline.jsonl \
  --candidate runs/2026-05-09-claude-mimo-cache/cache-friendly.jsonl \
  --uncached-input-per-mtok <USD> \
  --cached-input-per-mtok <USD> \
  --output-per-mtok <USD>
```

Report:

- cache hit rate;
- cached input tokens per task;
- uncached input tokens;
- output tokens;
- median TTFT;
- median total latency;
- task success;
- validation pass rate;
- any regressions.

## Success Criteria

The experiment supports the claim only if:

```text
cached_input_tokens increase
uncached_input_tokens decrease
estimated or observed cost decreases
TTFT / latency does not regress
task_success does not regress
validation_passed does not regress
```

If cached token accounting is unavailable:

```text
Report "cache accounting not observable" and do not claim token-cost savings.
```
