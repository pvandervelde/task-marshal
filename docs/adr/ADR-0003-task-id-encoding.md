# ADR-0003: Task ID Encoding

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

The requirements (§7) state that every task returned by task-marshal must carry a source-encoded ID that is:

- Opaque to the consumer (the consumer must not parse or interpret it)
- Sufficient for task-marshal to route completion (`done`) back to the correct source without any additional context

This means the ID itself must encode which source owns the task, so `task-marshal done <id>` can determine which source adapter to invoke.

---

## Decision

Task IDs use the format **`<SourceKey>:<NativeId>`**, where:

- `SourceKey` is the configured source name (`local`, `beads`, `gh`)
- `NativeId` is the identifier used by the source system (e.g., `TASK-042`, `bd-a1b2`, `187`)
- The colon (`:`) is the delimiter

Examples:

- `local:TASK-042`
- `beads:bd-a1b2`
- `gh:187`

The `TaskIdParser` splits on the first colon only, allowing `NativeId` values that themselves contain colons (e.g., BEADS hierarchical IDs like `bd-a1b2.3`).

---

## Rationale

- Simple, human-readable format that is also unambiguous for parsing
- Self-routing: the source can be determined without any external lookup
- Minimal: no encoding overhead, no UUID indirection, no external registry
- Split-on-first-colon handles BEADS hierarchical IDs that may contain dots but not colons

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| UUID with a source lookup table | Requires task-marshal to maintain a running registry mapping UUIDs to (source, native_id); not feasible for a stateless CLI |
| JSON envelope `{"source": "local", "id": "TASK-042"}` | Unnecessarily verbose; harder to type at the command line; not human-friendly |
| Numeric index into a cached source list | Would require a persistent cache; fragile if cache is stale |
| Opaque base64 encoded JSON | Unreadable to humans; provides no benefit over the simple format |

---

## Consequences

**Positive:**

- Stateless routing — `done` and `show` work without any session state
- Human-readable — developers can read and type task IDs
- Consistent across all sources

**Negative / Tradeoffs:**

- `SourceKey` is part of the public-facing ID; renaming a source key (e.g., `gh` → `github`) would break all existing IDs
- The format requires validation on input (`done <id>`) to reject malformed IDs

---

## References

- [vocabulary.md](../spec/vocabulary.md) — `TaskId`, `NativeId`, `SourceKey` definitions
- [assertions.md](../spec/assertions.md) — A19, A20: invalid ID handling
- User requirements §7 — Task IDs
