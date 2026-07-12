//! Intent Engine error types (Architecture Bible, Part 3 §7).
//!
//! Validation failures are the primary error surface here — an
//! Intent that fails validation must never reach the Planner
//! (Part 3 §11, "Intent -> Plan Boundary").

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum IntentError {
    #[error("intent {0} has an empty goal")]
    EmptyGoal(Uuid),

    #[error("intent {intent_id} failed validation: {reason}")]
    ValidationFailed { intent_id: Uuid, reason: String },

    #[error("intent {0} is ambiguous and needs clarification: {1}")]
    NeedsClarification(Uuid, String),

    #[error("intent {intent_id} has an invalid state transition: {from} -> {to}")]
    InvalidTransition {
        intent_id: Uuid,
        from: String,
        to: String,
    },

    #[error("intent {0} was rejected: {1}")]
    Rejected(Uuid, String),

    #[error("unknown intent category: '{0}'")]
    UnknownCategory(String),

    #[error(transparent)]
    Kernel(#[from] orbvynx_kernel::KernelError),
}

pub type IntentResult<T> = Result<T, IntentError>;
