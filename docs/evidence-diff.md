# Session Tool Evidence Diff

`evidence-diff` reconstructs code-change evidence from an agent/session record. It does not read the current worktree and it does not replace `git diff`.

The command answers:

```text
What did a tool call claim to change at that time?
```

It does not answer:

```text
What does the current filesystem differ from git baseline?
```

## Command

```bash
cargo run --quiet -- evidence-diff \
  --input runs/<experiment>/raw/session.jsonl \
  --output runs/<experiment>/code-changes.json
```

Input may be JSONL or JSON. The parser scans tool-like events such as `tool_call`, `tool_result`, `function_call`, and `function_call_output`.

## Detection

The parser searches likely payload fields:

- `input`
- `arguments`
- `args`
- `output`
- `result`
- `message`
- `text`
- `patch`
- `diff`

It recognizes:

- `apply_patch` blocks delimited by `*** Begin Patch` and `*** End Patch`;
- unified diffs with `diff --git`, or with `---`, `+++`, and `@@` hunk lines.

## Output Shape

The output is structured JSON:

```json
{
  "evidence_scope": "session_tool_record",
  "source": "session.jsonl",
  "limitations": [
    "This evidence diff is reconstructed from session/tool records.",
    "It answers what a tool call claimed to change at that time.",
    "It is not the current filesystem diff against git baseline."
  ],
  "code_changes": [
    {
      "event_index": 0,
      "timestamp": "2026-05-12T00:00:00Z",
      "tool_name": "apply_patch",
      "call_id": "call-1",
      "title": "apply_patch evidence diff",
      "summary": "apply_patch: 1 files +1 -1",
      "diff": {
        "kind": "apply_patch",
        "files": ["src/lib.rs"],
        "additions": 1,
        "deletions": 1,
        "summary": "apply_patch: 1 files +1 -1",
        "preview": "...",
        "before": "...",
        "after": "..."
      }
    }
  ]
}
```

## Why This Matters

For `make-agents-cheaper`, this is a missing evidence layer between token accounting and current git state. In direct Claude JSON runs, request/layer artifacts may be unavailable, but session/tool records can still preserve what the agent claimed it patched. This helps explain quality regressions, extra turns, and suspicious cost changes without relying on the current dirty worktree.
