//! Intent data model (Architecture Bible, Part 3 §4, §5, §6, §9, §10).
//!
//! "Intent != Command." An Intent captures WHAT the user wants,
//! never HOW to achieve it. The Planner crate is the only consumer
//! that is allowed to turn an Intent into an executable plan
//! (Part 3 §11, "Intent -> Plan Boundary").

use chrono::{DateTime, Utc};
use orbvynx_kernel::Identity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Where an Intent originated from (Part 3 §6).
/// The Intent Engine must remain UI-independent — it never assumes
/// a specific source shaped the goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentSource {
    Cli,
    Gui,
    RestApi,
    Sdk,
    Voice,
    AutomationTrigger,
    ScheduledTask,
    Plugin,
}

/// Intent categories (Part 3 §5). The Planner uses this to select
/// a suitable workflow family — the Intent Engine itself never
/// decides *how* a category will be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentCategory {
    Build,
    Analyze,
    Search,
    Generate,
    Deploy,
    Test,
    Refactor,
    Monitor,
    Optimize,
    Learn,
    Install,
    Configure,
    Diagnose,
    Automate,
    /// Category could not yet be determined (pre-classification).
    Unclassified,
}

/// Extra conditions the user attaches to a goal (Part 3 §10).
/// The Planner MUST NOT ignore these when generating a plan.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentConstraints {
    pub offline_only: bool,
    pub fastest_execution: bool,
    pub safe_mode: bool,
    pub dry_run: bool,
    pub no_network: bool,
    pub low_battery_mode: bool,
    /// Free-form constraints not covered by the fixed flags above,
    /// e.g. "max_cost_usd" -> "5.00".
    pub custom: HashMap<String, String>,
}

/// Environmental facts attached to an Intent (Part 3 §9).
/// The Intent Engine collects this via Kernel services; it never
/// interprets what the context *means* for execution — that is the
/// Planner's job.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IntentContext {
    pub working_directory: String,
    pub detected_project_types: Vec<String>,
    pub git_repository: bool,
    pub internet_available: Option<bool>,
    pub extra: HashMap<String, String>,
}

/// The fixed lifecycle every Intent moves through (Part 3 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentState {
    Created,
    Validated,
    Normalized,
    Classified,
    Planned,
    Approved,
    Executing,
    Completed,
    Rejected,
    PlanningFailed,
}

impl std::fmt::Display for IntentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// The Intent itself (Part 3 §4). `identity` supplies the universal
/// ID/version/timestamp block shared with every other ORBVYNX object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub identity: Identity,
    pub session_id: Uuid,

    /// The original, user-provided goal text. Immutable once created
    /// (Part 3 §14, "Intent immutable hona chahiye").
    pub original_goal: String,

    /// The normalized/standardized goal after Part 3 §8 normalization.
    /// `None` until the Normalization stage has run.
    pub normalized_goal: Option<String>,

    pub source: IntentSource,
    pub category: IntentCategory,
    pub constraints: IntentConstraints,
    pub context: IntentContext,
    pub state: IntentState,
    pub created_at: DateTime<Utc>,
    pub priority: u8,
}

impl Intent {
    /// Creates a brand-new Intent in the `Created` state. Priority
    /// defaults to 5 on a 0 (lowest) - 9 (highest) scale.
    pub fn new(goal: impl Into<String>, source: IntentSource, session_id: Uuid) -> Self {
        Self {
            identity: Identity::new(),
            session_id,
            original_goal: goal.into(),
            normalized_goal: None,
            source,
            category: IntentCategory::Unclassified,
            constraints: IntentConstraints::default(),
            context: IntentContext::default(),
            state: IntentState::Created,
            created_at: Utc::now(),
            priority: 5,
        }
    }

    pub fn id(&self) -> Uuid {
        self.identity.id.0
    }

    pub fn with_constraints(mut self, constraints: IntentConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_context(mut self, context: IntentContext) -> Self {
        self.context = context;
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(9);
        self
    }

    /// The goal that downstream engines (Planner) should read:
    /// normalized if available, otherwise the original text.
    pub fn effective_goal(&self) -> &str {
        self.normalized_goal.as_deref().unwrap_or(&self.original_goal)
    }
}
