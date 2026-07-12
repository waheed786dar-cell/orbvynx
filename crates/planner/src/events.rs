use crate::plan::Plan;
use orbvynx_kernel::{Event, EventBus, EventKind};
use serde_json::json;

pub mod topics {
    pub const PLANNING_STARTED: &str = "planner.planning_started";
    pub const PLAN_GENERATED: &str = "planner.plan_generated";
    pub const PLAN_REJECTED: &str = "planner.plan_rejected";
}

pub fn publish_plan_generated(bus: &EventBus, plan: &Plan) {
    let payload = json!({
        "plan_id": plan.id(),
        "intent_id": plan.intent_id,
        "risk_score": plan.risk_score.0,
        "steps": plan.steps.len(),
    });
    bus.publish(Event::new("planner", EventKind::External {
        topic: topics::PLAN_GENERATED.to_string(),
        payload,
    }));
}
