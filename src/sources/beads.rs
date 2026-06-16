// BEADS CLI source adapter.
// See docs/spec/interfaces/sources.md — "BeadsSource" section.

use crate::config::BeadsSourceConfig;
use crate::identity::NativeId;
use crate::selection::SelectionFilter;
use crate::sources::{SourceError, SubprocessRunner, TaskSource};
use crate::{Task, TaskSummary};

// ── BeadsSource ───────────────────────────────────────────────────────────────

/// Wraps the `bd` CLI binary to provide BEADS task source access.
///
/// CLI invocations (all use separate process arguments — no shell interpolation):
///   list:     bd ready --json                          (+ BEADS_DIR env if configured)
///   get:      bd show <native_id> --json
///   complete: bd close <native_id>
///   complete with comment: bd close <native_id> --comment <text>
///
/// `bd ready` already filters out blocked tasks; returns only tasks with no
/// open blockers.
///
/// See docs/spec/interfaces/sources.md
pub struct BeadsSource {
    config: BeadsSourceConfig,
    runner: Box<dyn SubprocessRunner>,
}

impl BeadsSource {
    /// Creates a new `BeadsSource`.
    ///
    /// `runner` is injected for testability.
    /// In production code, pass a `RealSubprocessRunner` instance.
    pub fn new(config: BeadsSourceConfig, runner: Box<dyn SubprocessRunner>) -> Self {
        BeadsSource { config, runner }
    }
}

impl TaskSource for BeadsSource {
    /// Invokes `bd ready --json` and parses the JSON task list.
    ///
    /// Passes BEADS_DIR env var if config.beads_dir is set.
    /// Returns SourceError::Unavailable if `bd` is not in PATH or exits non-zero
    /// due to DB unavailability.
    fn list(&self, filter: &SelectionFilter) -> Result<Vec<TaskSummary>, SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }

    /// Invokes `bd show <native_id> --json` and parses the full task.
    fn get(&self, native_id: &NativeId) -> Result<Task, SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }

    /// Invokes `bd close <native_id>` (with optional `--comment <text>`).
    ///
    /// Returns SourceError::Io if `bd` exits with a non-zero code.
    fn complete(&self, native_id: &NativeId, comment: Option<&str>) -> Result<(), SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }
}
