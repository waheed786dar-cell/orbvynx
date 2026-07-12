use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("plugin executable not found at '{0}'")]
    NotFound(String),

    #[error("failed to spawn plugin '{0}': {1}")]
    SpawnFailed(String, String),

    #[error("plugin '{0}' manifest is invalid: {1}")]
    InvalidManifest(String, String),

    #[error("plugin '{0}' exited with error: {1}")]
    ExecutionFailed(String, String),

    #[error("plugin '{0}' produced invalid output: {1}")]
    InvalidOutput(String, String),
}

pub type PluginResult<T> = Result<T, PluginError>;
