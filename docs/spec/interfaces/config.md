# Configuration

**Module:** `src/config.rs`
**Architectural layer:** Core business logic — file I/O, no source interactions

---

## Overview

This module defines all configuration types and the `ConfigLoader` that discovers and parses
`task-marshal.toml`.

**RDD responsibilities:**

- Knowing: config file name (`task-marshal.toml`), walk-up discovery algorithm, TOML schema
- Doing: walk up the directory tree to find the config file; parse TOML into typed `Config`;
  validate required fields; resolve relative paths against the config file's directory
- Delegates to: filesystem (read file)
- Does NOT: modify the config file; infer missing config from environment variables

---

## Config Types

### `Config`

Complete parsed configuration for task-marshal.

```rust
pub struct Config {
    /// Configuration for all task sources.
    pub sources: SourcesConfig,

    /// Absolute path to the directory containing the config file.
    /// Used as the base for resolving relative paths in source configs.
    pub config_dir: PathBuf,
}
```

---

### `SourcesConfig`

Configuration for all task sources.

```rust
pub struct SourcesConfig {
    /// Ordered list of source keys. Determines priority order during task selection.
    /// Duplicates are silently deduplicated (first occurrence wins).
    /// Keys without a corresponding source config section are silently ignored.
    pub priority: Vec<SourceKey>,

    /// Local markdown task file source. Absent if `[sources.local]` not in config.
    pub local: Option<LocalSourceConfig>,

    /// BEADS task source. Absent if `[sources.beads]` not in config.
    pub beads: Option<BeadsSourceConfig>,

    /// GitHub Issues source. Absent if `[sources.gh]` not in config.
    pub github: Option<GithubSourceConfig>,
}
```

---

### `LocalSourceConfig`

Configuration for the local markdown task file source.

```rust
pub struct LocalSourceConfig {
    /// Absolute path to the local task markdown file.
    /// Resolved from the raw config value (default: `.llm/tasks.md`) relative to
    /// the directory containing `task-marshal.toml`.
    pub path: PathBuf,
}
```

**TOML field:** `[sources.local]` → `path = ".llm/tasks.md"` (optional, default used if absent)

**Path resolution:** Always resolved relative to the config file's directory. If CWD is
`/project/src/components` and config is at `/project/task-marshal.toml`, then
`path = ".llm/tasks.md"` resolves to `/project/.llm/tasks.md`.

**Security:** The resolved path must be a descendant of the config file's directory. Paths
that escape the project root (e.g., `../../etc/passwd`) are rejected with `ConfigError::Invalid`.

---

### `BeadsSourceConfig`

Configuration for the BEADS CLI source.

```rust
pub struct BeadsSourceConfig {
    /// Optional override for the BEADS database directory.
    /// If set, passed as the `BEADS_DIR` environment variable to `bd` subprocess.
    /// If absent, `bd` uses its own git-root-based discovery.
    /// Resolved to an absolute path relative to the config file's directory.
    pub beads_dir: Option<PathBuf>,
}
```

**TOML field:** `[sources.beads]` → `beads_dir = ".beads"` (entirely optional)

---

### `GithubSourceConfig`

Configuration for the GitHub CLI source.

```rust
pub struct GithubSourceConfig {
    /// GitHub repository in `owner/repo` format.
    pub repo: String,

    /// Labels used to filter issues (AND logic — all labels must match).
    /// Empty list means no label filter.
    pub labels: Vec<String>,
}
```

**TOML field:** `[sources.gh]`

**Validation:** `repo` must be non-empty and match the pattern `<owner>/<repo>` (contains
exactly one `/`). An invalid `repo` value causes `ConfigError::Invalid`.

**No credentials:** `gh` CLI manages GitHub authentication. No token fields exist in this config.

---

## `ConfigLoader`

Discovers and loads the configuration file.

```rust
pub struct ConfigLoader;

impl ConfigLoader {
    /// Walks up from `start_dir` to find `task-marshal.toml`, then parses it.
    ///
    /// # Arguments
    /// * `start_dir` — the directory to start walking from (typically CWD)
    ///
    /// # Returns
    /// The parsed and validated `Config` with all paths resolved.
    ///
    /// # Errors
    /// * `ConfigError::NotFound` — reached filesystem root without finding the config file
    /// * `ConfigError::Io` — file exists but could not be read
    /// * `ConfigError::ParseFailed` — TOML is malformed
    /// * `ConfigError::Invalid` — required field missing or value invalid (e.g., bad repo format)
    ///
    /// # Walk-up Algorithm
    /// 1. Check `start_dir` for `task-marshal.toml`
    /// 2. If not found, move to parent directory
    /// 3. Repeat until found or filesystem root reached
    /// 4. At filesystem root, return `ConfigError::NotFound`
    ///
    /// See docs/spec/interfaces/config.md for full contract.
    pub fn discover_and_load(start_dir: &Path) -> Result<Config, ConfigError>;

    /// Walks up from `start_dir` to find the path to `task-marshal.toml`.
    ///
    /// Useful when only the path is needed (e.g., for diagnostics).
    ///
    /// # Errors
    /// * `ConfigError::NotFound` — reached filesystem root without finding the config file
    pub fn discover_config_path(start_dir: &Path) -> Result<PathBuf, ConfigError>;

    /// Loads and validates the config from a known path.
    ///
    /// Resolves all relative paths in the config against the directory containing `path`.
    ///
    /// # Errors
    /// * `ConfigError::Io` — file could not be read
    /// * `ConfigError::ParseFailed` — TOML is malformed
    /// * `ConfigError::Invalid` — required field missing or value invalid
    pub fn load_from_path(path: &Path) -> Result<Config, ConfigError>;
}
```

---

## `ConfigError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// No `task-marshal.toml` found walking up from the given directory.
    #[error("no task-marshal.toml found walking up from '{start_dir}'")]
    NotFound { start_dir: PathBuf },

    /// The config file exists but could not be read.
    #[error("failed to read config file '{path}': {message}")]
    Io { path: PathBuf, message: String },

    /// The config file contains invalid TOML.
    #[error("TOML parse error in '{path}': {message}")]
    ParseFailed { path: PathBuf, message: String },

    /// The config is structurally valid TOML but fails semantic validation.
    #[error("invalid configuration: {message}")]
    Invalid { message: String },
}
```

**Exit code:** 2 for all variants.

---

## TOML Schema

The canonical TOML schema for `task-marshal.toml`:

```toml
[sources]
# Ordered list of source keys. Determines priority during task selection.
# Valid keys: "local", "beads", "gh"
priority = ["local", "beads", "gh"]

[sources.local]
# Path to the local markdown task file.
# Relative to this config file's directory. Default: ".llm/tasks.md"
path = ".llm/tasks.md"

[sources.beads]
# Optional: path to BEADS database directory.
# Relative to this config file's directory. If absent, bd uses git-root discovery.
# beads_dir = ".beads"

[sources.gh]
# GitHub repository in owner/repo format. Required if this section is present.
repo = "owner/repo"
# Labels for filtering issues (AND logic). Empty = no label filter.
labels = ["agent-task"]
```

> **Note:** The section name `[sources.gh]` uses the canonical SourceKey string `"gh"`, not
> `"github"`. The `operations.md` example showing `[sources.github]` contains a discrepancy;
> `"gh"` is the canonical key per vocabulary.md and ADR-0003.

---

## Test Requirements

| Scenario | Expected |
|---|---|
| Valid TOML with all sources | `Ok(Config)` with all source configs populated |
| Valid TOML, missing `[sources.local]` | `Ok(Config)` with `sources.local = None` |
| Source in priority list but no section | Source silently ignored |
| Duplicate in priority list | Deduplicated; first occurrence wins |
| No config file in any ancestor dir | `Err(ConfigError::NotFound)` |
| Config file found in ancestor directory | Found; paths resolved against config dir, not CWD |
| Invalid TOML | `Err(ConfigError::ParseFailed)` |
| Missing `repo` in `[sources.gh]` | `Err(ConfigError::Invalid)` |
| Path traversal: `path = "../../etc/passwd"` | `Err(ConfigError::Invalid)` |
| Relative path resolved against config dir | Absolute path is `<config_dir>/<path>` |
