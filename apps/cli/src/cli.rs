use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "orbvynx", version, about = "ORBVYNX — Intent Operating Layer. Turns goals into deterministic, observable execution.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Boot the kernel and print status
    Status,

    /// Run an arbitrary goal through Intent -> Plan -> Workflow -> Executor
    Run { goal: Vec<String> },

    /// Git operations
    Git {
        #[command(subcommand)]
        action: GitAction,
    },

    /// Manage dynamically-loaded plugins
    Plugin {
        #[command(subcommand)]
        action: PluginAction,
    },

    /// Compute a SHA-256 hash of text or a file
    Hash {
        text: Option<String>,
        #[arg(short, long)]
        file: Option<String>,
    },

    /// HTTP requests
    Http {
        #[command(subcommand)]
        action: HttpAction,
    },

    /// Encode/decode text
    Encode {
        #[command(subcommand)]
        action: EncodeAction,
    },

    /// Filesystem inspection
    Fs {
        #[command(subcommand)]
        action: FsAction,
    },

    /// JSON inspection
    Json {
        #[command(subcommand)]
        action: JsonAction,
    },

    /// System utilities (UUID, current time, env vars)
    Sys {
        #[command(subcommand)]
        action: SysAction,
    },
}

#[derive(Subcommand)]
pub enum GitAction {
    Status,
    Commit {
        #[arg(short, long, default_value = "ORBVYNX automated commit")]
        message: String,
    },
    Push,
}

#[derive(Subcommand)]
pub enum PluginAction {
    List,
}

#[derive(Subcommand)]
pub enum HttpAction {
    Get { url: String },
}

#[derive(Subcommand)]
pub enum EncodeAction {
    /// Base64-encode text
    Base64 { text: String },
    /// Base64-decode text
    Base64Decode { text: String },
    /// URL-encode text
    Url { text: String },
}

#[derive(Subcommand)]
pub enum FsAction {
    /// List directory contents
    List {
        path: String,
        #[arg(short, long, default_value_t = 1)]
        depth: u64,
    },
    /// Check if a path exists
    Exists { path: String },
}

#[derive(Subcommand)]
pub enum JsonAction {
    /// Validate and pretty-print JSON text
    Parse { text: String },
    /// Query a dotted path in a JSON file's contents
    Query {
        text: String,
        path: String,
    },
}

#[derive(Subcommand)]
pub enum SysAction {
    /// Generate a random UUID v4
    Uuid,
    /// Print the current time (ISO 8601 + Unix seconds)
    Time,
    /// Read an environment variable
    Env { key: String },
}
