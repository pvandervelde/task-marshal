# ADR-0004: Configuration Discovery via Walk-Up

**Date:** 2026-05-24
**Status:** Accepted

---

## Context

task-marshal needs to locate its configuration file (`.llm/task-marshal.toml`) when invoked from any directory within a project. Developers and agents may invoke it from:

- The project root
- Any subdirectory (e.g., `src/`, `src/components/auth/`)

The config must be found consistently without requiring the user to specify an explicit path.

---

## Decision

task-marshal discovers its config file by **walking up from the current working directory**, checking for `.llm/task-marshal.toml` at each level, stopping at the first match. If the filesystem root is reached without finding the file, `ConfigError` is returned.

Relative paths inside the config file are resolved relative to the **config file's directory**, not the CWD.

No `--config` flag or environment variable override is provided in v1.

---

## Rationale

- Mirrors the well-established convention used by `git` (`.git/` discovery) and many other tools
- Works from any project subdirectory without user configuration
- Relative paths resolved from the config file's directory ensures consistency regardless of invocation location
- Simple to implement and reason about

---

## Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Fixed path from CWD (`.llm/task-marshal.toml` relative to CWD only) | Breaks if invoked from a subdirectory; requires users to always run from project root |
| Fixed absolute path or `$HOME` config | Not project-specific; cannot support multiple projects |
| `--config <path>` flag | Adds CLI complexity; not needed for typical single-project use |
| `TASK_MARSHAL_CONFIG` env var | Requires users to set env vars; verbose for daily use |
| XDG config directory (`~/.config/task-marshal/`) | Not project-scoped; cannot support multiple projects with different sources |

---

## Consequences

**Positive:**

- Works from any project subdirectory — zero user friction
- Consistent with developer expectations (git-style discovery)
- Config is co-located with the project (`.llm/` directory), suitable for version control

**Negative / Tradeoffs:**

- A project nested inside another project with its own config may unexpectedly inherit the parent's config if the inner project has no config file. This is documented as expected behavior (the nearest ancestor wins).
- No override mechanism in v1 — advanced use cases (monorepos, non-standard layouts) cannot override the discovery path without a code change

---

## References

- [operations.md](../spec/operations.md) — config discovery algorithm
- [edge-cases.md](../spec/edge-cases.md) — EC-15: config found in ancestor directory
- [assumptions.md](../spec/assumptions.md) — CA-08: no `--config` override in v1
- User requirements §9 — Configuration
