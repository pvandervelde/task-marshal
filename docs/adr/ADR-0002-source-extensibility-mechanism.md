# ADR-0002: Source Extensibility via Trait Objects

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

The requirements state (§6.4): "The architecture must allow additional sources to be added without changing the CLI interface."

task-marshal currently supports three sources: `local`, `beads`, `github`. Future sources (e.g., Linear, Jira, a remote task service) must be addable without modifying the CLI, selection logic, or any existing source.

A decision is needed on the mechanism for source polymorphism.

---

## Decision

Source polymorphism is implemented via a **`TaskSource` trait** with **trait objects** (`Box<dyn TaskSource>`).

The `SourceRegistry` builds an ordered `Vec<Box<dyn TaskSource>>` from config. All business logic (`TaskSelector`, `TaskIdParser`, CLI commands) interacts exclusively with this trait.

---

## Rationale

- Adding a new source requires only: implementing `TaskSource` for a new struct; adding a new `SourceKey` variant; registering it in `SourceRegistry`
- No changes to `TaskSelector`, CLI dispatch, or any existing source adapter
- Dynamic dispatch overhead is immaterial for a CLI tool making at most 3 subprocess calls per invocation
- The `TaskSource` interface is stable and well-defined; it becomes the contract that new sources must satisfy

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Enum of all source variants (`enum Source { Local(LocalFileSource), Beads(BeadsSource), Github(GithubSource) }`) | Adding a source requires editing the enum and exhaustive match arms throughout the codebase; violates the "no CLI change" requirement |
| Dynamic plugin loading (dlopen) | Far too complex for a local dev tool; binary compatibility issues across platforms; no meaningful benefit for the scale of this tool |
| Function pointers / closures | Less discoverable; no clear type boundary for the interface |

---

## Consequences

**Positive:**

- True open/closed principle for sources — new sources extend without modifying existing code
- Source adapters are independently testable via mock implementations of `TaskSource`
- The `TaskSource` trait is the single, stable contract for all source integration

**Negative / Tradeoffs:**

- Dynamic dispatch (vtable) — negligible for this use case
- The compiler cannot exhaustively verify that all `SourceKey` variants have a corresponding implementation; this must be enforced by the `SourceRegistry` constructor

---

## References

- [architecture.md](../spec/architecture.md) — `TaskSource` trait definition
- [tradeoffs.md](../spec/tradeoffs.md) — T1: Trait objects vs enum dispatch
- [responsibilities.md](../spec/responsibilities.md) — SourceRegistry card
