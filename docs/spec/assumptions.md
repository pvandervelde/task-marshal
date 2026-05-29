# Challenged Assumptions

Assumptions that were explicitly questioned during architecture, with their resolutions.

---

## CA-01: BEADS config uses a `.db` file path

**Stated in requirements:** `[sources.beads] db_path = ".llm/beads.db"`

**Challenged because:**
BEADS (`bd`) is a CLI tool backed by a Dolt version-controlled SQL database stored in a directory (`.beads/`), not a single `.db` file. Accessing the Dolt database directly — even if the schema were known — would bypass BEADS' own concurrency controls, transaction guarantees, and future schema evolution.

**Resolution:**
The `beads` source adapter shells out to the `bd` CLI subprocess. The BEADS database location is managed by `bd` itself via its own git-root-based discovery or via the `BEADS_DIR` environment variable.

The corrected config entry is:
```toml
[sources.beads]
# Optional: override BEADS_DIR for bd subprocess invocations.
# If absent, bd uses its default git-root-based discovery.
# beads_dir = ".beads"
```

**Impact:** Adds `ADR-0006-beads-integration-via-bd-cli.md`. The `db_path` key is removed from the configuration schema.

---

## CA-02: BEADS task IDs are short hex strings

**Stated in requirements:** Examples show `beads:a3f9c2` as a BEADS task ID

**Challenged because:**
BEADS uses hash-based IDs in the format `bd-XXXX` (e.g., `bd-a1b2`), with optional hierarchy notation (`bd-a1b2.3` for subtasks). The format in the requirements does not match BEADS' actual output.

**Resolution:**
The `NativeId` for a BEADS task is the BEADS-native format as returned by `bd ready --json`. The full `TaskId` would therefore be `beads:bd-a1b2`, not `beads:a3f9c2`.

The examples in `user-requirements.md` section 7 are illustrative only and do not constrain the implementation format.

**Impact:** The `NativeId` for BEADS tasks will look like `bd-XXXX` in all task output.

---

## CA-03: GitHub dependencies are tracked in v1

**Implicit in requirements:** The selection rules (§8.1, rule 2) state "filter to tasks whose dependencies are all in a completed state" with no source-specific exception.

**Challenged because:**
GitHub Issues have no native dependency mechanism in the standard API. BEADS provides rich dependency tracking; the local file format can encode it explicitly. But GitHub Issues do not have a standard "blocks/depends on" field. Implementing cross-issue dependency tracking for GitHub would require a convention (e.g., parsing issue body for `Depends on #N` markers), which is fragile and out of scope.

**Resolution:**
In v1, GitHub Issues are treated as always unblocked. Dependency filtering is not applied to the `github` source. This is a **documented v1 limitation**, not a bug.

**Impact:** Noted in [edge-cases.md](edge-cases.md) and [operations.md](operations.md). The interface for `GithubSource.list()` returns all open (label-matching) issues as eligible, with `dependencies: []`.

---

## CA-04: Cross-source dependencies are in scope

**Implicit in requirements:** Task dependencies are described without restricting them to a single source.

**Challenged because:**
Resolving a cross-source dependency (e.g., a `local` task that depends on a `beads` task) requires task-marshal to query the BEADS source to check the dependency's state before returning the local task. This adds significant complexity: the selection algorithm must perform source lookups per dependency, sources must be queryable by individual ID, and the interaction is circular (selection depends on sources, sources are built per-selection-pass).

**Resolution:**
In v1, dependencies are **within-source only**. A `local` task's `Depends on` list may only reference other tasks in the same local file (by their native IDs, without source prefix). The `beads` source handles dependencies natively via `bd ready`. The `github` source does not support dependencies.

Cross-source dependency is a v2 feature.

**Impact:** Noted in [edge-cases.md](edge-cases.md). The dependency format in the local file is a list of bare `NativeId` values (e.g., `TASK-040`), not full `TaskId` values.

---

## CA-05: `bd` BEADS API reference implies a programmatic API

**Stated in requirements:** "Updates the task state via the BEADS API" (§8.2, beads behaviour)

**Challenged because:**
BEADS does not expose a stable Rust library or HTTP API for programmatic access. It is a CLI tool. The "API" referred to in the requirements is the `bd` command-line interface.

**Resolution:**
All BEADS operations are performed by spawning `bd` as a subprocess and capturing its output. No Rust bindings or direct database access are used.

**Impact:** Confirmed in `ADR-0006`. `BeadsSource` uses `std::process::Command`.

---

## CA-06: Synchronous I/O is sufficient

**Challenged because:**
If multiple sources are consulted in sequence and one has high latency (e.g., GitHub network call), total `next` response time could be noticeable.

**Resolution:**
For v1, `task-marshal` is a local development tool used interactively. Latency from a single `gh issue list` call is acceptable. The tool is not used in hot loops. Synchronous I/O keeps the implementation simple and eliminates async complexity.

If response time becomes a problem, sources can be parallelised (three threads, one per source) without changing the `TaskSource` trait. This is a v2 optimisation.

**Impact:** Synchronous `std::process::Command` confirmed in [constraints.md](constraints.md).

---

## CA-07: The `--role` filter excludes unassigned tasks

**Implicit assumption that could go either way.**

**Challenged because:**
An unassigned task has no explicit role. When a user passes `--role coder`, should unassigned tasks be excluded (strict filter) or included (liberal filter)?

**Resolution:**
Unassigned tasks are **eligible under any role filter**. The rationale: unassigned tasks have no role constraint, meaning any role can work them. Excluding them would hide valid work from agents.

This is documented as a business rule in [vocabulary.md](vocabulary.md) and tested in assertion A6.

---

## CA-08: The config file location is fixed

**Stated in requirements:** "Default location: `.llm/task-marshal.toml`, resolved from the working directory upward"

**Challenged because:**
The word "default" implies there may be an override mechanism (e.g., `--config` flag or `TASK_MARSHAL_CONFIG` env var). The requirements don't define one.

**Resolution:**
For v1, no override mechanism is implemented. The config is always discovered by walk-up from CWD. Adding a `--config` flag is straightforward in v2 if needed.

**Impact:** No `--config` flag in v1. The walk-up algorithm is the only discovery mechanism.
