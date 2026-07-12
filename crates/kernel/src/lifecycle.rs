//! Universal Lifecycle (Architecture Bible, Part 1 §11, Part 2A §15).
//!
//! Every kernel-managed object — modules, sessions, tasks — moves
//! through the same fixed set of states. Transitions are validated;
//! invalid transitions are rejected rather than silently allowed.

use crate::error::{KernelError, KernelResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed,
    Recovering,
    Archived,
    Destroyed,
}

impl std::fmt::Display for LifecycleState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl LifecycleState {
    /// Defines the permanent, allowed transition table.
    /// This is intentionally exhaustive and explicit — no wildcard
    /// "anything goes" fallback, per Kernel Rule: "Everything observable,
    /// no hidden behavior."
    pub fn can_transition_to(&self, next: LifecycleState) -> bool {
        use LifecycleState::*;
        matches!(
            (self, next),
            (Created, Initializing)
                | (Initializing, Ready)
                | (Initializing, Failed)
                | (Ready, Running)
                | (Running, Paused)
                | (Paused, Running)
                | (Running, Stopping)
                | (Paused, Stopping)
                | (Stopping, Stopped)
                | (Stopped, Archived)
                | (Stopped, Destroyed)
                | (Running, Failed)
                | (Failed, Recovering)
                | (Recovering, Ready)
                | (Recovering, Failed)
                | (Archived, Destroyed)
        )
    }
}

/// A small state machine wrapper that enforces valid transitions
/// and records the transition history for replay/debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleMachine {
    current: LifecycleState,
    history: Vec<LifecycleState>,
}

impl LifecycleMachine {
    pub fn new() -> Self {
        Self {
            current: LifecycleState::Created,
            history: vec![LifecycleState::Created],
        }
    }

    pub fn state(&self) -> LifecycleState {
        self.current
    }

    pub fn history(&self) -> &[LifecycleState] {
        &self.history
    }

    pub fn transition_to(&mut self, next: LifecycleState) -> KernelResult<()> {
        if !self.current.can_transition_to(next) {
            return Err(KernelError::InvalidLifecycleTransition {
                from: self.current.to_string(),
                to: next.to_string(),
            });
        }
        self.current = next;
        self.history.push(next);
        Ok(())
    }
}

impl Default for LifecycleMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_transition_succeeds() {
        let mut m = LifecycleMachine::new();
        assert!(m.transition_to(LifecycleState::Initializing).is_ok());
        assert!(m.transition_to(LifecycleState::Ready).is_ok());
        assert_eq!(m.state(), LifecycleState::Ready);
    }

    #[test]
    fn invalid_transition_rejected() {
        let mut m = LifecycleMachine::new();
        assert!(m.transition_to(LifecycleState::Running).is_err());
        assert_eq!(m.state(), LifecycleState::Created);
    }

    #[test]
    fn history_is_recorded() {
        let mut m = LifecycleMachine::new();
        m.transition_to(LifecycleState::Initializing).unwrap();
        m.transition_to(LifecycleState::Ready).unwrap();
        assert_eq!(m.history().len(), 3);
    }
}
