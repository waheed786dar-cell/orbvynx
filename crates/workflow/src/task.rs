use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Created,
    Queued,
    Running,
    Completed,
    Failed,
    Retrying,
    Cancelled,
    TimedOut,
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_seconds: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self { max_attempts: 3, backoff_seconds: 2 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutPolicy {
    pub max_seconds: u64,
}

impl Default for TimeoutPolicy {
    fn default() -> Self {
        Self { max_seconds: 600 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub required_capability: String,
    pub depends_on: Vec<String>,
    pub state: TaskState,
    pub attempts: u32,
    pub retry_policy: RetryPolicy,
    pub timeout_policy: TimeoutPolicy,
}

impl Task {
    pub fn new(id: impl Into<String>, description: impl Into<String>, required_capability: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: description.into(),
            required_capability: required_capability.into(),
            depends_on: Vec::new(),
            state: TaskState::Created,
            attempts: 0,
            retry_policy: RetryPolicy::default(),
            timeout_policy: TimeoutPolicy::default(),
        }
    }

    pub fn depends_on(mut self, task_id: impl Into<String>) -> Self {
        self.depends_on.push(task_id.into());
        self
    }

    pub fn can_retry(&self) -> bool {
        self.attempts < self.retry_policy.max_attempts
    }
}
