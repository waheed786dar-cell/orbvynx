use crate::error::{WorkflowError, WorkflowResult};
use crate::graph::TaskGraph;
use orbvynx_kernel::Identity;
use orbvynx_planner::Plan;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    Created,
    Validated,
    Queued,
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed,
}

impl std::fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl WorkflowState {
    pub fn can_transition_to(&self, next: WorkflowState) -> bool {
        use WorkflowState::*;
        matches!(
            (self, next),
            (Created, Validated) | (Validated, Queued) | (Queued, Running)
                | (Running, Paused) | (Paused, Running) | (Running, Completed)
                | (Running, Failed) | (Queued, Cancelled) | (Running, Cancelled)
        )
    }
}

pub struct Workflow {
    pub identity: Identity,
    pub plan_id: Uuid,
    pub state: WorkflowState,
    pub graph: TaskGraph,
}

impl Workflow {
    pub fn from_plan(plan: &Plan, graph: TaskGraph) -> Self {
        Self {
            identity: Identity::new(),
            plan_id: plan.id(),
            state: WorkflowState::Created,
            graph,
        }
    }

    pub fn id(&self) -> Uuid {
        self.identity.id.0
    }

    pub fn transition(&mut self, next: WorkflowState) -> WorkflowResult<()> {
        if !self.state.can_transition_to(next) {
            return Err(WorkflowError::InvalidTransition {
                workflow_id: self.id(),
                from: self.state.to_string(),
                to: next.to_string(),
            });
        }
        self.state = next;
        Ok(())
    }

    pub fn validate(&mut self) -> WorkflowResult<()> {
        self.graph.validate(self.id())?;
        self.transition(WorkflowState::Validated)
    }
}
