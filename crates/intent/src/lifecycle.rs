//! Intent Lifecycle enforcement (Architecture Bible, Part 3 §3, §14).
//!
//! Wraps the fixed state machine every Intent must follow:
//!
//!   Created -> Validated -> Normalized -> Classified -> Planned
//!   -> Approved -> Executing -> Completed
//!
//! with early-exit branches to Rejected / PlanningFailed. Invalid
//! transitions are rejected outright — matching the Kernel's own
//! `LifecycleMachine` philosophy (no silent, ad-hoc jumps).

use crate::error::{IntentError, IntentResult};
use crate::model::{Intent, IntentState};

impl IntentState {
    /// The permanent, explicit transition table for Intents
    /// (Part 3 §3). Every arrow drawn in the spec's diagram is
    /// represented here, and only those arrows.
    pub fn can_transition_to(&self, next: IntentState) -> bool {
        use IntentState::*;
        matches!(
            (self, next),
            (Created, Validated)
                | (Created, Rejected)
                | (Validated, Normalized)
                | (Normalized, Classified)
                | (Classified, Planned)
                | (Classified, PlanningFailed)
                | (Planned, Approved)
                | (Planned, PlanningFailed)
                | (Approved, Executing)
                | (Executing, Completed)
        )
    }
}

/// Applies a validated state transition to an Intent, mutating it
/// in place. This is the *only* sanctioned way to change
/// `Intent::state` — callers should never assign `intent.state = X`
/// directly, so that invalid jumps are always caught.
pub fn transition(intent: &mut Intent, next: IntentState) -> IntentResult<()> {
    if !intent.state.can_transition_to(next) {
        return Err(IntentError::InvalidTransition {
            intent_id: intent.id(),
            from: intent.state.to_string(),
            to: next.to_string(),
        });
    }
    intent.state = next;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Intent, IntentSource};
    use uuid::Uuid;

    fn sample_intent() -> Intent {
        Intent::new("Build my app", IntentSource::Cli, Uuid::new_v4())
    }

    #[test]
    fn full_happy_path_succeeds() {
        let mut intent = sample_intent();
        transition(&mut intent, IntentState::Validated).unwrap();
        transition(&mut intent, IntentState::Normalized).unwrap();
        transition(&mut intent, IntentState::Classified).unwrap();
        transition(&mut intent, IntentState::Planned).unwrap();
        transition(&mut intent, IntentState::Approved).unwrap();
        transition(&mut intent, IntentState::Executing).unwrap();
        transition(&mut intent, IntentState::Completed).unwrap();
        assert_eq!(intent.state, IntentState::Completed);
    }

    #[test]
    fn skipping_a_stage_is_rejected() {
        let mut intent = sample_intent();
        // Created -> Classified directly must fail.
        assert!(transition(&mut intent, IntentState::Classified).is_err());
        assert_eq!(intent.state, IntentState::Created);
    }

    #[test]
    fn rejection_branch_from_created_works() {
        let mut intent = sample_intent();
        transition(&mut intent, IntentState::Rejected).unwrap();
        assert_eq!(intent.state, IntentState::Rejected);
    }

    #[test]
    fn planning_failed_branch_from_classified_works() {
        let mut intent = sample_intent();
        transition(&mut intent, IntentState::Validated).unwrap();
        transition(&mut intent, IntentState::Normalized).unwrap();
        transition(&mut intent, IntentState::Classified).unwrap();
        transition(&mut intent, IntentState::PlanningFailed).unwrap();
        assert_eq!(intent.state, IntentState::PlanningFailed);
    }
}
