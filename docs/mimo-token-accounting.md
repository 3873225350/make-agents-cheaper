# MiMo Token Accounting Note

This project currently evaluates Claude Code runs routed to:

```text
mimo-v2.5-pro
```

The primary research signal is token usage, not price.

## Official MiMo API Background

Sources checked on 2026-05-09:

```text
https://platform.xiaomimimo.com/docs/zh-CN/
https://platform.xiaomimimo.com/docs/zh-CN/api/chat/openai-api
```

The official docs are rendered as a front-end app, so the repository records only the stable details that can be confirmed from the loaded document assets.

MiMo is exposed as an API platform with both pay-as-you-go API usage and Token Plan usage. The docs show two compatible protocol families:

| Usage mode | OpenAI-compatible base URL | Anthropic-compatible base URL | API key shape |
| --- | --- | --- | --- |
| Pay-as-you-go API | `https://api.xiaomimimo.com/v1` | `https://api.xiaomimimo.com/anthropic` | `sk-xxxxx` |
| Token Plan | `https://token-plan-cn.xiaomimimo.com/v1` | `https://token-plan-cn.xiaomimimo.com/anthropic` | `tp-xxxxx` |

The OpenAI-compatible chat examples use:

```text
POST https://api.xiaomimimo.com/v1/chat/completions
header: api-key: $MIMO_API_KEY
header: Content-Type: application/json
```

The request shape follows the familiar chat-completions form:

```text
model
messages
max_completion_tokens
temperature
top_p
stream
stop
frequency_penalty
presence_penalty
tools
tool_choice
thinking
```

The response examples expose token usage in OpenAI-compatible fields:

```text
usage.prompt_tokens
usage.completion_tokens
usage.total_tokens
usage.completion_tokens_details.reasoning_tokens
usage.prompt_tokens_details.cached_tokens
```

`usage.prompt_tokens_details` may be `null` when no detailed prompt-token accounting is returned. When `cached_tokens` is present, it is the direct MiMo-side cache-hit token signal we care about.

For streaming responses, the examples show `usage: null` on intermediate chunks and a final chunk containing `usage`. Therefore direct MiMo streaming experiments must record token usage from the terminal usage chunk, not from early stream chunks.

The official background pages also position `mimo-v2.5-pro` / `mimo-v2-pro` as long-context agent-oriented models. The release material describes `mimo-v2.5-pro` as an agent-workload model with 1T total parameters, 42B activated parameters, hybrid attention, and 1M context. For our project, the important point is not the model architecture itself; it is that long coding-agent requests can contain very large repeated prefixes, and MiMo exposes cached-token accounting for judging whether those prefixes are being reused.

## Why We Record Tokens First

Price tables can change. Token accounting is the stable experimental observation.

For every measured task, record:

```text
task_id
input_tokens
cached_input_tokens
cache_creation_input_tokens
uncached_input_tokens = input_tokens - cached_input_tokens
output_tokens
cache_hit_rate = cached_input_tokens / input_tokens
```

When Claude Code JSON exposes `modelUsage.mimo-v2.5-pro`, extract:

```text
inputTokens
cacheReadInputTokens
cacheCreationInputTokens
outputTokens
costUSD
```

For this repository's JSONL schema:

```text
input_tokens = inputTokens + cacheReadInputTokens + cacheCreationInputTokens
cached_input_tokens = cacheReadInputTokens
cache_creation_input_tokens = cacheCreationInputTokens
output_tokens = outputTokens
```

When calling MiMo's OpenAI-compatible API directly, use the direct `usage` fields instead:

```text
input_tokens = usage.prompt_tokens
cached_input_tokens = usage.prompt_tokens_details.cached_tokens
cache_creation_input_tokens = 0 unless the route exposes a separate write metric
output_tokens = usage.completion_tokens
reasoning_tokens = usage.completion_tokens_details.reasoning_tokens
```

If `usage.prompt_tokens_details` is `null`, set `cached_input_tokens = 0` and mark whether cache accounting was observable for that record. Do not infer cache hits from low cost alone.

The key comparison is:

```text
baseline uncached input tokens
vs
cache-friendly uncached input tokens
```

not simply total cost.

## MiMo Pricing Background

Source page:

```text
https://platform.xiaomimimo.com/docs/zh-CN/quick-start/model-hyperparameters
```

The following pricing table is treated as technical background supplied for interpretation. Verify current pricing on the official page before making any cost claim.

Domestic pricing, CNY per million tokens:

| Model | <=256K cached input | <=256K uncached input | <=256K output | 256K-1M cached input | 256K-1M uncached input | 256K-1M output |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `mimo-v2.5-pro` / `mimo-v2-pro` | 1.40 | 7.00 | 21.00 | 2.80 | 14.00 | 42.00 |
| `mimo-v2.5` | 0.56 | 2.80 | 14.00 | 1.12 | 5.60 | 28.00 |
| `mimo-v2-omni` | 0.56 | 2.80 | 14.00 | n/a | n/a | n/a |
| `mimo-v2-flash` | 0.07 | 0.70 | 2.10 | n/a | n/a | n/a |
| `mimo-v2.5-tts` / `mimo-v2.5-tts-voiceclone` / `mimo-v2.5-tts-voicedesign` / `mimo-v2-tts` | limited-time free | limited-time free | limited-time free | n/a | n/a | n/a |

This table explains why cache-hit tokens matter, but the benchmark should still report token usage first.

## Reporting Rule

Every task-level result table should include:

| Field | Meaning |
| --- | --- |
| `input_tokens` | Total observed input tokens, including cached and uncached portions |
| `cached_input_tokens` | Tokens read from prompt cache |
| `cache_creation_input_tokens` | Tokens used to create cache, when observable |
| `uncached_input_tokens` | Non-cache-read input portion |
| `output_tokens` | Generated tokens |
| `cache_hit_rate` | Cached input divided by total input |

Cost fields such as `actual_cost_usd` are secondary. They are useful, but they should not replace token accounting.

## CLI

Use the aggregate report:

```bash
cargo run --quiet -- eval \
  --baseline runs/.../baseline.jsonl \
  --candidate runs/.../cache-friendly.jsonl
```

Use the per-task token report:

```bash
cargo run --quiet -- task-report \
  --baseline runs/.../baseline.jsonl \
  --candidate runs/.../cache-friendly.jsonl
```
