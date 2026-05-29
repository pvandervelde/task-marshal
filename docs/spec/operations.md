# Operations

---

## Distribution

- task-marshal is distributed as a **single static binary** with no runtime managed dependencies
- The binary is named `task-marshal` (or `task-marshal.exe` on Windows)
- No installer is required — copy the binary to a directory in `PATH`
- External runtime dependencies (not bundled): `bd` CLI (for BEADS source), `gh` CLI (for GitHub source)
- These external tools must be installed and configured separately; task-marshal fails gracefully if they are absent

---

## Configuration Discovery

task-marshal locates its config file by walking up the directory tree:

1. Start at the current working directory
2. Check for `.llm/task-marshal.toml` in the current directory
3. If not found, move to the parent directory
4. Repeat until the file is found or the filesystem root is reached
5. If the filesystem root is reached without finding the file, fail with `ConfigError`

**Resolution of relative paths in config:**
All relative paths specified in the config file (e.g., `sources.local.path`) are resolved relative to the **directory containing the config file**, not the CWD. This ensures consistent behaviour regardless of which subdirectory the user invokes task-marshal from.

Example:
```
/project/.llm/task-marshal.toml  → config file
/project/.llm/tasks.md           → resolved from path = ".llm/tasks.md" relative to /project/
```

---

## Config File

Default location: `.llm/task-marshal.toml` (in the project root, discovered by walk-up)

```toml
[sources]
priority = ["local", "beads", "github"]

[sources.local]
# Path to the local markdown task file.
# Relative to this config file's directory.
path = ".llm/tasks.md"

[sources.beads]
# Optional: override the BEADS database directory.
# If set, passed as BEADS_DIR env var to bd subprocess.
# If absent, bd uses its own git-root-based discovery.
# beads_dir = ".beads"

[sources.github]
# GitHub repository in owner/repo format.
repo = "owner/repo"
# Labels used to filter issues. All labels must match (AND logic).
labels = ["agent-task"]
```

Notes:
- All `[sources.*]` sections are optional. Sources not present in config are not used.
- Sources listed in `priority` but without a corresponding `[sources.*]` section are silently ignored.
- **No credentials belong in this file.** GitHub auth is managed by `gh`; BEADS auth (if any) is managed by `bd`.

---

## Exit Codes

| Code | Meaning | Commands |
|---|---|---|
| 0 | Success | All commands |
| 1 | No task found | `next` only |
| 2 | Operational error | All commands |

Exit code 1 is distinct from exit code 2 to allow agent skill files to detect the empty-queue case programmatically:

```bash
task-marshal next --role coder
status=$?
if [ $status -eq 1 ]; then
  echo "Queue is empty — nothing to do"
elif [ $status -eq 2 ]; then
  echo "Error — check configuration"
fi
```

---

## Output Conventions

| Channel | Content |
|---|---|
| stdout | Task content: `TaskBlock`, summary table, completion confirmation |
| stderr | Warnings (skipped sources), errors, diagnostic messages |

- stdout output contains **no ANSI escape codes** — suitable for direct LLM context ingestion
- stderr output may contain ANSI color codes for readability
- Warnings for unavailable sources are written to stderr as they occur (before any task output)

---

## Skill File Integration

The intended usage pattern for agent skill files:

```markdown
## Finding Your Task

Run `task-marshal next --role <your_role>` to receive your task.
Work the task. When acceptance criteria are met, run `task-marshal done <id>`.
Do not read task source files directly.
```

No additional instructions are needed. task-marshal encapsulates all source knowledge.

---

## Local Task File Format

The local task file (`.llm/tasks.md` by default) uses markdown with structured task blocks.

**Task block structure:**

```markdown
## TASK-042: Short title

**Priority:** P1
**Role:** coder
**State:** unstarted
**Depends:** TASK-040, TASK-041

### Description

Full description of the work to be done.

### Acceptance Criteria

- [ ] Criterion one
- [ ] Criterion two
```

**Field rules:**
- `## <NativeId>: <title>` — heading defines the task block boundary and ID
- `**Priority:**` — `P0`, `P1`, `P2`, etc. (default `P2` if absent)
- `**Role:**` — optional free-form string
- `**State:**` — one of `unstarted`, `in_progress`, `blocked`, `done`
- `**Depends:**` — optional comma-separated list of NativeIds (must be in the same file)
- `### Description` — required subsection
- `### Acceptance Criteria` — required subsection

**State update by `done`:** The `**State:**` line is updated in-place from its current value to `done`.

**Comment annotation:** When `--comment` is provided, a `**Completion Note:**` line is appended after the state line.

---

## v1 Limitations

| Limitation | Notes |
|---|---|
| No subprocess timeout | If `gh` or `bd` hangs, task-marshal hangs. Disable affected source if network is unavailable. |
| No concurrent write safety | Last-writer-wins on local file. Avoid concurrent `done` on the same file. |
| No GitHub dependency tracking | All GitHub issues are treated as unblocked. |
| No cross-source dependencies | Dependencies must be within the same source. |
| No `--config` override | Config is always discovered by walk-up. |
| No task creation | task-marshal reads and completes tasks; it does not create them. |
