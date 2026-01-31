use clap::{Parser, Subcommand};
use gitnu::commands::*;
use colored::Colorize;

#[derive(Parser)]
#[command(name = "gnu")]
#[command(about = "gitnu - Version-controlled knowledge operating system for AI agents", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new gitnu vault
    Init {
        /// Name of the vault/project
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Show current context state
    Status,

    /// Create a checkpoint of current context
    Commit {
        /// Commit message
        message: String,

        /// Author type: human or agent
        #[arg(long, default_value = "agent")]
        author: String,

        /// Model name (for agent commits)
        #[arg(long)]
        model: Option<String>,
    },

    /// Show commit history
    Log {
        /// Show one line per commit
        #[arg(long)]
        oneline: bool,

        /// Limit number of commits shown
        #[arg(short, long)]
        limit: Option<usize>,

        /// Show log for specific branch
        #[arg(short, long)]
        branch: Option<String>,
    },

    /// Manage branches
    Branch {
        /// Branch name (creates new branch)
        name: Option<String>,

        /// Delete branch
        #[arg(short = 'd', long)]
        delete: Option<String>,

        /// Description for new branch
        #[arg(long)]
        describe: Option<String>,
    },

    /// Switch branches or restore commits
    Checkout {
        /// Branch name or commit hash
        target: String,

        /// Force checkout, discarding uncommitted changes
        #[arg(short, long)]
        force: bool,
    },

    /// Roll back to a previous commit
    Rewind {
        /// Commit reference (hash or HEAD~N)
        target: String,

        /// Keep working directory unchanged
        #[arg(long)]
        soft: bool,
    },

    /// Show changes between commits or branches
    Diff {
        /// Source commit/branch
        source: Option<String>,

        /// Target commit/branch
        target: Option<String>,
    },

    /// Merge learnings from one branch into another
    Merge {
        /// Source branch to merge from
        source: String,

        /// Target branch to merge into (default: current)
        #[arg(long)]
        into: Option<String>,

        /// Squash all commits into one
        #[arg(long)]
        squash: bool,
    },

    /// Load domains/files into active context
    Load {
        /// Path or wikilink to load
        path: Option<String>,

        /// Pin this file (always include)
        #[arg(short, long)]
        pin: bool,

        /// List currently loaded files
        #[arg(short, long)]
        list: bool,
    },

    /// Remove domains/files from active context
    Unload {
        /// Path or wikilink to unload
        path: Option<String>,

        /// Unload all non-pinned files
        #[arg(long)]
        all: bool,
    },

    /// Pin files to always include in context
    Pin {
        /// Path or wikilink to pin
        path: String,

        /// Exclude this file (blacklist)
        #[arg(long)]
        exclude: bool,
    },

    /// Unpin files
    Unpin {
        /// Path or wikilink to unpin
        path: String,
    },

    /// Resolve wikilink to full path
    Resolve {
        /// Wikilink to resolve (e.g., [[spec]])
        wikilink: String,
    },

    /// Output current active context
    Context {
        /// Copy to clipboard
        #[arg(long)]
        clipboard: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Apply markdown compression
        #[arg(long)]
        compress: bool,
    },

    /// Generate summary of current context state
    Summary,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name } => init(name),
        Commands::Status => status(),
        Commands::Commit { message, author, model } => commit(&message, &author, model),
        Commands::Log { oneline, limit, branch } => log(oneline, limit, branch),
        Commands::Branch { name, delete, describe } => {
            if let Some(branch_name) = delete {
                branch_delete(&branch_name)
            } else if let Some(branch_name) = name {
                branch_create(&branch_name, describe)
            } else {
                branch_list()
            }
        }
        Commands::Checkout { target, force } => checkout(&target, force),
        Commands::Rewind { target, soft } => rewind(&target, soft),
        Commands::Diff { source, target } => diff(source, target),
        Commands::Merge { source, into, squash } => merge(&source, into, squash),
        Commands::Load { path, pin, list } => {
            if list {
                load("", false, true)
            } else if let Some(p) = path {
                load(&p, pin, false)
            } else {
                Err(gitnu::GitnuError::Other(
                    "Must specify path or use --list".to_string(),
                ))
            }
        }
        Commands::Unload { path, all } => unload(path, all),
        Commands::Pin { path, exclude } => pin(&path, exclude),
        Commands::Unpin { path } => unpin(&path),
        Commands::Resolve { wikilink } => resolve(&wikilink),
        Commands::Context { clipboard, json, compress } => context(clipboard, json, compress),
        Commands::Summary => summary(),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}
