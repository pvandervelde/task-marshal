// GitHub CLI source adapter.
// See docs/spec/interfaces/sources.md — "GithubSource" section.

use crate::config::GithubSourceConfig;
use crate::identity::NativeId;
use crate::selection::SelectionFilter;
use crate::sources::{SourceError, SubprocessRunner, TaskSource};
use crate::{Task, TaskSummary};

// ── GithubSource ──────────────────────────────────────────────────────────────

/// Wraps the `gh` CLI binary to provide GitHub Issues task source access.
///
/// CLI invocations (all use separate process arguments — no shell interpolation):
///   list:     gh issue list --repo <repo> [--label <l1> --label <l2> ...] --json ... --state open
///   get:      gh issue view <native_id> --repo <repo> --json ...
///   complete (no comment):   gh issue close <native_id> --repo <repo>
///   complete (with comment): gh issue close <native_id> --repo <repo>
///                            gh issue comment <native_id> --repo <repo> --body <text>
///
/// v1 constraint: GitHub issues have no dependency tracking.
/// All issues returned by list are treated as TaskState::Unstarted,
/// or TaskState::InProgress if the GitHub issue is assigned.
///
/// See docs/spec/interfaces/sources.md
pub struct GithubSource {
    config: GithubSourceConfig,
    runner: Box<dyn SubprocessRunner>,
}

impl GithubSource {
    /// Creates a new `GithubSource`.
    ///
    /// `runner` is injected for testability.
    /// In production code, pass a `RealSubprocessRunner` instance.
    pub fn new(config: GithubSourceConfig, runner: Box<dyn SubprocessRunner>) -> Self {
        GithubSource { config, runner }
    }
}

impl TaskSource for GithubSource {
    /// Invokes `gh issue list` and parses the JSON issue list.
    ///
    /// Passes `--label` for each label in config.labels (AND filtering).
    /// If config.labels is empty, no --label args are added.
    /// Returns SourceError::Unavailable if `gh` is not in PATH or exits non-zero
    /// (e.g. auth failure, network error).
    fn list(&self, filter: &SelectionFilter) -> Result<Vec<TaskSummary>, SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }

    /// Invokes `gh issue view <native_id> --repo <repo> --json ...` and parses the full task.
    fn get(&self, native_id: &NativeId) -> Result<Task, SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }

    /// Invokes `gh issue close <native_id>`, then optionally `gh issue comment <native_id>`.
    ///
    /// If close succeeds but comment fails, returns SourceError::Io with a message
    /// noting the partial state (task IS marked done, annotation missing).
    /// An empty comment string is treated as no comment (only close is invoked).
    fn complete(&self, native_id: &NativeId, comment: Option<&str>) -> Result<(), SourceError> {
        unimplemented!("See docs/spec/interfaces/sources.md")
    }
}
