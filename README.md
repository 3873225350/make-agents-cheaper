# Make Agents Cheaper

A Rust CLI for improving prompt cache hit rate in coding-agent workflows.

Phase 1 is Codex-first. The longer-term goal is to make the same cache-hit discipline useful for Claude Code, Cursor, custom agent runners, and multi-agent routers.

The idea is simple:

> Do not remove context. Make repeated context cacheable.

## Why This Can Be Cheap

Coding agents are expensive in long sessions because every turn can resend a large repeated prefix:

- system and developer instructions
- tool definitions and JSON schemas
- repo rules such as `AGENTS.md`
- stable project context
- previous session and conversation identifiers

Prompt caching can make that repeated prefix cheaper, but only when the provider sees the same beginning of the request again. The cache is strict: similar text is not enough; the prefix has to stay stable enough to match.

`make-agents-cheaper` helps with the parts a user can control:

- **Stable provider:** do not bounce the same task between providers or upstream keys.
- **Stable transport:** prefer one agent path for the task, especially Responses API for Codex.
- **Stable session:** WebSocket mode and session-aware routing make it easier for later turns to land near existing cache.
- **Stable model settings:** model and reasoning effort changes can create different request buckets.
- **Stable static context:** keep repeated rules and tool context stable; avoid injecting changing bridge text before it.

The savings come from the provider charging or processing cached input more cheaply than uncached input. This project does not hide context from the agent, truncate important instructions, or rewrite the model's task. It makes the official cache path easier to hit.

The rough mental model is:

```text
same long prefix + same session route + compatible transport
  -> higher prompt-cache hit probability
  -> less repeated prefill work
  -> lower repeated-input cost and latency
```

In other words:

```text
It reduces paid uncached input, not necessarily total input.
```

## Feature 1: Codex Cache-Hit Audit

XAI Router improves cache hits in the routing layer. This repository focuses on the missing client-side step:

> Before blaming the router or model, verify that your local Codex config is actually cache-hit friendly.

The bundled Rust CLI is read-only by default. It inspects a Codex `config.toml` and reports:

- whether the configured provider points at `https://api.xairouter.com`
- whether `wire_api = "responses"` is set
- whether WebSocket mode is enabled when you expect long sessions
- whether `env_key` is configured and present in the current shell
- whether model and reasoning settings are stable enough for repeat sessions
- whether the config looks likely to drift between providers or transport modes

It also prints copy-ready HTTP and WebSocket configuration templates.

## Quick Start

Ask Codex:

```text
Use $make-agents-cheaper to inspect my Codex config and tell me whether it is prompt-cache friendly.
```

Or run the CLI directly:

```bash
cargo run --quiet
```

Print the recommended WebSocket template:

```bash
cargo run --quiet -- --print-ws-config
```

Print the simpler HTTP template:

```bash
cargo run --quiet -- --print-http-config
```

Inspect a custom config path:

```bash
cargo run --quiet -- --config /path/to/config.toml
```

## Recommended Codex WebSocket Config

Use this when you want stronger long-session continuity:

```toml
model_provider = "xai"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"
approval_policy = "never"
sandbox_mode = "danger-full-access"
suppress_unstable_features_warning = true

[model_providers.xai]
name = "OpenAI"
base_url = "https://api.xairouter.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "XAI_API_KEY"
supports_websockets = true

[features]
responses_websockets_v2 = true
```

## Recommended Codex HTTP Config

Use this when you prefer a simpler, broadly compatible setup:

```toml
model_provider = "xai"
model = "gpt-5.4"
model_reasoning_effort = "xhigh"
plan_mode_reasoning_effort = "xhigh"
model_reasoning_summary = "none"
model_verbosity = "medium"
approval_policy = "never"
sandbox_mode = "danger-full-access"

[model_providers.xai]
name = "OpenAI"
base_url = "https://api.xairouter.com"
wire_api = "responses"
requires_openai_auth = false
env_key = "XAI_API_KEY"
```

## Cheapness Checklist

- Keep static instructions, tool schemas, and repo rules stable.
- Avoid switching providers, models, or transport modes mid-task.
- Prefer Responses API for Codex-style workflows.
- Use WebSocket mode for long interactive sessions when available.
- Keep session and conversation continuity intact.
- Put dynamic task details after stable context when you control prompt layout.
- Do not chase artificial cache metrics by rewriting request semantics.

## What It Does Not Do

- It does not make every token cheap.
- It does not cache model outputs or replay old answers.
- It does not share cache across organizations.
- It does not mutate `~/.codex/config.toml` unless a future command explicitly implements that and you ask for it.
- It does not print API keys.
- It does not claim support for every agent yet; Codex is the first supported target.

## Roadmap

- **Phase 1:** Codex config audit and XAI Router-friendly templates.
- **Phase 2:** benchmark helpers for cache hit rate, cached tokens, uncached paid input, latency, and task success.
- **Phase 3:** Claude Code and Cursor cache-friendliness checks where reliable local signals exist.
- **Phase 4:** router and multi-agent workflow diagnostics.

## Build

```bash
cargo build --release
```

The binary will be available at:

```bash
target/release/make-agents-cheaper
```

Run validation:

```bash
cargo test
```

## Install As A Skill

Ask Codex:

```text
Install the make-agents-cheaper skill from https://github.com/3873225350/make-agents-cheaper
```

Or clone/copy this folder into your Codex skills directory as `make-agents-cheaper`.

## Privacy And Safety

Report mode does not write files. It prints only configuration health and hides environment variable values. It never prints API keys.

If you share reports publicly, review local paths and provider names first.

