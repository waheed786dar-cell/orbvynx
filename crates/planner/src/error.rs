//! Planner Engine error types (Architecture Bible, Part 4 §16).
//!
//! "Planner fail hone par execution kabhi start nahi hogi." Every
//! failure case listed in Part 4 §16 has a matching variant here so
//! callers can distinguish *why* planning failed, not just *that*
//! it failed.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum PlannerError {
    #[error("planning failed for intent {intent_id}: no capability available for '{needed}'")]
    NoCapabilityAvailable { intent_id: Uuid, needed: String },

    #[error("planning failed for intent {intent_id}: policy conflict: {reason}")]
    PolicyConflict { intent_id: Uuid, reason: String },

    #[error("planning failed for intent {intent_id}: missing resource '{resource}'")]
    MissingResource { intent_id: Uuid, resource: String },

    #[error("planning failed for intent {intent_id}: goal is ambiguous: {reason}")]
    AmbiguousIntent { intent_id: Uuid, reason: String },

    #[error("no viable plan could be generated for intent {0}")]
    NoViablePlan(Uuid),

    #[error(transparent)]
    Intent(#[from] orbvynx_intent::IntentError),

    #[error(transparent)]
    Kernel(#[from] orbvynx_kernel::KernelError),
}

pub type PlannerResult<T> = Result<T, PlannerError>;
