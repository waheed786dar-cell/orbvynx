//! # orbvynx-intent
//!
//! The ORBVYNX Intent Engine (Architecture Bible, Part 3).
//!
//! > "Intent Engine kabhi bhi workflow generate nahi karega. Intent
//! > Engine ka kaam: Goal receive karo -> Validate karo -> Normalize
//! > karo -> Context attach karo -> Planner ko handover karo. Bas."
//!
//! This crate captures raw user goals and turns them into validated,
//! normalized, classified `Intent` objects — and nothing more. It
//! never plans, never executes, and never assumes a specific UI.
//!
//! - [`model`] — the `Intent` data structure and its supporting types
//! - [`validation`] — the fixed validation rule pipeline
//! - [`normalization`] — deterministic goal-text normalization
//! - [`lifecycle`] — the enforced Intent state machine
//! - [`events`] — Kernel event-bus integration for observability
//! - [`error`] — the canonical `IntentError` type

pub mod error;
pub mod events;
pub mod lifecycle;
pub mod model;
pub mod normalization;
pub mod validation;

pub use error::{IntentError, IntentResult};
pub use model::{
    Intent, IntentCategory, IntentConstraints, IntentContext, IntentSource, IntentState,
};
pub use normalization::{IntentNormalizer, NormalizationRule};
pub use validation::{IntentValidator, ValidationRule};

use orbvynx_kernel::EventBus;
use uuid::Uuid;

/// The Intent Engine: a thin orchestrator that wires validation,
/// normalization, and lifecycle transitions together, publishing an
/// event at each stage. This is the primary entry point other
/// crates (or the CLI) should use rather than calling the
/// individual stages by hand.
pub struct IntentEngine {
    validator: IntentValidator,
    normalizer: IntentNormalizer,
    event_bus: EventBus,
}

impl IntentEngine {
    pub fn new(event_bus: EventBus) -> Self {
        Self {
            validator: IntentValidator::standard(),
            normalizer: IntentNormalizer::standard(),
            event_bus,
        }
    }

    pub fn with_validator(mut self, validator: IntentValidator) -> Self {
        self.validator = validator;
        self
    }

    pub fn with_normalizer(mut self, normalizer: IntentNormalizer) -> Self {
        self.normalizer = normalizer;
        self
    }

    /// Runs an Intent through Created -> Validated -> Normalized,
    /// publishing an event at every successful stage. Stops and
    /// transitions the Intent to `Rejected` if validation fails
    /// (Part 3 §3, the "Created -> Rejected" branch).
    ///
    /// Classification, Planning, Approval and Execution are owned by
    /// the Planner/Workflow/Executor crates and are intentionally
    /// out of scope here.
    pub fn intake(&self, goal: impl Into<String>, source: IntentSource, session_id: Uuid) -> IntentResult<Intent> {
        let mut intent = Intent::new(goal, source, session_id);
        events::publish(&self.event_bus, events::topics::CREATED, &intent);

        if let Err(e) = self.validator.validate(&intent) {
            lifecycle::transition(&mut intent, IntentState::Rejected)?;
            events::publish(&self.event_bus, events::topics::REJECTED, &intent);
            return Err(e);
        }
        lifecycle::transition(&mut intent, IntentState::Validated)?;
        events::publish(&self.event_bus, events::topics::VALIDATED, &intent);

        if let Some(normalized) = self.normalizer.normalize(&intent.original_goal) {
            intent.normalized_goal = Some(normalized);
        }
        lifecycle::transition(&mut intent, IntentState::Normalized)?;
        events::publish(&self.event_bus, events::topics::NORMALIZED, &intent);

        Ok(intent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn intake_produces_normalized_intent() {
        let bus = EventBus::default();
        let mut sub = bus.subscribe();
        let engine = IntentEngine::new(bus);

        let intent = engine
            .intake("compile app for release", IntentSource::Cli, Uuid::new_v4())
            .unwrap();

        assert_eq!(intent.state, IntentState::Normalized);
        assert_eq!(
            intent.effective_goal(),
            "Build Android application"
        );

        // Drain the three expected lifecycle events without blocking
        // forever if fewer arrive than expected.
        for _ in 0..3 {
            assert!(sub.recv().await.is_ok());
        }
    }

    #[tokio::test]
    async fn intake_rejects_empty_goal() {
        let bus = EventBus::default();
        let engine = IntentEngine::new(bus);

        let result = engine.intake("   ", IntentSource::Cli, Uuid::new_v4());
        assert!(result.is_err());
    }
}
