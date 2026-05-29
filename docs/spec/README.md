# task-marshal — Architecture Specification

**Status:** Draft
**Source requirements:** [user-requirements.md](user-requirements.md)

---

## Workflow

```
User Requirements (user-requirements.md)
         ↓
Architecture Spec (this folder)   ← you are here
         ↓
Interface Designer → docs/spec/interfaces/ + typed stubs
         ↓
Planner → tasks.md
         ↓
Coder → implements against interfaces
```

---

## Documents

| Document | Purpose |
|---|---|
| [overview.md](overview.md) | System context, data flow diagrams, glossary summary |
| [vocabulary.md](vocabulary.md) | Canonical domain terms and precise definitions |
| [responsibilities.md](responsibilities.md) | CRC-style component responsibilities and collaborations |
| [architecture.md](architecture.md) | Clean architecture boundaries and dependency rules |
| [assertions.md](assertions.md) | Testable behavioral assertions for every requirement |
| [constraints.md](constraints.md) | Implementation rules to be enforced in code |
| [tradeoffs.md](tradeoffs.md) | Design alternatives considered, with rationale |
| [testing.md](testing.md) | Testing strategy by layer |
| [security.md](security.md) | Threat model and mitigations |
| [edge-cases.md](edge-cases.md) | Non-standard flows and failure modes |
| [operations.md](operations.md) | Distribution, config discovery, runtime behaviour |
| [assumptions.md](assumptions.md) | Challenged assumptions and their resolutions |

**ADRs:** [../adr/](../adr/)

---

## Key Architectural Decisions

1. **Rust, single static binary** — no runtime managed dependencies in the binary itself.
2. **Trait-based source abstraction (`TaskSource`)** — new sources can be added without changing the CLI interface.
3. **`<source>:<identifier>` task ID encoding** — self-routing, opaque to consumers.
4. **BEADS integration via `bd` CLI subprocess** — not direct Dolt database access.
5. **GitHub integration via `gh` CLI subprocess** — auth delegated to the tool already required by the environment.
6. **Deterministic selection** — priority-ordered, dependency-filtered, stable-sorted; no randomness.
7. **`Result<T, E>` error handling throughout** — exit codes: 0 = task found/operation succeeded, 1 = no task found, 2 = operational error.

---

## For the Interface Designer

Start with [vocabulary.md](vocabulary.md) to understand domain concepts, then [architecture.md](architecture.md) for boundary definitions.

**Central abstraction:** The `TaskSource` trait. All business logic depends only on this trait, never on concrete implementations.

**Types to define:**

- Domain: `Task`, `TaskId`, `NativeId`, `SourceKey`, `TaskBlock`, `TaskSummary`, `TaskState`, `Priority`, `Role`, `Dependency`
- Selection: `SelectionFilter`, `SortKey`
- Config: `Config`, `SourcesConfig`, `LocalSourceConfig`, `BeadsSourceConfig`, `GithubSourceConfig`
- Errors: `ConfigError`, `SourceError`, `SelectionError`, `CompletionError`, `CliError`

**Module boundaries follow business domain names** (e.g., `selection`, `sources`, `config`, `output`) — not architectural layer names.
