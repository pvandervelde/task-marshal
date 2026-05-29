# ADR-0007: Error Handling Strategy

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

task-marshal must handle a variety of failure modes across four layers:

1. **Config failures** — config file not found, invalid TOML, missing required fields
2. **Source failures** — binary not in PATH, network unavailable, database locked, task not found
3. **Selection failures** — no eligible task after filtering
4. **Completion failures** — source update failed mid-write

The requirements specify distinct behaviour for each:

- Source unavailability → warning + skip (not exit 2)
- No task found → exit 1 (not exit 2)
- Operational errors → exit 2

A unified error handling strategy is needed.

---

## Decision

All fallible operations return `Result<T, E>` with typed error enums derived using `thiserror`. No `panic`, `unwrap()`, or `expect()` is used in non-test code outside of `main()`.

**Error types:**

| Type | Produced by | Exit code |
|---|---|---|
| `ConfigError` | `ConfigLoader` | 2 |
| `SourceError::Unavailable` | Source adapters | warning + skip (no exit) |
| `SourceError::NotFound` | Source adapters | 2 |
| `SourceError::Io(...)` | Source adapters | 2 (or skip, depending on command) |
| `SourceError::Parse(...)` | Source adapters | 2 |
| `SelectionError::NoTaskFound` | `TaskSelector` | 1 |
| `CompletionError` | Source adapters (via `done`) | 2 |
| `ParseError::InvalidTaskId` | `TaskIdParser` | 2 |

The **CLI Dispatcher** is the sole component responsible for converting `Result` errors to exit codes and writing error messages to stderr.

**Exit code mapping:**

- `Ok(...)` → exit 0
- `SelectionError::NoTaskFound` → exit 1
- All other errors → exit 2

---

## Rationale

- Typed error enums make all error paths explicit at compile time
- `thiserror` eliminates boilerplate while keeping errors structured
- Distinguishing exit 1 (no task) from exit 2 (error) allows agent skill files to handle the empty-queue case programmatically
- Centralising exit code logic in the CLI Dispatcher keeps error semantics out of business logic
- `SourceError::Unavailable` is a soft failure — it must not propagate to exit 2, as unavailability is a normal operational condition

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| `anyhow` throughout | Type-erased errors lose the ability to match on `SourceError::Unavailable` for skip behaviour; acceptable only in `main()` as a last resort |
| String errors | No structure; cannot programmatically distinguish error types |
| Exceptions / panics | Not idiomatic in Rust; panics produce unfriendly output for a CLI; cannot be caught cleanly |
| Single error enum for everything | Conflates distinct error domains; makes the skip logic for `SourceError::Unavailable` harder to express cleanly |

---

## Consequences

**Positive:**

- All error paths are explicit and compiler-checked
- Skip logic for `SourceError::Unavailable` is clear and testable
- Exit 1 vs exit 2 distinction enables programmatic use by agents
- Error messages are consistently written to stderr by one place (CLI Dispatcher)

**Negative / Tradeoffs:**

- More boilerplate than `anyhow` (mitigated by `thiserror`)
- Converting between error types across layer boundaries requires `From` implementations

---

## References

- [constraints.md](../spec/constraints.md) — Error Handling and Exit Codes sections
- [assertions.md](../spec/assertions.md) — A2, A11, A12, A18, A19, A20
- [edge-cases.md](../spec/edge-cases.md) — EC-01 through EC-05
- User requirements §11 — Behaviour Requirements (graceful source failure, atomic done, non-zero exit)
