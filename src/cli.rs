//! CLI argument definitions — clap only, no business logic
//!
//! This CLI is designed to be AI-agent friendly with intuitive commands.

use clap::{Command, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "windcli",
    about = "A safe file workspace CLI for AI agents",
    version,
    after_help = "Examples:
  windcli init ~/workspace
  echo 'hello' | windcli write notes/todo.md
  windcli read notes/todo.md
  windcli list notes
  windcli mkdir docs
  windcli delete notes/todo.md"
)]
pub struct Cli {
    /// Enable JSON output (machine-readable)
    #[arg(long, short)]
    pub json: bool,

    /// Show verbose/debug output
    #[arg(long, short)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Create or switch to a workspace directory
    Init {
        /// Workspace path (default: current directory)
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
    },

    /// List files in workspace
    Ls {
        /// Path to list (default: workspace root)
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
    },

    /// Read file content (max 10MB)
    Read {
        /// File path to read
        path: std::path::PathBuf,
    },

    /// Write content to a file (use --stdin or --content)
    Write {
        /// Destination file path
        path: std::path::PathBuf,

        /// Read content from stdin
        #[arg(long, short = 's')]
        stdin: bool,

        /// Content to write directly (alternative to --stdin)
        #[arg(long, short = 'c')]
        content: Option<String>,
    },

    /// Create a directory
    Mkdir {
        /// Directory path to create
        path: std::path::PathBuf,
    },

    /// Delete a file or directory
    Delete {
        /// Path to delete
        path: std::path::PathBuf,

        /// Delete directory and all contents
        #[arg(long, short = 'r')]
        recursive: bool,

        /// Skip confirmation
        #[arg(long, short = 'y')]
        yes: bool,

        /// Show what would be deleted without deleting
        #[arg(long)]
        dry_run: bool,
    },

    /// Check for updates
    Upgrade {
        /// Only check, do not download
        #[arg(long)]
        check: bool,
    },

    /// Show version information
    Version,
}

// Aliases for backward compatibility
pub fn build() -> Cli {
    let args: Vec<String> = std::env::args().collect();
    let args = normalize_args(&args);
    Cli::parse_from(args)
}

/// Normalize AI-friendly command aliases
fn normalize_args(args: &[String]) -> Vec<String> {
    let mut result = vec![args[0].clone()];
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            // Alias: cat -> read
            "cat" => result.push("read".to_string()),
            // Alias: put -> write
            "put" => result.push("write".to_string()),
            // Alias: rm -> delete
            "rm" => result.push("delete".to_string()),
            // Keep other args as-is
            arg => result.push(arg.to_string()),
        }
        i += 1;
    }

    result
}
