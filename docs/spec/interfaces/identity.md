# Identity Types and Parsing

**Module:** `src/identity.rs`
**Architectural layer:** Core domain — pure parsing, no I/O

---

## Overview

This module defines task identity types (`TaskId`, `NativeId`, `SourceKey`) and the parser
that converts user-provided strings into typed identity values.

**RDD responsibilities:**

- Knowing: the `<SourceKey>:<NativeId>` format; valid `SourceKey` values
- Doing: parse a `TaskId` string; encode a `(SourceKey, NativeId)` pair into a `TaskId` string
- Delegates to: nothing — pure computation

---

## Types

### `SourceKey`

The source prefix component of a `TaskId`.

```rust
pub enum SourceKey {
    Local,
    Beads,
    Github,
}
```

**String representations:**

| Variant | `as_str()` | Used in TOML section | Used in TaskId |
|---|---|---|---|
| `Local` | `"local"` | `[sources.local]` | `local:TASK-042` |
| `Beads` | `"beads"` | `[sources.beads]` | `beads:bd-a1b2` |
| `Github` | `"gh"` | `[sources.gh]` | `gh:187` |

**Methods:**

```rust
impl SourceKey {
    /// Returns the canonical string representation used in TaskIds and TOML config.
    pub fn as_str(&self) -> &'static str;

    /// Parses a string into a SourceKey. Returns None for unknown keys.
    pub fn from_str(s: &str) -> Option<Self>;
}
```

**Derives:** `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`

**Display:** Outputs `as_str()` value.

---

### `NativeId`

The identifier used by the source system for a task.

```rust
pub struct NativeId(pub String);
```

**Examples:**

- `NativeId("TASK-042")` — local file
- `NativeId("bd-a1b2")` — BEADS (may contain dots for hierarchy, e.g., `"bd-a1b2.3"`)
- `NativeId("187")` — GitHub issue number

**Constraints:**

- Must be non-empty
- Must not contain a colon (`:`) — the colon is the `TaskId` delimiter

**Sort key normalization:** For tiebreaker sorting in `TaskSelector`, numeric NativeIds
(consisting entirely of digits) are zero-padded to 9 digits. Non-numeric NativeIds use
direct lexicographic comparison. The displayed NativeId is always the original value.

```rust
impl NativeId {
    /// Returns the sort key string for this NativeId.
    ///
    /// Numeric NativeIds are zero-padded to 9 digits:
    ///   "187" → "000000187"
    /// Non-numeric NativeIds are returned as-is:
    ///   "TASK-042" → "TASK-042"
    ///   "bd-a1b2" → "bd-a1b2"
    pub fn sort_key(&self) -> String;
}
```

**Derives:** `Debug`, `Clone`, `PartialEq`, `Eq`, `Hash`

---

### `TaskId`

The globally unique, source-encoded identifier for a task.

```rust
pub struct TaskId {
    pub source_key: SourceKey,
    pub native_id: NativeId,
}
```

**Format:** `<source_key>:<native_id>` — e.g., `local:TASK-042`, `beads:bd-a1b2`, `gh:187`

**Property:** Self-routing — `source_key` tells task-marshal which adapter to use.

**Methods:**

```rust
impl TaskId {
    /// Returns the formatted `<source>:<native_id>` string representation.
    pub fn to_string(&self) -> String;
}
```

**Display:** Outputs `<source_key>:<native_id>` format.

**Derives:** `Debug`, `Clone`, `PartialEq`, `Eq`

---

## `TaskIdParser`

Pure parser: converts a string into a typed `TaskId` or returns a `ParseError`.

```rust
pub struct TaskIdParser;

impl TaskIdParser {
    /// Parses a task ID string in `<source>:<native_id>` format.
    ///
    /// # Arguments
    /// * `input` — the raw task ID string (e.g., `"local:TASK-042"`, `"gh:187"`)
    ///
    /// # Returns
    /// The parsed `TaskId` on success.
    ///
    /// # Errors
    /// * `ParseError::InvalidTaskId` — no colon in input, or empty source/native parts
    /// * `ParseError::UnknownSourceKey` — source part does not match any known `SourceKey`
    ///
    /// # Parsing Rules
    /// 1. Split on the **first** colon only — NativeId may contain colons (BEADS hierarchy)
    /// 2. Source part is trimmed and matched case-sensitively against known SourceKey strings
    /// 3. NativeId part must be non-empty after trimming
    pub fn parse(input: &str) -> Result<TaskId, ParseError>;
}
```

**Example:**

```rust
let id = TaskIdParser::parse("gh:187")?;
assert_eq!(id.source_key, SourceKey::Github);
assert_eq!(id.native_id.0, "187");

let id = TaskIdParser::parse("beads:bd-a1b2.3")?;
assert_eq!(id.source_key, SourceKey::Beads);
assert_eq!(id.native_id.0, "bd-a1b2.3");  // colon split on first only

TaskIdParser::parse("unknown:TASK-001"); // Err(ParseError::UnknownSourceKey { .. })
TaskIdParser::parse("local");            // Err(ParseError::InvalidTaskId { .. })
TaskIdParser::parse("local:");           // Err(ParseError::InvalidTaskId { .. }) — empty native
```

---

## `ParseError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// The input string does not contain a colon, or the source/native parts are empty.
    #[error("invalid task ID '{input}': expected '<source>:<id>' format")]
    InvalidTaskId { input: String },

    /// The source key prefix is not a recognised SourceKey.
    #[error("unknown source key '{key}' in task ID '{input}': valid keys are local, beads, gh")]
    UnknownSourceKey { key: String, input: String },
}
```

**Exit code:** Both variants map to exit code 2.

---

## Test Requirements

| Case | Expected result |
|---|---|
| `"local:TASK-042"` | `Ok(TaskId { Local, NativeId("TASK-042") })` |
| `"beads:bd-a1b2"` | `Ok(TaskId { Beads, NativeId("bd-a1b2") })` |
| `"gh:187"` | `Ok(TaskId { Github, NativeId("187") })` |
| `"beads:bd-a1b2.3"` | `Ok(TaskId { Beads, NativeId("bd-a1b2.3") })` — first-colon split |
| `"unknown:TASK-001"` | `Err(ParseError::UnknownSourceKey { key: "unknown", .. })` |
| `"local"` | `Err(ParseError::InvalidTaskId { .. })` — no colon |
| `"local:"` | `Err(ParseError::InvalidTaskId { .. })` — empty native_id |
| `":TASK-042"` | `Err(ParseError::InvalidTaskId { .. })` — empty source |
| `""` | `Err(ParseError::InvalidTaskId { .. })` |

**NativeId sort key tests:**

| Input | `sort_key()` |
|---|---|
| `"187"` | `"000000187"` |
| `"42"` | `"000000042"` |
| `"TASK-042"` | `"TASK-042"` |
| `"bd-a1b2"` | `"bd-a1b2"` |
| `"123456789"` | `"123456789"` — exactly 9 digits, no padding |
| `"1234567890"` | `"1234567890"` — >9 digits, no truncation |
