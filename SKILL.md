---
name: "make-agents-cheaper"
description: "Use when a user wants Codex or coding agents to cost less by improving prompt-cache friendliness, cache hit rate, Responses/WebSocket continuity, and stable-prefix habits. Phase 1 audits Codex config; later phases can cover other agents."
metadata:
  short-description: "Improve coding-agent prompt cache hit rate"
---

# Make Agents Cheaper

Use this skill to inspect and improve prompt-cache friendliness for coding-agent workflows. Phase 1 is Codex-focused: lower repeated-input cost and latency through stable official prompt caching, not by changing task semantics.

This is outside-the-model cache work, not model training. The skill works at the agent harness layer: configuration, request envelope, transport, session route, and stable prompt prefix.

## Safety Rules

- Default to report-only. Do not modify `~/.codex/config.toml` unless the user explicitly asks.
- Never print API key values. It is okay to report whether an expected env var is set.
- Do not promise universal savings. Say that savings depend on provider pricing, cache policy, stable prefixes, and session routing.
- Preserve Codex semantics. Do not recommend request rewriting that changes `store`, `stream`, `instructions`, conversation continuity, or tool schemas merely to make cache metrics look better.
- Prefer stable configuration and stable sessions over aggressive prompt compression.
- Frame this as cache-aware agent harness engineering. Do not describe it as model training, fine-tuning, or output replay.
- Treat reusable skills as runbooks. The evidence for claims comes from `make-agents-cheaper` audit/eval logs, not from the skill layer itself.

## Default Workflow

1. Explain that the first pass is read-only and checks whether the Codex config is cache friendly.
2. Run:

```bash
cargo run --quiet
```

3. Summarize:
   - configured provider
   - active model
   - base URL
   - Responses API status
   - WebSocket status
   - env key presence without printing the value
   - warnings that could reduce cache hits
4. If the user wants a template, run one of:

```bash
cargo run --quiet -- --print-ws-config
cargo run --quiet -- --print-http-config
```

5. For trace/evaluation work, use the first executable checks:

```bash
cargo run --quiet -- fingerprint --input layers.json
cargo run --quiet -- tool-schema --input tools.json
cargo run --quiet -- breakpoints --input request.json
cargo run --quiet -- eval --baseline baseline.jsonl --candidate cache-friendly.jsonl
cargo run --quiet -- task-report --baseline baseline.jsonl --candidate cache-friendly.jsonl
cargo run --quiet -- analysis-report --baseline baseline.jsonl --candidate cache-friendly.jsonl
cargo run --quiet -- init-experiment --dir runs/experiment-name
cargo run --quiet -- compact-template
```

6. If the user wants you to edit config, first show the exact intended config and ask for confirmation. Back up the existing config before writing.

## Standardized A/B Workflow

Use this when the user wants a benchmark rather than a config audit.

1. Create a reproducible experiment directory:

```bash
cargo run --quiet -- init-experiment --dir runs/<experiment>
```

2. Generate the paired run plan from the task manifest:

```bash
cargo run --quiet -- pilot-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --task <task-id> \
  --experiment-dir runs/<experiment> \
  --slice dynamic-drift \
  --repeats 1
```

For the full suite, print all task/slice plans:

```bash
cargo run --quiet -- matrix-plan \
  --manifest docs/task-suites/real-coding-ablation-v2.manifest.json \
  --experiment-dir runs/<experiment> \
  --repeats 3
```

3. Run baseline warm-up, candidate warm-up, baseline measured, and candidate measured calls. Keep provider, model, route, tools, MCP servers, hooks, working directory, task prompt, and validation command fixed.

4. Normalize each measured direct Claude JSON run:

```bash
cargo run --quiet -- claude-json-import \
  --input runs/<experiment>/raw/claude-json/<run-id>.json \
  --run-id <run-id> \
  --task-id <task-id> \
  --condition baseline \
  --slice dynamic-drift \
  --repeat-id 1 \
  --phase measured \
  --output runs/<experiment>/baseline.jsonl \
  --validation-path runs/<experiment>/validation/<run-id>.txt \
  --validation-passed true \
  --task-success true
```

Use `--condition cache-friendly` and `--output runs/<experiment>/cache-friendly.jsonl` for candidate runs.

5. Compare and write paper-facing tables:

```bash
cargo run --quiet -- eval --baseline runs/<experiment>/baseline.jsonl --candidate runs/<experiment>/cache-friendly.jsonl
cargo run --quiet -- task-report --baseline runs/<experiment>/baseline.jsonl --candidate runs/<experiment>/cache-friendly.jsonl
cargo run --quiet -- analysis-report \
  --baseline runs/<experiment>/baseline.jsonl \
  --candidate runs/<experiment>/cache-friendly.jsonl \
  --output runs/<experiment>/analysis-report.md
```

Only call the candidate cheaper when all-runs uncached input is lower, task success does not regress, and cache accounting is observable for the relevant records. Successful-only rows are diagnostic and must not replace all-runs accounting.

## Role Separation

- Codex: development assistant and operator of this skill.
- Claude Code: current studied harness when using the Claude adapter.
- MiMo: current backend route/model family, such as `mimo-v2.5-pro`.
- `make-agents-cheaper`: Rust audit/eval instrumentation and paper-facing evidence.
- `make-agents-cheaper-skill` / `cheaper-skill-for-claude`: reusable runbooks and usability artifacts, not primary cache-hit evidence.

## Recommended Policy

- Use a single stable provider during one long coding session.
- Keep model, reasoning effort, and transport stable for the duration of a task.
- Prefer `wire_api = "responses"` for Codex workflows.
- Prefer WebSocket mode for long-running interactive coding when available.
- Keep repeated workspace instructions stable and put changing task details later.
- Treat cache hit rate as an engineering outcome, not a trick.
- Package reusable advice as agent skills once the policy is stable, so different agents can reuse the same harness-level cache discipline.
- Start with detection in existing agents; move policies into `cheapcode` only when the native framework controls prompt assembly and tool registry.

## User-Facing Explanation

Say this in plain language:

> This makes agents cheaper only when repeated prompt prefixes stay identical enough for the provider cache to hit. Phase 1 focuses on Codex. The skill helps keep that path stable; it does not fake cache, hide context, or change the model's answer.

Also say:

> Model-side projects make the model itself cheaper. This project makes the agent harness more cache-friendly, so repeated context is more likely to be billed or processed as cached input.
