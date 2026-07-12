use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("permission denied: session {session_id} lacks capability '{capability}'")]
    PermissionDenied { session_id: Uuid, capability: String },

    #[error("policy '{0}' not found")]
    PolicyNotFound(String),

    #[error("policy violation: {0}")]
    PolicyViolation(String),
}

pub type SecurityResult<T> = Result<T, SecurityError>;
