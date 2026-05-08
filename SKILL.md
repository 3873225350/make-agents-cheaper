---
name: "make-agents-cheaper"
description: "Use when a user wants Codex or coding agents to cost less by improving prompt-cache friendliness, cache hit rate, Responses/WebSocket continuity, and stable-prefix habits. Phase 1 audits Codex config; later phases can cover other agents."
metadata:
  short-description: "Improve coding-agent prompt cache hit rate"
---

# Make Agents Cheaper

Use this skill to inspect and improve prompt-cache friendliness for coding-agent workflows. Phase 1 is Codex-focused: lower repeated-input cost and latency through stable official prompt caching, not by changing task semantics.

## Safety Rules

- Default to report-only. Do not modify `~/.codex/config.toml` unless the user explicitly asks.
- Never print API key values. It is okay to report whether an expected env var is set.
- Do not promise universal savings. Say that savings depend on provider pricing, cache policy, stable prefixes, and session routing.
- Preserve Codex semantics. Do not recommend request rewriting that changes `store`, `stream`, `instructions`, conversation continuity, or tool schemas merely to make cache metrics look better.
- Prefer stable configuration and stable sessions over aggressive prompt compression.

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

5. If the user wants you to edit config, first show the exact intended config and ask for confirmation. Back up the existing config before writing.

## Recommended Policy

- Use a single stable provider during one long coding session.
- Keep model, reasoning effort, and transport stable for the duration of a task.
- Prefer `wire_api = "responses"` for Codex workflows.
- Prefer WebSocket mode for long-running interactive coding when available.
- Keep repeated workspace instructions stable and put changing task details later.
- Treat cache hit rate as an engineering outcome, not a trick.

## User-Facing Explanation

Say this in plain language:

> This makes agents cheaper only when repeated prompt prefixes stay identical enough for the provider cache to hit. Phase 1 focuses on Codex. The skill helps keep that path stable; it does not fake cache, hide context, or change the model's answer.
