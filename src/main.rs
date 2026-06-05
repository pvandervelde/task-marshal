// CLI entry point — command dispatcher and exit code mapping.
// See docs/spec/architecture.md — "CLI Layer" section.

use std::process;

use clap::{Parser, Subcommand};
use task_marshal::config::ConfigLoader;
use task_marshal::identity::SourceKey;
use task_marshal::identity::TaskIdParser;
use task_marshal::output::{CompletionError, OutputFormatter};
use task_marshal::selection::{ScoredCandidate, SelectionError, SelectionFilter, TaskSelector};
use task_marshal::sources::{SourceError, SourceRegistry};
use task_marshal::{Role, TaskSummary};

// ── CLI argument structure ────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "task-marshal",
    about = "A CLI broker for retrieving and completing tasks across multiple sources",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Select and display the next highest-priority eligible task.
    Next {
        /// Filter tasks by agent role (unassigned tasks always pass).
        #[arg(long)]
        role: Option<String>,

        /// Restrict to a single source (local, beads, gh).
        #[arg(long)]
        source: Option<String>,
    },

    /// Mark a task as done and propagate the change to its source.
    Done {
        /// The task ID to complete (e.g. local:TASK-042, gh:187).
        id: String,

        /// Optional completion annotation.
        #[arg(long)]
        comment: Option<String>,
    },

    /// Display the full content of a specific task.
    Show {
        /// The task ID to display (e.g. local:TASK-042, beads:bd-a1b2).
        id: String,
    },

    /// List tasks across all configured sources.
    List {
        /// Include done and blocked tasks.
        #[arg(long)]
        all: bool,

        /// Filter by agent role.
        #[arg(long)]
        role: Option<String>,

        /// Restrict to a single source.
        #[arg(long)]
        source: Option<String>,
    },
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    let exit_code = match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", OutputFormatter::format_error(&e.to_string()));
            2
        }
    };

    process::exit(exit_code);
}

// ── Command dispatch ──────────────────────────────────────────────────────────

/// Dispatches to the appropriate command handler and returns the exit code.
///
/// Exit codes:
///   0 — success (task found, operation completed)
///   1 — no task found (next command only)
///   2 — operational error
fn run(cli: Cli) -> Result<i32, Box<dyn std::error::Error>> {
    unimplemented!("See docs/spec/architecture.md — CLI Layer")
}
