# Shared Types Registry

Catalog of all reusable types, traits, and patterns across task-marshal.
The coder must update this as implementation proceeds.

**Spec source:** `docs/spec/interfaces/`
**Source stubs:** `src/`

---

## Core Domain Types (`src/lib.rs`)

### `TaskState`

- **Purpose:** Lifecycle state of a task (Unstarted, InProgress, Blocked, Done)
- **Spec:** [interfaces/shared-types.md](interfaces/shared-types.md)
- **Usage:** All modules that filter or display tasks

### `Priority`

- **Purpose:** Urgency integer — lower number = higher priority. Default 2.
- **Spec:** [interfaces/shared-types.md](interfaces/shared-types.md)
- **Usage:** `TaskSummary`, `Task`, `SortKey`, `TaskSelector`

### `Role`

- **Purpose:** Agent role label. Free-form, case-insensitive equality.
- **Spec:** [interfaces/shared-types.md](interfaces/shared-types.md)
- **Usage:** `TaskSummary`, `Task`, `SelectionFilter`

### `Dependency`

- **Purpose:** Newtype over `NativeId` — within-source task dependency reference.
- **Spec:** [interfaces/shared-types.md](interfaces/shared-types.md)
- **Usage:** `Task.dependencies`

### `Task`

- **Purpose:** Full task content returned by `next` and `show`.
- **Spec:** [interfaces/shared-types.md](interfaces/shared-types.md)
- **Usage:** `TaskSource::get()`, `OutputFormatter::format_task()`

### `TaskSummary`

- **Purpose:** Abbreviated task info used in list output and selection.
- **Spec:** [interfaces/shared-types.md](interfaces/shared-types.md)
- **Usage:** `TaskSource::list()`, `TaskSelector`, `OutputFormatter::format_summary_list()`

### `TaskBlock`

- **Purpose:** Newtype over `String` — plain-text formatted task for stdout.
- **Spec:** [interfaces/shared-types.md](interfaces/shared-types.md)
- **Usage:** `OutputFormatter::format_task()` → written to stdout

---

## Identity Types (`src/identity.rs`)

### `SourceKey`

- **Purpose:** Source prefix enum — Local, Beads, Github. String values: `"local"`, `"beads"`, `"gh"`.
- **Spec:** [interfaces/identity.md](interfaces/identity.md)
- **Usage:** `TaskId`, `SourceRegistry`, `SelectionFilter`, `SourcesConfig`

### `NativeId`

- **Purpose:** Source-internal task identifier. Newtype over `String`.
- **Spec:** [interfaces/identity.md](interfaces/identity.md)
- **Usage:** `TaskSource::get()`, `TaskSource::complete()`, `Dependency`

### `TaskId`

- **Purpose:** Globally unique `<source>:<native_id>` identifier.
- **Spec:** [interfaces/identity.md](interfaces/identity.md)
- **Usage:** `Task`, `TaskSummary`, `CompletionError`, CLI `done` and `show` commands

### `TaskIdParser`

- **Purpose:** Pure parser: `&str` → `TaskId` or `ParseError`.
- **Spec:** [interfaces/identity.md](interfaces/identity.md)
- **Usage:** CLI Layer — `done` and `show` commands

### `ParseError`

- **Purpose:** Error from `TaskIdParser`. Variants: `InvalidTaskId`, `UnknownSourceKey`.
- **Spec:** [interfaces/identity.md](interfaces/identity.md)
- **Usage:** CLI Layer → exit code 2

---

## Selection Types (`src/selection.rs`)

### `SelectionFilter`

- **Purpose:** Combined filter criteria (role, source, include_all flag).
- **Spec:** [interfaces/selection.md](interfaces/selection.md)
- **Usage:** `TaskSource::list()`, `TaskSelector::select()`

### `SortKey`

- **Purpose:** Composite ordering key (priority, source_order, native_id_sort_key).
- **Spec:** [interfaces/selection.md](interfaces/selection.md)
- **Usage:** Internal to `TaskSelector`

### `ScoredCandidate`

- **Purpose:** `TaskSummary` paired with its source priority index.
- **Spec:** [interfaces/selection.md](interfaces/selection.md)
- **Usage:** Input to `TaskSelector::select()`

### `TaskSelector`

- **Purpose:** Deterministic selection engine. Pure function, no I/O.
- **Spec:** [interfaces/selection.md](interfaces/selection.md)
- **Usage:** CLI `next` command (after collecting summaries from all sources)

### `SelectionError`

- **Purpose:** Single variant: `NoTaskFound`. Triggers exit code 1.
- **Spec:** [interfaces/selection.md](interfaces/selection.md)

---

## Config Types (`src/config.rs`)

### `Config`

- **Purpose:** Complete parsed config including `config_dir`.
- **Spec:** [interfaces/config.md](interfaces/config.md)

### `SourcesConfig`

- **Purpose:** Priority list and per-source config sections.
- **Spec:** [interfaces/config.md](interfaces/config.md)

### `LocalSourceConfig`

- **Purpose:** Validated absolute path to local task file.
- **Spec:** [interfaces/config.md](interfaces/config.md)

### `BeadsSourceConfig`

- **Purpose:** Optional `beads_dir` path for `BEADS_DIR` env var injection.
- **Spec:** [interfaces/config.md](interfaces/config.md)

### `GithubSourceConfig`

- **Purpose:** `repo` (owner/repo) and `labels` for GitHub Issues filtering.
- **Spec:** [interfaces/config.md](interfaces/config.md)

### `ConfigLoader`

- **Purpose:** Walk-up discovery + TOML parsing + path validation.
- **Spec:** [interfaces/config.md](interfaces/config.md)

### `ConfigError`

- **Purpose:** Config failure variants: NotFound, Io, ParseFailed, Invalid.
- **Spec:** [interfaces/config.md](interfaces/config.md)

---

## Source Abstraction Types (`src/sources/mod.rs`)

### `TaskSource` (trait)

- **Purpose:** The central abstraction — list, get, complete operations.
- **Spec:** [interfaces/sources.md](interfaces/sources.md)
- **Usage:** All business logic; `Box<dyn TaskSource>` in `SourceRegistry`

### `SourceError`

- **Purpose:** Error from source operations: Unavailable, NotFound, Io, Parse.
- **Spec:** [interfaces/sources.md](interfaces/sources.md)
- **Usage:** `TaskSource` method return types; mapped to `CompletionError` for `done`

### `SourceRegistry`

- **Purpose:** Ordered list of sources built from `Config`; lookup by `SourceKey`.
- **Spec:** [interfaces/sources.md](interfaces/sources.md)
- **Usage:** CLI Layer — builds sources, looks up source for `done`/`show`

### `SubprocessRunner` (trait)

- **Purpose:** Abstraction over `std::process::Command` for testability.
- **Spec:** [interfaces/sources.md](interfaces/sources.md)
- **Usage:** `BeadsSource`, `GithubSource`; `Box<dyn SubprocessRunner>`

### `SubprocessOutput`

- **Purpose:** stdout, stderr, exit_code from a subprocess.
- **Spec:** [interfaces/sources.md](interfaces/sources.md)
- **Usage:** `SubprocessRunner::run()` return type

---

## Output Types (`src/output.rs`)

### `OutputFormatter`

- **Purpose:** Pure rendering: Task → TaskBlock, TaskSummary list → table, confirmations, warnings, errors.
- **Spec:** [interfaces/output.md](interfaces/output.md)
- **Usage:** CLI Layer — formats all output before writing to stdout/stderr

### `CompletionError`

- **Purpose:** CLI-layer error for `done` failures: TaskNotFound, SourceUnavailable, Io, PartialCompletion.
- **Spec:** [interfaces/output.md](interfaces/output.md)
- **Usage:** CLI `done` command handler → exit code 2

---

## Patterns

### Error Handling

- All fallible operations return `Result<T, E>` with typed error enums
- `unwrap()` / `expect()` forbidden in non-test code
- `thiserror` used for all error derivation

### Subprocess Safety

- All `bd` / `gh` invocations use `SubprocessRunner::run()` with args as separate items
- No shell string composition
- User-provided values (TaskId, comment) are argument values, never command strings

### Atomic File Writes

- `LocalFileSource::complete()` writes temp → renames atomically
- Failure leaves original unchanged

### Newtype Pattern

- `Priority(u32)`, `Role(String)`, `NativeId(String)`, `Dependency(NativeId)`, `TaskBlock(String)`
- Prevents misuse of raw primitives in domain operations
