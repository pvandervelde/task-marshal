# Component Responsibilities

CRC-style responsibility cards for every major component. Each card defines what the component **knows**, what it **does**, and who it **delegates to**.

---

## CLI Dispatcher

**Knows:**

- Command-line argument structure (commands, flags, their types and defaults)
- Mapping from command to handler
- Mapping from error type to exit code

**Does:**

- Parses arguments and validates flag types
- Routes to the appropriate command handler
- Maps `Result` errors to exit codes (0, 1, 2)
- Writes task output to stdout; warnings and errors to stderr

**Delegates to:**

- ConfigLoader (to load config before dispatching)
- TaskSelector (for `next` and `list`)
- TaskIdParser (for `done` and `show`)
- SourceRegistry (to build the source list)
- OutputFormatter (to format task blocks and summaries)

**Does NOT:**

- Implement any business logic
- Access task sources directly
- Make decisions about task selection

---

## ConfigLoader

**Knows:**

- The config file name (`task-marshal.toml`)
- The walk-up discovery algorithm (start at CWD, ascend to root)
- The TOML schema for all source types

**Does:**

- Walks up the directory tree to find the config file
- Parses the TOML config into a `Config` struct
- Validates required fields (e.g., repo for GitHub source)
- Returns a `ConfigError` if no file is found or parsing fails

**Delegates to:**

- Filesystem (read file)

**Does NOT:**

- Modify the config file
- Infer missing configuration from environment variables (that is the source adapters' responsibility)

---

## SourceRegistry

**Knows:**

- The mapping from `SourceKey` to source adapter constructor
- Which sources are present in the config

**Does:**

- Constructs the ordered list of `TaskSource` instances from `Config`
- Looks up a single source adapter by `SourceKey` (used by `done` and `show`)

**Delegates to:**

- LocalFileSource constructor
- BeadsSource constructor
- GithubSource constructor

**Does NOT:**

- Call any source methods directly
- Know about selection logic

---

## TaskSelector

**Knows:**

- The deterministic selection algorithm (priority order, dependency filtering, role filtering, sort key)
- What "blocked" means (any dependency not in `done` state)
- The `SelectionFilter` structure

**Does:**

- Accepts a list of `TaskSummary` candidates from multiple sources (already merged, source-annotated)
- Filters out blocked tasks (those with unresolved dependencies, per source-reported state)
- Applies role filter if present in `SelectionFilter`
- Sorts by `SortKey` (Priority asc, source order asc, NativeId asc)
- Returns the single highest-priority candidate, or `None`

**Delegates to:**

- Nothing — pure computation, no I/O

**Does NOT:**

- Call any source directly
- Perform I/O of any kind
- Involve randomness

---

## TaskIdParser

**Knows:**

- The `<SourceKey>:<NativeId>` format

**Does:**

- Parses a string into `(SourceKey, NativeId)`
- Returns `InvalidTaskId` if the format is malformed or the `SourceKey` is unknown

**Delegates to:**

- Nothing — pure parsing

---

## OutputFormatter

**Knows:**

- The `TaskBlock` format (plain text, no ANSI in task output)
- The `TaskSummary` table format for `list`

**Does:**

- Formats a `Task` into a `TaskBlock` string for stdout
- Formats a `Vec<TaskSummary>` into a summary table for stdout
- Formats completion confirmations for stdout
- Formats warnings for stderr (may include ANSI color)
- Formats errors for stderr (may include ANSI color)

**Delegates to:**

- Nothing — pure formatting

---

## LocalFileSource

**Knows:**

- The path to the task markdown file (from `LocalSourceConfig`)
- The local task file format (markdown task blocks — see [assumptions.md](assumptions.md))

**Does:**

- Reads and parses the markdown file into a list of tasks
- Filters tasks to those whose dependencies are all `done` (dependency resolution within the file)
- Returns a `TaskSummary` list for `list` and selection
- Returns a full `Task` for `show`
- Updates a task's state to `done` in-place via atomic rename (write temp, rename)
- Appends a completion annotation if `--comment` is provided

**Delegates to:**

- Filesystem (read, write, rename)
- Local task file parser (internal)

**Does NOT:**

- Perform network I/O
- Modify tasks other than the one specified in `done`

---

## BeadsSource

**Knows:**

- The `bd` CLI interface: `bd ready --json`, `bd show <id> --json`, `bd close <id> [--comment "..."]`
- How to map BEADS JSON output to `TaskSummary` and `Task`
- The optional `beads_dir` override for the `BEADS_DIR` environment variable
- That `bd ready` already handles dependency filtering (returns only tasks with no open blockers)

**Does:**

- Spawns `bd ready --json` to list eligible tasks
- Spawns `bd show <native_id> --json` to retrieve a full task
- Spawns `bd close <native_id>` (with optional `--comment`) to mark done
- Passes `BEADS_DIR=<beads_dir>` as an env var if `beads_dir` is configured
- Returns `SourceUnavailable` if `bd` is not in `PATH` or the BEADS DB cannot be opened

**Delegates to:**

- OS subprocess (`std::process::Command`)

**Does NOT:**

- Access the Dolt database directly
- Manage BEADS credentials or auth

---

## GithubSource

**Knows:**

- The `gh` CLI interface: `gh issue list --repo R --label L --json`, `gh issue view N --json`, `gh issue close N`, `gh issue comment N --body "..."`
- The configured `repo` and `labels` from `GithubSourceConfig`
- That GitHub issues do not have dependency metadata in v1 (all issues treated as unblocked)

**Does:**

- Spawns `gh issue list --repo <repo> --label <label> --json ...` to list eligible open issues
- Spawns `gh issue view <number> --json` to retrieve full issue content
- Spawns `gh issue close <number>` to close an issue
- Spawns `gh issue comment <number> --body "..."` if `--comment` is provided
- Returns `SourceUnavailable` if `gh` is not in `PATH`, not authenticated, or network is unavailable

**Delegates to:**

- OS subprocess (`std::process::Command`)

**Does NOT:**

- Manage GitHub tokens or OAuth flows
- Support dependency resolution (v1 limitation)
