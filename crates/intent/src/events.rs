//! Intent Engine events (Architecture Bible, Part 3 §12).
//!
//! The Intent crate has no kernel-blessed `EventKind` variants of
//! its own — per the dependency rule (Kernel never knows about
//! higher-level concepts), Intent events are published through the
//! Kernel's `EventKind::External` escape hatch with a namespaced
//! topic, so any subscriber (CLI, GUI, logging, Planner) can filter
//! on `topic.starts_with("intent.")` without the Kernel needing to
//! know Intent even exists.

use crate::model::Intent;
use orbvynx_kernel::{Event, EventBus, EventKind};
use serde_json::json;

/// Namespaced topic constants, kept in one place so subscribers and
/// publishers never drift out of sync on the exact string used.
pub mod topics {
    pub const CREATED: &str = "intent.created";
    pub const VALIDATED: &str = "intent.validated";
    pub const REJECTED: &str = "intent.rejected";
    pub const NORMALIZED: &str = "intent.normalized";
    pub const CLASSIFIED: &str = "intent.classified";
    pub const QUEUED: &str = "intent.queued";
    pub const PLANNING_STARTED: &str = "intent.planning_started";
    pub const PLANNING_FINISHED: &str = "intent.planning_finished";
}

/// Publishes a standard Intent lifecycle event onto the shared
/// Kernel event bus, carrying the Intent's ID and current state as
/// the JSON payload (Part 3 §12/§13 — every step must be observable
/// and eventually replayable from its recorded history).
pub fn publish(bus: &EventBus, topic: &str, intent: &Intent) {
    let payload = json!({
        "intent_id": intent.id(),
        "session_id": intent.session_id,
        "state": intent.state.to_string(),
        "category": format!("{:?}", intent.category),
        "goal": intent.effective_goal(),
    });

    bus.publish(
        Event::new(
            "intent-engine",
            EventKind::External {
                topic: topic.to_string(),
                payload,
            },
        )
        .with_session(intent.session_id),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Intent, IntentSource};
    use uuid::Uuid;

    #[tokio::test]
    async fn publish_emits_external_event_with_correct_topic() {
        let bus = EventBus::default();
        let mut sub = bus.subscribe();

        let intent = Intent::new("Build my app", IntentSource::Cli, Uuid::new_v4());
        publish(&bus, topics::CREATED, &intent);

        let event = sub.recv().await.unwrap();
        match event.kind {
            EventKind::External { topic, .. } => assert_eq!(topic, topics::CREATED),
            other => panic!("expected External event, got {other:?}"),
        }
    }
}
