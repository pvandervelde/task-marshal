# Security

---

## Threat Model Summary

task-marshal is a local development tool. It runs with the same privileges as the user. It does not operate as a daemon, does not listen on any port, and does not handle authentication. Its security surface is limited to:

1. File system access (read config, read/write task file)
2. Subprocess execution (`bd`, `gh`)
3. Config and task file parsing
4. User-provided input (task IDs, comment text)

---

## Threats and Mitigations

### T1: Command Injection via Task ID or Comment Text

**Threat:** A malicious or malformed `TaskId` or `--comment` value could be used to inject shell commands if it is interpolated into a shell string before being passed to `bd` or `gh`.

**Mitigation:**
- All subprocess calls use `std::process::Command` with arguments supplied as **separate process arguments**, never via `sh -c "..."` or any other shell interpolation
- User-provided values (`<id>`, `--comment <text>`) are passed directly as argument values, not as part of a command string
- See [constraints.md](constraints.md) — "Subprocess Safety" rule

**Residual risk:** None, if the no-shell-interpolation rule is enforced in code review.

---

### T2: Path Traversal via Config `path` Fields

**Threat:** A malicious `.llm/task-marshal.toml` could set `sources.local.path = "../../etc/passwd"` to cause task-marshal to read or write arbitrary files.

**Mitigation:**
- File paths read from config must be resolved to canonical absolute paths using the config file's directory as the base
- The resolved path must be validated to remain within the project directory (beneath the directory containing the config file, or a configurable root)
- Symlink following must be handled carefully: use `std::fs::canonicalize()` and check that the result is a descendant of the expected base

**Residual risk:** Low — the config file is authored by the developer in their own project; this is defence-in-depth.

---

### T3: Credential Exposure via Config

**Threat:** Developers may accidentally add credentials (GitHub tokens, API keys) to `.llm/task-marshal.toml`, which may be committed to version control.

**Mitigation:**
- The config schema contains **no credential fields**
- Credentials for GitHub are managed by `gh` CLI; credentials for BEADS are managed by `bd` CLI
- Documentation and config schema comments explicitly state no credentials belong here
- A `.gitignore` entry for `.llm/task-marshal.toml` is **not** recommended (config is intended to be committed); the absence of credential fields removes the risk

---

### T4: TOML Injection / Denial of Service via Large Config

**Threat:** An adversarial or malformed config file could cause excessive memory use or CPU consumption during parsing.

**Mitigation:**
- Use the `toml` crate's standard parser; do not implement a custom TOML parser
- The config schema is small and bounded; reject configs with unexpected top-level keys
- No need for explicit size limits beyond what the OS enforces (config files are always small)

**Residual risk:** Negligible.

---

### T5: Subprocess Output Injection into Stdout

**Threat:** A malicious `bd` or `gh` binary (or one that has been tampered with) could produce output designed to confuse task-marshal's parsing or inject content into the agent's context window.

**Mitigation:**
- task-marshal only writes to stdout what it has parsed and formatted itself
- Raw subprocess stdout is **never** forwarded directly to task-marshal's stdout
- Subprocess stderr is either forwarded to task-marshal's stderr (for user diagnostics) or discarded — never forwarded to stdout
- JSON output from subprocesses is deserialized into typed structs; unrecognised fields are ignored (using `#[serde(deny_unknown_fields)]` selectively or using `flatten` patterns carefully)

**Residual risk:** Low — the attack requires compromising `bd` or `gh` in the user's PATH.

---

### T6: Sensitive Data in Task Content

**Threat:** Task descriptions or acceptance criteria may inadvertently contain secrets (tokens, passwords) that are then written to agent context windows.

**Mitigation:**
- task-marshal faithfully reproduces task content; it does not redact or inspect task content
- This is a content authoring responsibility, not a tool responsibility
- Documentation should note that task content will be sent to agent context windows

---

### T7: Atomic Write Race Condition on Local Task File

**Threat:** Two concurrent `task-marshal done` invocations for the same local file could result in one update being silently overwritten.

**Mitigation:**
- The atomic rename strategy (`write to temp → rename`) reduces the window for corruption to near-zero
- Last-writer-wins is the v1 behaviour (the final `rename()` wins)
- No file locking is implemented in v1; this is an acceptable tradeoff for a local dev tool where concurrent invocations are rare
- The conflict scenario is noted in [edge-cases.md](edge-cases.md)

**Residual risk:** Low for typical use cases (single developer, single agent session at a time).

---

## OWASP Applicability

task-marshal is a CLI tool, not a web application. The following OWASP Top 10 items are relevant:

| Item | Relevance | Status |
|---|---|---|
| A01 Broken Access Control | N/A — no multi-user access model | — |
| A02 Cryptographic Failures | N/A — no encryption or credential storage | — |
| A03 Injection | **High** — subprocess argument injection | Mitigated (T1) |
| A04 Insecure Design | **Medium** — path traversal in config | Mitigated (T2) |
| A05 Security Misconfiguration | **Medium** — credential exposure in config | Mitigated (T3) |
| A06 Vulnerable Components | Low — minimal dependency footprint | Managed via `cargo audit` |
| A08 Software Integrity Failures | Low — binary distributed without signing (v1) | Noted for v2 |
| A09 Logging Failures | Low — no sensitive data in logs | N/A |
