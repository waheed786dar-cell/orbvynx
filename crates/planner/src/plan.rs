//! Plan data model (Architecture Bible, Part 4 §13, §17).
//!
//! A Plan is an execution-ready blueprint — the Planner's sole
//! output. "Planner creates plans. It never executes them."
//! The Workflow Engine consumes a `Plan` and turns it into a task
//! graph; the Planner itself never touches resources.

use crate::scoring::{CostEstimate, RiskScore};
use chrono::{DateTime, Utc};
use orbvynx_kernel::Identity;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single abstract step within a Plan. This is deliberately
/// coarse-grained (not yet a Workflow "Task") — the Workflow Engine
/// is responsible for expanding each `PlanStep` into one or more
/// concrete tasks with dependencies (Part 5 §1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub order: u32,
    pub description: String,
    pub required_capability: String,
}

/// Validation status recorded on a Plan (Part 4 §13's
/// "Validation Status" field, further elaborated by the Plan
/// Validator recommended in Part 4 §20).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanValidationStatus {
    NotValidated,
    Valid,
    Invalid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub identity: Identity,
    pub intent_id: Uuid,
    pub goal: String,
    pub required_capabilities: Vec<String>,
    pub required_resources: Vec<String>,
    pub steps: Vec<PlanStep>,
    pub estimated_cost: CostEstimate,
    pub risk_score: RiskScore,
    pub validation_status: PlanValidationStatus,
    pub created_at: DateTime<Utc>,
}

impl Plan {
    pub fn id(&self) -> Uuid {
        self.identity.id.0
    }
}

/// Builder for constructing a `Plan` incrementally as the pipeline
/// stages run (Context -> Resources -> Capabilities -> Policy ->
/// Risk -> Cost -> Generation).
pub struct PlanBuilder {
    intent_id: Uuid,
    goal: String,
    required_capabilities: Vec<String>,
    required_resources: Vec<String>,
    steps: Vec<PlanStep>,
}

impl PlanBuilder {
    pub fn new(intent_id: Uuid, goal: impl Into<String>) -> Self {
        Self {
            intent_id,
            goal: goal.into(),
            required_capabilities: Vec::new(),
            required_resources: Vec::new(),
            steps: Vec::new(),
        }
    }

    pub fn require_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.push(capability.into());
        self
    }

    pub fn require_resource(mut self, resource: impl Into<String>) -> Self {
        self.required_resources.push(resource.into());
        self
    }

    pub fn add_step(mut self, description: impl Into<String>, required_capability: impl Into<String>) -> Self {
        let order = self.steps.len() as u32 + 1;
        self.steps.push(PlanStep {
            order,
            description: description.into(),
            required_capability: required_capability.into(),
        });
        self
    }

    pub fn finish(self, cost: CostEstimate, risk: RiskScore) -> Plan {
        Plan {
            identity: Identity::new(),
            intent_id: self.intent_id,
            goal: self.goal,
            required_capabilities: self.required_capabilities,
            required_resources: self.required_resources,
            steps: self.steps,
            estimated_cost: cost,
            risk_score: risk,
            validation_status: PlanValidationStatus::NotValidated,
            created_at: Utc::now(),
        }
    }
}
