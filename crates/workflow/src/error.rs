use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("workflow {0} has a circular dependency involving task '{1}'")]
    CircularDependency(Uuid, String),

    #[error("workflow {workflow_id} references unknown task '{task}'")]
    UnknownTask { workflow_id: Uuid, task: String },

    #[error("workflow {workflow_id} invalid transition: {from} -> {to}")]
    InvalidTransition { workflow_id: Uuid, from: String, to: String },

    #[error("task '{0}' exceeded max retries ({1})")]
    MaxRetriesExceeded(String, u32),

    #[error(transparent)]
    Kernel(#[from] orbvynx_kernel::KernelError),
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;
