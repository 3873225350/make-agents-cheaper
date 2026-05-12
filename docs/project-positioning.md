# Project Positioning

`make-agents-cheaper` is a Token Saver direction for coding agents: prompt-cache-aware context layout, trace-level token accounting, and paired A/B evaluation to reduce paid uncached input without reducing validated task success.

## Origin

Many researchers with training resources work on the model side: training, architecture, distillation, and serving. Many others work on the inference side: cache, KV cache, batching, kernels, and lower-latency serving. These are still mostly model-layer or serving-layer directions.

This project did not start from "let's make agents cheaper." It started from trying to understand why DeepSeek v4-style systems make long-context tokens cheaper, what cache hit really means, and how this differs from ordinary model use.

At the same time, while building `claude-trace` and `codex-trace` style tooling for agent visualization and explainability, we saw the concrete request payload. A coding-agent request is not just the user's latest message. It is a harness-assembled bundle of stable instructions, tool schemas, repo rules, session state, transport choices, and dynamic task data.

That made the key question practical:

```text
In the agent era, what can an individual builder do outside the model?
```

The answer explored here is small but useful: make the repeated prefix stable and measurable, so prompt cache hit rate can improve without removing context.

## Outside The Model

Modern systems such as DeepSeek v4-style long-context routes point to an important industry direction: tokens are not only a quality bottleneck, but also a cost and scalability bottleneck. Model providers can attack this from the model and serving side with long-context architecture, sparse attention, compression, KV/cache systems, batching, and cache hit/miss pricing.

`make-agents-cheaper` attacks the same cost problem from the agent side. It does not change model weights or claim a universal reduction in total tokens. Instead, it studies the prompt assembled by the coding-agent harness and asks:

```text
Which repeated parts can stay stable enough to become cheap cached input?
Which dynamic parts should move later so they do not break the reusable prefix?
```

The practical rule is:

```text
Structure your prompt so stable components come first
and dynamic components come later.
```

For this project, "prompt" means the full agent harness payload, not just the user's natural-language instruction.

This is why it can later be packaged as a reusable skill. The same cache-hit discipline can be applied by different agents even if their model providers and UI surfaces differ.
