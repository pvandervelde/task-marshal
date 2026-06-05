// Identity types and parsing.
// See docs/spec/interfaces/identity.md for full contract documentation.

use std::fmt;
use thiserror::Error;

// ── SourceKey ─────────────────────────────────────────────────────────────────

/// The source prefix component of a TaskId. Determines which adapter handles the task.
///
/// Canonical string values (used in TaskIds and TOML config):
///   Local  → "local"
///   Beads  → "beads"
///   Github → "gh"
///
/// See docs/spec/interfaces/identity.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceKey {
    Local,
    Beads,
    Github,
}

impl SourceKey {
    /// Returns the canonical string representation (e.g. `"local"`, `"beads"`, `"gh"`).
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKey::Local => "local",
            SourceKey::Beads => "beads",
            SourceKey::Github => "gh",
        }
    }

    /// Parses a string into a `SourceKey`. Returns `None` for unknown keys.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "local" => Some(SourceKey::Local),
            "beads" => Some(SourceKey::Beads),
            "gh" => Some(SourceKey::Github),
            _ => None,
        }
    }
}

impl fmt::Display for SourceKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ── NativeId ──────────────────────────────────────────────────────────────────

/// The identifier used by the source system for a task.
///
/// Must be non-empty and must not contain a colon (`:`).
///
/// See docs/spec/interfaces/identity.md
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NativeId(pub String);

impl NativeId {
    /// Returns the sort key string for tiebreaker ordering in TaskSelector.
    ///
    /// Numeric NativeIds (all digits) are zero-padded to 9 digits:
    ///   "187"  → "000000187"
    ///   "42"   → "000000042"
    ///
    /// Non-numeric NativeIds are returned as-is:
    ///   "TASK-042" → "TASK-042"
    ///   "bd-a1b2"  → "bd-a1b2"
    ///
    /// If a numeric ID has more than 9 digits, it is returned without truncation.
    pub fn sort_key(&self) -> String {
        unimplemented!("See docs/spec/interfaces/identity.md")
    }
}

impl fmt::Display for NativeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── TaskId ────────────────────────────────────────────────────────────────────

/// Globally unique, source-encoded task identifier in `<source>:<native_id>` format.
///
/// Self-routing: the `source_key` field tells task-marshal which adapter to use.
///
/// See docs/spec/interfaces/identity.md
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskId {
    pub source_key: SourceKey,
    pub native_id: NativeId,
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.source_key, self.native_id)
    }
}

// ── TaskIdParser ──────────────────────────────────────────────────────────────

/// Pure parser: converts a `<source>:<native_id>` string into a typed `TaskId`.
///
/// See docs/spec/interfaces/identity.md
pub struct TaskIdParser;

impl TaskIdParser {
    /// Parses a task ID string in `<source>:<native_id>` format.
    ///
    /// Splits on the **first** colon only — NativeId values may contain colons
    /// (e.g. BEADS hierarchical IDs like `"bd-a1b2.3"`).
    ///
    /// # Errors
    /// * `ParseError::InvalidTaskId` — no colon present, or source/native parts are empty
    /// * `ParseError::UnknownSourceKey` — source prefix is not a known SourceKey string
    pub fn parse(input: &str) -> Result<TaskId, ParseError> {
        unimplemented!("See docs/spec/interfaces/identity.md")
    }
}

// ── ParseError ────────────────────────────────────────────────────────────────

/// Error returned when a task ID string cannot be parsed into a `TaskId`.
///
/// Both variants map to exit code 2.
///
/// See docs/spec/interfaces/identity.md
#[derive(Debug, Error)]
pub enum ParseError {
    /// The input does not contain a colon, or the source/native parts are empty.
    #[error("invalid task ID '{input}': expected '<source>:<id>' format")]
    InvalidTaskId { input: String },

    /// The source key prefix is not a recognised SourceKey.
    #[error("unknown source key '{key}' in task ID '{input}': valid keys are local, beads, gh")]
    UnknownSourceKey { key: String, input: String },
}
