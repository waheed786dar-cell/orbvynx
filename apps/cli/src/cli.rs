use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orbvynx", version, about = "ORBVYNX — Intent Operating Layer. Turns goals into deterministic, observable execution.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Boot the kernel and print status (event bus, registries, uptime)
    Status,

    /// Run an arbitrary goal through Intent -> Plan -> Workflow -> Executor
    Run {
        /// The goal, e.g. "build my app" or "commit changes"
        goal: Vec<String>,
    },

    /// Git operations via the built-in git capabilities
    Git {
        #[command(subcommand)]
        action: GitAction,
    },

    /// Manage and inspect dynamically-loaded plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },

    /// Compute a SHA-256 hash of text or a file
    Hash {
        /// Raw text to hash (mutually exclusive with --file)
        text: Option<String>,
        /// Path to a file to hash instead of raw text
        #[arg(short, long)]
        file: Option<String>,
    },

    /// Make HTTP requests via the built-in http capabilities
    Http {
        #[command(subcommand)]
        action: HttpAction,
    },
}

#[derive(Subcommand)]
pub enum GitAction {
    /// Show working tree status
    Status,
    /// Stage all changes and commit
    Commit {
        #[arg(short, long, default_value = "ORBVYNX automated commit")]
        message: String,
    },
    /// Push to the configured remote
    Push,
}

#[derive(Subcommand)]
pub enum PluginAction {
    /// List all discovered plugins and their capability names
    List,
}

#[derive(Subcommand)]
pub enum HttpAction {
    /// Perform an HTTP GET request
    Get {
        url: String,
    },
}
