# Paired Cache-Hit Ablation Runbook

This runbook standardizes the ablation that compares an ordinary agent harness against a cache-friendly harness.

Target mechanism:

```text
dynamic harness state in the early prompt prefix hurts cache reuse;
moving dynamic state later preserves the reusable prefix.
```

## Conditions

Baseline:

```bash
claude --print --output-format json --model mimo-v2.5-pro --max-budget-usd 0.2 "$PROMPT"
```

Cache-friendly:

```bash
claude --print --output-format json --model mimo-v2.5-pro --max-budget-usd 0.2 \
  --exclude-dynamic-system-prompt-sections "$PROMPT"
```

This does not patch Claude Code. The flag asks Claude Code's internal prompt assembler to move per-machine dynamic sections such as cwd, environment info, memory paths, and git status out of the early system prompt and into the first user message. The dynamic context is still sent to the model; it just appears later.

For cleaner standalone benchmark runs, add:

```text
--no-session-persistence
```

This reduces hidden conversation-history effects in `--print` mode. If the benchmark intentionally studies normal persisted Claude Code behavior, record that choice in `manifest.json`.

## Required Slices

Run at least two slices:

| Slice | Purpose | Expected pattern |
| --- | --- | --- |
| `control-steady` | No deliberate repo-state change | Both conditions can be warm and high-cache |
| `dynamic-drift` | Deliberately change dynamic harness state before each pair | Cache-friendly should keep higher cache read |

The control slice is not optional. It prevents the claim from becoming "the flag is always cheaper."

## Warm-Up Rule

Do not compare a warm prefix against a cold prefix.

For every condition and every prompt shape:

```text
1. run a warm-up call;
2. run a second warm-up or confirmation call if cache_read is still low;
3. exclude warm-up calls from the main result;
4. start measured A/B only after both conditions show warm cache behavior.
```

If the prompt text, model, tool set, MCP config, hook config, cwd, repo state, or prompt-assembly mode changes, treat the next call as a possible cold start.

## Dynamic Drift Rule

Use a small, reversible repo-state perturbation. Example:

```bash
printf 'dynamic drift probe 1\n' > .cache-drift-probe-1
git status --short > runs/.../notes/drift-state-1.txt
```

After the experiment, remove the probe files:

```bash
rm -f .cache-drift-probe-*
```

The drift artifact should never be committed.

## Main JSONL Fields

Each measured call must record:

```json
{
  "task_id": "dynamic-drift-docs-summary",
  "run_id": "drift-docs-baseline-01",
  "condition": "baseline",
  "agent": "claude_code",
  "model": "mimo-v2.5-pro",
  "transport": "claude_print_json",
  "input_tokens": 26865,
  "cached_input_tokens": 4096,
  "cache_creation_input_tokens": 0,
  "output_tokens": 23,
  "total_latency_ms": 7843,
  "model_latency_ms": 7751,
  "actual_cost_usd": 0.116468,
  "validation_command": "trimmed exact reply",
  "validation_passed": true,
  "task_success": true,
  "cache_accounting_observable": true,
  "trace_path": "traces/real/drift-docs-baseline-01.json"
}
```

For Claude Code JSON output, use:

```text
modelUsage.<model>.inputTokens
modelUsage.<model>.cacheReadInputTokens
modelUsage.<model>.cacheCreationInputTokens
modelUsage.<model>.outputTokens
total_cost_usd
duration_ms
duration_api_ms
```

Also record `num_turns` when available. If a simple exact-reply task unexpectedly uses multiple turns, treat the run as a possible quality or harness anomaly and document it.

For `make-agents-cheaper eval`, set:

```text
input_tokens = inputTokens + cacheReadInputTokens + cacheCreationInputTokens
cached_input_tokens = cacheReadInputTokens
```

This makes `cache_hit_rate = cached_input_tokens / input_tokens`.

## Reporting

Report at least:

- overall result;
- control-only result;
- dynamic-drift-only result;
- per-task dynamic-drift result;
- per-task token usage from `task-report`;
- excluded warm-up calls;
- whether any quality validation failed.

Use cautious wording:

```text
The cache-friendly structure helps when dynamic harness state would otherwise disturb the early prompt prefix.
```

Do not write:

```text
The cache-friendly structure is always cheaper.
```

## Output Contract

For cheap validation tasks, use a strict output contract:

```text
Return only the exact string: <expected>
Do not explain.
```

Weak exact-reply prompts can still produce verbose answers. Keep those failures; do not silently replace them in the main result.

## Per-Task Token Report

Every benchmark run should include a per-task token table:

```bash
cargo run --quiet -- task-report \
  --baseline runs/.../baseline.jsonl \
  --candidate runs/.../cache-friendly.jsonl
```

The table must make these fields visible per task:

```text
input tokens
cached input tokens
uncached input tokens
output tokens
task success
```
