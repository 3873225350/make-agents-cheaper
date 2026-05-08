# Cache-Friendly Codex Notes

Prompt caching rewards identical prefixes. In Codex, the repeated prefix often includes system/developer instructions, tool schemas, repo rules, and stable session context.

## Why This Can Reduce Cost

Long Codex turns are not expensive only because of the new user message. A large part of the input can be repeated boilerplate that Codex must send again so the model has the same operating contract:

- developer and system policy
- tool schemas
- project instructions
- repository state and continuity context

Provider-side prompt caching reduces the cost of processing that repeated prefix when the beginning of the next request matches a previous request. The cache is not semantic. It does not say "these prompts are close enough." It is based on stable request prefixes and compatible routing.

That gives this project its first narrow job: audit and explain the Codex settings that make cache hits more likely. The repository name is broader because the same principle should later apply to other coding agents, but Codex is phase 1.

Practical rules:

- Keep the configured provider stable during a task.
- Keep `model` and reasoning effort stable during a task.
- Use Responses API when possible.
- Use WebSocket mode for long sessions when the provider supports it.
- Avoid compatibility layers that inject changing bridge prompts before stable content.
- Avoid provider rotation that sends the same session to different upstream accounts or keys.
- Prefer session-aware routing using `session_id`, `conversation_id`, or stable cache keys when the router supports them.

What this does not do:

- It does not compress away important project context.
- It does not rewrite model instructions.
- It does not claim that every request gets a discount.
- It does not expose secrets.
