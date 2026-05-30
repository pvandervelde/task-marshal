# ADR-0006: BEADS Integration via `bd` CLI Subprocess

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

task-marshal needs to list eligible BEADS tasks and mark them complete. BEADS (`bd`) is a distributed graph issue tracker backed by a Dolt version-controlled SQL database stored in a `.beads/` directory.

The user requirements (§6.2) reference "the BEADS local database" and "the BEADS API," and the original config example used `db_path = ".llm/beads.db"`. These descriptions are not accurate to how BEADS works.

**Key facts about BEADS:**

- `bd` is a CLI tool (Go binary), not a library
- Data is stored in a Dolt database directory (`.beads/`), not a single `.db` file
- `bd ready --json` returns tasks with no open blockers (dependency filtering handled natively)
- `bd show <id> --json` returns full task details
- `bd close <id>` marks a task done; `bd close <id> --comment "..."` adds a completion note
- BEADS manages its own database discovery via git root or `BEADS_DIR` environment variable

**Version note:** The `bd` commands in this ADR were documented against BEADS v1.0.4. Before implementation, verify the `--json` flag and `bd close` syntax with `bd ready --help` and `bd close --help` for the installed version.

---

## Decision

BEADS operations are performed by spawning **`bd` CLI** as a subprocess using `std::process::Command`. task-marshal does not access the Dolt database directly. Arguments are passed as separate process arguments; no shell interpolation is used.

The `db_path` config key from the original requirements is replaced with an optional `beads_dir` key, which — if present — is passed as the `BEADS_DIR` environment variable to `bd` subprocess calls. If `beads_dir` is absent, `bd` uses its own discovery.

Relevant commands:

- `bd ready --json` — list tasks with no open blockers
- `bd show <native_id> --json` — get full task details
- `bd close <native_id>` — mark task done
- `bd close <native_id> --comment "<text>"` — mark done with note

If `bd` is not in PATH or the BEADS database cannot be opened, `BeadsSource` returns `SourceError::Unavailable`.

---

## Rationale

- BEADS is a CLI tool; there is no stable Rust library API to call
- The `.beads/issues.jsonl` export file is explicitly noted as "not the source of truth" in the BEADS README
- Direct Dolt database access would couple to BEADS' internal schema (subject to change) and bypass its concurrency controls
- `bd ready` handles dependency filtering natively — task-marshal does not need to implement dependency graph traversal for BEADS tasks
- `bd` handles its own database locking, transactions, and Dolt-specific semantics

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Direct Dolt SQL access via Rust driver | Couples to internal schema; bypasses BEADS concurrency controls; no stable Rust Dolt library; fragile |
| Parse `.beads/issues.jsonl` directly | BEADS README explicitly states this is not the source of truth; misses working-set state, Dolt branches, non-issue tables |
| Embed BEADS as a Go library via cgo | Cross-language FFI complexity; version pinning issues; not supported by BEADS |

---

## Consequences

**Positive:**

- Decoupled from BEADS internals; survives BEADS schema changes
- Dependency filtering is free (delegated to `bd ready`)
- `bd` manages all concurrency, locking, and Dolt semantics
- Consistent with how the BEADS project itself recommends integration

**Negative / Tradeoffs:**

- Runtime dependency on `bd` binary in PATH; graceful degradation if absent
- JSON parsing of `bd` output is required
- If `bd` hangs, task-marshal hangs (v1 has no subprocess timeout). **v2 extension point:** same pattern as ADR-0005 — spawn via `Command::spawn()` and race `child.wait()` against a timer thread.
- The corrected config schema (`beads_dir` not `db_path`) is a breaking change from the original requirements doc

---

## Config Schema Change

Original (incorrect, from user-requirements.md §9):

```toml
[sources.beads]
db_path = ".llm/beads.db"
```

Corrected:

```toml
[sources.beads]
# Optional: path to the .beads/ directory.
# Passed as BEADS_DIR env var to bd subprocess if set.
# beads_dir = ".beads"
```

See [assumptions.md](../spec/assumptions.md) — CA-01, CA-02.

---

## Security Notes

- Arguments passed as separate process arguments, never shell-interpolated
- `BEADS_DIR` value from config is validated as a path before use (no path traversal)
- See [security.md](../spec/security.md) — T1, T2

---

## References

- [architecture.md](../spec/architecture.md) — `BeadsSource` infrastructure section
- [responsibilities.md](../spec/responsibilities.md) — BeadsSource card
- [assumptions.md](../spec/assumptions.md) — CA-01, CA-02, CA-05
- [security.md](../spec/security.md) — T1, T2
- BEADS README: <https://github.com/gastownhall/beads>
- User requirements §6.2, §8.2 (beads behaviour)
