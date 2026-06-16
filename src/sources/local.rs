// Local markdown task file source adapter.
// See docs/spec/interfaces/sources.md — "LocalFileSource" section.

use std::path::PathBuf;

use crate::config::{ConfigError, LocalSourceConfig};
use crate::identity::NativeId;
use crate::selection::SelectionFilter;
use crate::sources::{SourceError, TaskSource};
use crate::{Task, TaskSummary};

// ── LocalFileSource ───────────────────────────────────────────────────────────

/// Reads and updates the local markdown task file (`.llm/tasks.md` by default).
///
/// Dependency resolution is performed within the file: a task is set to
/// TaskState::Blocked if any of its listed NativeId dependencies are not Done.
///
/// Completion uses atomic rename: write to temp file, then rename over original.
///
/// See docs/spec/interfaces/sources.md
pub struct LocalFileSource {
    /// Absolute, validated path to the task markdown file.
    path: PathBuf,
}

impl LocalFileSource {
    /// Creates a new `LocalFileSource` for the path in `config`.
    ///
    /// # Errors
    /// * `ConfigError::Invalid` — path is outside the project root (path traversal detected)
    pub fn new(config: &LocalSourceConfig) -> Result<Self, ConfigError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }
}

impl TaskSource for LocalFileSource {
    /// Reads and parses the markdown file; returns matching task summaries.
    ///
    /// Malformed task blocks are skipped with a warning to stderr.
    /// Returns SourceError::Unavailable if the file does not exist.
    fn list(&self, filter: &SelectionFilter) -> Result<Vec<TaskSummary>, SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }

    /// Returns the full Task for the given NativeId.
    ///
    /// Returns SourceError::NotFound if no task with that ID exists.
    fn get(&self, native_id: &NativeId) -> Result<Task, SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }

    /// Updates the task state to Done, optionally appending a completion annotation.
    ///
    /// Uses atomic rename (write temp → rename) to prevent partial updates.
    /// An empty comment string is treated as no comment (no annotation appended).
    fn complete(&self, native_id: &NativeId, comment: Option<&str>) -> Result<(), SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }
}
