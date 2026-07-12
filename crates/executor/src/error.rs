use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExecutorError {
    #[error("capability '{0}' not registered")]
    CapabilityNotFound(String),

    #[error("permission denied for capability '{0}'")]
    PermissionDenied(String),

    #[error("task '{task_id}' failed: {reason}")]
    TaskFailed { task_id: String, reason: String },

    #[error("task '{0}' timed out after {1}s")]
    TaskTimedOut(String, u64),

    #[error("task '{0}' was cancelled")]
    TaskCancelled(String),

    #[error(transparent)]
    Workflow(#[from] orbvynx_workflow::WorkflowError),

    #[error(transparent)]
    Kernel(#[from] orbvynx_kernel::KernelError),
}

pub type ExecutorResult<T> = Result<T, ExecutorError>;
