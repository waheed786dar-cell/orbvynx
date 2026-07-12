//! Kernel-wide error types.
//!
//! Har kernel subsystem apna specific error variant yahan define
//! karta hai. Ye single `KernelError` enum poore kernel ke liye
//! canonical error type hai — Executor, Planner wagera apne errors
//! isse wrap ya convert kar sakte hain.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum KernelError {
    #[error("kernel is not in a valid state for this operation: expected {expected}, found {found}")]
    InvalidState { expected: String, found: String },

    #[error("module '{0}' is already registered")]
    ModuleAlreadyRegistered(String),

    #[error("module '{0}' not found in registry")]
    ModuleNotFound(String),

    #[error("service '{0}' not found in service registry")]
    ServiceNotFound(String),

    #[error("event bus channel closed unexpectedly")]
    EventBusClosed,

    #[error("event bus lagged, {0} events were dropped by a slow subscriber")]
    EventBusLagged(u64),

    #[error("permission denied for capability '{capability}' (session {session_id})")]
    PermissionDenied {
        capability: String,
        session_id: Uuid,
    },

    #[error("boot sequence failed at stage '{stage}': {reason}")]
    BootFailed { stage: String, reason: String },

    #[error("shutdown sequence failed at stage '{stage}': {reason}")]
    ShutdownFailed { stage: String, reason: String },

    #[error("invalid lifecycle transition: {from} -> {to} is not allowed")]
    InvalidLifecycleTransition { from: String, to: String },

    #[error("configuration error: {0}")]
    Config(String),

    #[error("timeout after {0:?}")]
    Timeout(std::time::Duration),

    #[error("internal kernel error: {0}")]
    Internal(String),
}

pub type KernelResult<T> = Result<T, KernelError>;
