// Sources module: TaskSource trait, SourceRegistry, SubprocessRunner, and error types.
// See docs/spec/interfaces/sources.md for full contract documentation.

pub mod beads;
pub mod github;
pub mod local;

use thiserror::Error;

use crate::config::{Config, ConfigError};
use crate::identity::{NativeId, SourceKey};
use crate::selection::SelectionFilter;
use crate::{Task, TaskSummary};

// ── SourceError ───────────────────────────────────────────────────────────────

/// Error returned by `TaskSource` operations.
///
/// See docs/spec/interfaces/sources.md
#[derive(Debug, Error)]
pub enum SourceError {
    /// The source system is unreachable (binary missing, network down, DB locked).
    /// Callers should log a warning to stderr and skip this source; do not exit code 2.
    #[error("source unavailable: {message}")]
    Unavailable { message: String },

    /// The requested task does not exist in this source.
    #[error("task not found: {native_id}")]
    NotFound { native_id: NativeId },

    /// An underlying I/O failure occurred.
    #[error("I/O error: {message}")]
    Io { message: String },

    /// The source's output could not be parsed into domain types.
    #[error("parse error: {message}")]
    Parse { message: String },

    /// A multi-step operation partially succeeded: the primary action (e.g. close/mark-done)
    /// completed but the secondary action (e.g. post a comment) failed.
    /// The task IS marked done; only the annotation is missing.
    #[error("partial completion: {message}")]
    PartialCompletion { message: String },
}

// ── TaskSource ────────────────────────────────────────────────────────────────

/// The central abstraction all source adapters implement.
///
/// All business logic interacts with task sources exclusively via this trait.
/// Object-safe — use as `Box<dyn TaskSource>`.
///
/// See docs/spec/interfaces/sources.md
pub trait TaskSource {
    /// Returns all tasks from this source that match the filter.
    ///
    /// Source adapters are responsible for setting TaskState::Blocked on tasks
    /// whose dependencies are unresolved. The returned summaries reflect the
    /// actual state as known by the source.
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — source unreachable; caller should skip + warn
    /// * `SourceError::Io` — underlying I/O failure
    /// * `SourceError::Parse` — source output could not be interpreted
    ///
    /// Must not modify the source.
    fn list(&self, filter: &SelectionFilter) -> Result<Vec<TaskSummary>, SourceError>;

    /// Returns the full task content for the given native ID.
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — source unreachable
    /// * `SourceError::NotFound` — no task with this native_id in this source
    /// * `SourceError::Io` — underlying I/O failure
    /// * `SourceError::Parse` — source output could not be interpreted
    ///
    /// Must not modify the source.
    fn get(&self, native_id: &NativeId) -> Result<Task, SourceError>;

    /// Marks the task as done and propagates the change to the source system.
    ///
    /// # Arguments
    /// * `native_id` — the task to complete
    /// * `comment` — optional annotation; None or Some("") means no annotation
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — source unreachable
    /// * `SourceError::NotFound` — task does not exist in this source
    /// * `SourceError::Io` — I/O failure preventing completion
    /// * `SourceError::Parse` — source response could not be interpreted
    ///
    /// Must be atomic: either fully complete or no change.
    /// Must be idempotent: completing an already-done task must succeed.
    fn complete(&self, native_id: &NativeId, comment: Option<&str>) -> Result<(), SourceError>;
}

// ── SubprocessOutput ──────────────────────────────────────────────────────────

/// The output of a subprocess invocation.
///
/// See docs/spec/interfaces/sources.md
#[derive(Debug, Clone)]
pub struct SubprocessOutput {
    /// Raw stdout bytes from the subprocess.
    pub stdout: Vec<u8>,
    /// Raw stderr bytes from the subprocess.
    pub stderr: Vec<u8>,
    /// Exit code of the subprocess (0 = success).
    pub exit_code: i32,
}

// ── SubprocessRunner ──────────────────────────────────────────────────────────

/// Abstraction over subprocess execution.
///
/// Production implementation calls `std::process::Command`.
/// Test implementation returns a configured (stdout, stderr, exit_code) triple.
///
/// Object-safe — use as `Box<dyn SubprocessRunner>`.
///
/// Security: implementations MUST pass arguments as separate process arguments.
/// Shell string composition is FORBIDDEN.
///
/// See docs/spec/interfaces/sources.md
pub trait SubprocessRunner {
    /// Runs an external program with the given arguments and environment variables.
    ///
    /// # Arguments
    /// * `program` — the binary to execute (e.g., `"bd"`, `"gh"`)
    /// * `args` — arguments as separate process arguments (no shell interpolation)
    /// * `env_vars` — additional environment variable pairs to set for this invocation
    ///
    /// # Returns
    /// `SubprocessOutput` if the process was spawned and exited (any exit code).
    ///
    /// # Errors
    /// * `SourceError::Unavailable` — binary not found in PATH (ENOENT) or cannot spawn
    /// * `SourceError::Io` — I/O error capturing stdout/stderr
    fn run(
        &self,
        program: &str,
        args: &[&str],
        env_vars: &[(String, String)],
    ) -> Result<SubprocessOutput, SourceError>;
}

// ── RealSubprocessRunner ──────────────────────────────────────────────────────

/// Production implementation of `SubprocessRunner` using `std::process::Command`.
///
/// Passes all arguments as separate process arguments; no shell interpolation.
///
/// See docs/spec/interfaces/sources.md
pub struct RealSubprocessRunner;

impl SubprocessRunner for RealSubprocessRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        env_vars: &[(String, String)],
    ) -> Result<SubprocessOutput, SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }
}

// ── SourceRegistry ────────────────────────────────────────────────────────────

/// Builds and stores the ordered list of source adapters from parsed config.
///
/// See docs/spec/interfaces/sources.md
pub struct SourceRegistry {
    /// Source adapters in priority order, paired with their SourceKey.
    ordered_sources: Vec<(SourceKey, Box<dyn TaskSource>)>,
}

impl SourceRegistry {
    /// Builds the source registry from the parsed config.
    ///
    /// Sources are instantiated in priority order. Sources in the priority list
    /// without a corresponding config section are silently skipped.
    ///
    /// # Errors
    /// * `ConfigError::Invalid` — a source config contains an invalid path
    pub fn from_config(config: &Config) -> Result<Self, ConfigError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }

    /// Returns all source adapters in priority order, paired with their SourceKey.
    pub fn ordered_sources(&self) -> &[(SourceKey, Box<dyn TaskSource>)] {
        &self.ordered_sources
    }

    /// Looks up a source adapter by SourceKey.
    ///
    /// Returns `None` if the key is not in the configured sources.
    pub fn find_source(&self, key: &SourceKey) -> Option<&dyn TaskSource> {
        self.ordered_sources
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, source)| source.as_ref())
    }

    /// Returns the 0-indexed source priority position for the given key.
    ///
    /// Returns `None` if the key is not in the configured sources.
    pub fn source_order(&self, key: &SourceKey) -> Option<usize> {
        self.ordered_sources.iter().position(|(k, _)| k == key)
    }
}
