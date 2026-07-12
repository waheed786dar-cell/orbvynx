use crate::capability::{CapabilityInput, CapabilityRegistry};
use crate::error::{ExecutorError, ExecutorResult};
use crate::result::{ResourceUsage, TaskOutcome, TaskResult};
use chrono::Utc;
use orbvynx_kernel::{Event, EventBus, EventKind};
use orbvynx_workflow::Task;
use serde_json::json;
use std::collections::HashMap;
use std::time::Instant;
use tokio::time::{timeout, Duration};

pub struct Executor {
    pub registry: CapabilityRegistry,
    pub event_bus: EventBus,
}

impl Executor {
    pub fn new(registry: CapabilityRegistry, event_bus: EventBus) -> Self {
        Self { registry, event_bus }
    }

    pub async fn execute(&self, task: &Task, params: HashMap<String, serde_json::Value>) -> ExecutorResult<TaskResult> {
        let capability = self.registry.get(&task.required_capability)
            .ok_or_else(|| ExecutorError::CapabilityNotFound(task.required_capability.clone()))?;

        self.publish("executor.task_started", &task.id);

        let start = Instant::now();
        let input = CapabilityInput { params };
        let max_duration = Duration::from_secs(task.timeout_policy.max_seconds);

        let invocation = timeout(max_duration, capability.invoke(input)).await;

        let usage = ResourceUsage {
            cpu_seconds: start.elapsed().as_secs_f64(),
            memory_peak_mb: 0.0,
            duration_ms: start.elapsed().as_millis(),
        };

        let result = match invocation {
            Ok(Ok(output)) => {
                self.publish("executor.task_completed", &task.id);
                TaskResult {
                    task_id: task.id.clone(),
                    outcome: TaskOutcome::Success,
                    output: Some(output.data),
                    error: None,
                    usage,
                    completed_at: Utc::now(),
                }
            }
            Ok(Err(reason)) => {
                self.publish("executor.task_failed", &task.id);
                TaskResult {
                    task_id: task.id.clone(),
                    outcome: TaskOutcome::Failed,
                    output: None,
                    error: Some(reason),
                    usage,
                    completed_at: Utc::now(),
                }
            }
            Err(_) => {
                self.publish("executor.task_timed_out", &task.id);
                TaskResult {
                    task_id: task.id.clone(),
                    outcome: TaskOutcome::TimedOut,
                    output: None,
                    error: Some(format!("timed out after {}s", task.timeout_policy.max_seconds)),
                    usage,
                    completed_at: Utc::now(),
                }
            }
        };

        Ok(result)
    }

    fn publish(&self, topic: &str, task_id: &str) {
        self.event_bus.publish(Event::new("executor", EventKind::External {
            topic: topic.to_string(),
            payload: json!({ "task_id": task_id }),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::{EchoCapability, FailingCapability};
    use std::sync::Arc;

    #[tokio::test]
    async fn successful_task_returns_success_outcome() {
        let mut registry = CapabilityRegistry::new();
        registry.register(Arc::new(EchoCapability));
        let executor = Executor::new(registry, EventBus::default());

        let task = Task::new("t1", "echo test", "test.echo");
        let result = executor.execute(&task, HashMap::new()).await.unwrap();
        assert_eq!(result.outcome, TaskOutcome::Success);
    }

    #[tokio::test]
    async fn failing_capability_returns_failed_outcome() {
        let mut registry = CapabilityRegistry::new();
        registry.register(Arc::new(FailingCapability));
        let executor = Executor::new(registry, EventBus::default());

        let task = Task::new("t2", "fail test", "test.fail");
        let result = executor.execute(&task, HashMap::new()).await.unwrap();
        assert_eq!(result.outcome, TaskOutcome::Failed);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn missing_capability_returns_error() {
        let registry = CapabilityRegistry::new();
        let executor = Executor::new(registry, EventBus::default());

        let task = Task::new("t3", "missing test", "test.missing");
        assert!(executor.execute(&task, HashMap::new()).await.is_err());
    }
}
