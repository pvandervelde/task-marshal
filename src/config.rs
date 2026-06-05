// Configuration types and loader.
// See docs/spec/interfaces/config.md for full contract documentation.

use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::identity::SourceKey;

// ── Config ────────────────────────────────────────────────────────────────────

/// Complete parsed configuration for task-marshal.
///
/// See docs/spec/interfaces/config.md
#[derive(Debug, Clone)]
pub struct Config {
    /// Configuration for all task sources.
    pub sources: SourcesConfig,

    /// Absolute path to the directory containing the config file.
    /// Used as the base for resolving relative paths in source configs.
    pub config_dir: PathBuf,
}

// ── SourcesConfig ─────────────────────────────────────────────────────────────

/// Configuration for all task sources.
///
/// See docs/spec/interfaces/config.md
#[derive(Debug, Clone)]
pub struct SourcesConfig {
    /// Ordered list of source keys; determines priority during task selection.
    /// Duplicates are silently deduplicated (first occurrence kept).
    /// Keys without a config section are silently ignored.
    pub priority: Vec<SourceKey>,

    /// Local markdown task file source. None if `[sources.local]` not present.
    pub local: Option<LocalSourceConfig>,

    /// BEADS task source. None if `[sources.beads]` not present.
    pub beads: Option<BeadsSourceConfig>,

    /// GitHub Issues source. None if `[sources.gh]` not present.
    pub github: Option<GithubSourceConfig>,
}

// ── LocalSourceConfig ─────────────────────────────────────────────────────────

/// Configuration for the local markdown task file source.
///
/// See docs/spec/interfaces/config.md
#[derive(Debug, Clone)]
pub struct LocalSourceConfig {
    /// Absolute path to the local task markdown file.
    /// Resolved from the raw config value relative to the config file's directory.
    /// Default raw value: `.llm/tasks.md`
    pub path: PathBuf,
}

// ── BeadsSourceConfig ─────────────────────────────────────────────────────────

/// Configuration for the BEADS CLI source.
///
/// See docs/spec/interfaces/config.md
#[derive(Debug, Clone)]
pub struct BeadsSourceConfig {
    /// Optional path to the BEADS database directory.
    /// If set, passed as `BEADS_DIR` env var to `bd` subprocess.
    /// Resolved to an absolute path relative to the config file's directory.
    pub beads_dir: Option<PathBuf>,
}

// ── GithubSourceConfig ────────────────────────────────────────────────────────

/// Configuration for the GitHub Issues CLI source.
///
/// See docs/spec/interfaces/config.md
#[derive(Debug, Clone)]
pub struct GithubSourceConfig {
    /// GitHub repository in `owner/repo` format. Required.
    pub repo: String,

    /// Labels used to filter issues. All labels must match (AND logic).
    /// Empty list means no label filter.
    pub labels: Vec<String>,
}

// ── ConfigError ───────────────────────────────────────────────────────────────

/// Error returned when the configuration cannot be found, read, or validated.
///
/// All variants map to exit code 2.
///
/// See docs/spec/interfaces/config.md
#[derive(Debug, Error)]
pub enum ConfigError {
    /// No `task-marshal.toml` found walking up from the given directory.
    #[error("no task-marshal.toml found walking up from '{}'", start_dir.display())]
    NotFound { start_dir: PathBuf },

    /// The config file exists but could not be read.
    #[error("failed to read config file '{}': {message}", path.display())]
    Io { path: PathBuf, message: String },

    /// The config file contains invalid TOML.
    #[error("TOML parse error in '{}': {message}", path.display())]
    ParseFailed { path: PathBuf, message: String },

    /// The config is structurally valid TOML but fails semantic validation
    /// (e.g. missing required field, invalid repo format, path traversal).
    #[error("invalid configuration: {message}")]
    Invalid { message: String },
}

// ── ConfigLoader ──────────────────────────────────────────────────────────────

/// Discovers and loads `task-marshal.toml` by walking up the directory tree.
///
/// See docs/spec/interfaces/config.md
pub struct ConfigLoader;

impl ConfigLoader {
    /// Walks up from `start_dir` to find `task-marshal.toml`, then parses it.
    ///
    /// All relative paths in the config are resolved against the config file's directory.
    ///
    /// # Errors
    /// * `ConfigError::NotFound` — reached filesystem root without finding the config file
    /// * `ConfigError::Io` — file exists but could not be read
    /// * `ConfigError::ParseFailed` — TOML is malformed
    /// * `ConfigError::Invalid` — required field missing, value invalid, or path traversal
    pub fn discover_and_load(start_dir: &Path) -> Result<Config, ConfigError> {
        unimplemented!("See docs/spec/interfaces/config.md")
    }

    /// Walks up from `start_dir` to find the path to `task-marshal.toml`.
    ///
    /// # Errors
    /// * `ConfigError::NotFound` — reached filesystem root without finding the config file
    pub fn discover_config_path(start_dir: &Path) -> Result<PathBuf, ConfigError> {
        unimplemented!("See docs/spec/interfaces/config.md")
    }

    /// Loads and validates the config from a known path.
    ///
    /// Resolves all relative paths against the directory containing `path`.
    ///
    /// # Errors
    /// * `ConfigError::Io` — file could not be read
    /// * `ConfigError::ParseFailed` — TOML is malformed
    /// * `ConfigError::Invalid` — required field missing or value invalid
    pub fn load_from_path(path: &Path) -> Result<Config, ConfigError> {
        unimplemented!("See docs/spec/interfaces/config.md")
    }
}
