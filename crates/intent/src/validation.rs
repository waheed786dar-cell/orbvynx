//! Intent Validation (Architecture Bible, Part 3 §7).
//!
//! "Validation fail hui to execution shuru hi nahi hogi." This
//! runs before Normalization and before the Intent is ever handed
//! to the Planner. Validation only checks structural/basic
//! soundness — it never evaluates policy or permissions in detail
//! (that happens later, in the Planner's Policy Evaluation stage).

use crate::error::{IntentError, IntentResult};
use crate::model::Intent;

/// A single validation check. Each check returns `Ok(())` if it
/// passes, or an error describing exactly why it failed — so the
/// caller always knows precisely which rule rejected the Intent.
pub trait ValidationRule: Send + Sync {
    fn name(&self) -> &str;
    fn check(&self, intent: &Intent) -> IntentResult<()>;
}

/// Goal must not be empty or whitespace-only (Part 3 §7,
/// "Goal empty to nahi?").
pub struct NonEmptyGoalRule;

impl ValidationRule for NonEmptyGoalRule {
    fn name(&self) -> &str {
        "non_empty_goal"
    }

    fn check(&self, intent: &Intent) -> IntentResult<()> {
        if intent.original_goal.trim().is_empty() {
            return Err(IntentError::EmptyGoal(intent.id()));
        }
        Ok(())
    }
}

/// Goal must be reasonably short — extremely long "goals" are more
/// likely malformed input than a genuine user intent. This is a
/// pragmatic guard rail, not part of the original spec, but keeps
/// the pipeline defensive.
pub struct MaxGoalLengthRule {
    pub max_chars: usize,
}

impl Default for MaxGoalLengthRule {
    fn default() -> Self {
        Self { max_chars: 4000 }
    }
}

impl ValidationRule for MaxGoalLengthRule {
    fn name(&self) -> &str {
        "max_goal_length"
    }

    fn check(&self, intent: &Intent) -> IntentResult<()> {
        if intent.original_goal.chars().count() > self.max_chars {
            return Err(IntentError::ValidationFailed {
                intent_id: intent.id(),
                reason: format!("goal exceeds {} characters", self.max_chars),
            });
        }
        Ok(())
    }
}

/// Conflicting constraints must be rejected early rather than
/// silently producing an unsatisfiable plan later. Example:
/// `offline_only` combined with a goal that explicitly requires
/// network access is caught downstream by the Planner, but directly
/// contradictory flags (none currently exist, but the hook is here
/// for future constraint combinations) are caught here.
pub struct ConsistentConstraintsRule;

impl ValidationRule for ConsistentConstraintsRule {
    fn name(&self) -> &str {
        "consistent_constraints"
    }

    fn check(&self, intent: &Intent) -> IntentResult<()> {
        if intent.constraints.offline_only && intent.constraints.no_network {
            // Not actually contradictory (both mean "no network"),
            // but flagged as redundant so callers can be alerted to
            // simplify their constraint set. Kept as a warning-level
            // no-op rather than a hard failure.
            tracing::debug!(
                intent_id = %intent.id(),
                "both offline_only and no_network set; redundant but not invalid"
            );
        }
        Ok(())
    }
}

/// Runs a fixed pipeline of validation rules against an Intent,
/// in order, stopping at the first failure — matching Part 3 §7's
/// implication that validation is a single pass/fail gate.
pub struct IntentValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl IntentValidator {
    /// The standard rule set. Additional domain-specific rules can
    /// be appended via [`IntentValidator::with_rule`].
    pub fn standard() -> Self {
        Self {
            rules: vec![
                Box::new(NonEmptyGoalRule),
                Box::new(MaxGoalLengthRule::default()),
                Box::new(ConsistentConstraintsRule),
            ],
        }
    }

    pub fn with_rule(mut self, rule: Box<dyn ValidationRule>) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn validate(&self, intent: &Intent) -> IntentResult<()> {
        for rule in &self.rules {
            rule.check(intent).map_err(|e| {
                tracing::warn!(
                    intent_id = %intent.id(),
                    rule = rule.name(),
                    error = %e,
                    "intent validation rule failed"
                );
                e
            })?;
        }
        Ok(())
    }
}

impl Default for IntentValidator {
    fn default() -> Self {
        Self::standard()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Intent, IntentSource};
    use uuid::Uuid;

    fn sample_intent(goal: &str) -> Intent {
        Intent::new(goal, IntentSource::Cli, Uuid::new_v4())
    }

    #[test]
    fn empty_goal_is_rejected() {
        let validator = IntentValidator::standard();
        let intent = sample_intent("   ");
        assert!(validator.validate(&intent).is_err());
    }

    #[test]
    fn normal_goal_passes() {
        let validator = IntentValidator::standard();
        let intent = sample_intent("Build my Android app");
        assert!(validator.validate(&intent).is_ok());
    }

    #[test]
    fn overly_long_goal_is_rejected() {
        let validator = IntentValidator::standard()
            .with_rule(Box::new(MaxGoalLengthRule { max_chars: 10 }));
        let intent = sample_intent("this goal text is definitely longer than ten characters");
        assert!(validator.validate(&intent).is_err());
    }
}
