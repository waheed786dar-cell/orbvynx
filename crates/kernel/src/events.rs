//! The Kernel Event Bus (Architecture Bible, Part 2A §11).
//!
//! "Ye ORBVYNX ka nervous system hai." Every significant kernel
//! action is published here as an immutable, timestamped event.
//! Built on `tokio::sync::broadcast` so any number of subscribers
//! (Planner, Executor, CLI, GUI, logging) can observe the same
//! stream independently without coupling to the publisher.

use crate::error::{KernelError, KernelResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

/// The canonical event envelope. Every event flowing through the
/// bus is wrapped in this — payload-specific data lives in `EventKind`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub correlation_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub kind: EventKind,
}

impl Event {
    pub fn new(source: impl Into<String>, kind: EventKind) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            source: source.into(),
            correlation_id: None,
            session_id: None,
            kind,
        }
    }

    pub fn with_correlation(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }

    pub fn with_session(mut self, id: Uuid) -> Self {
        self.session_id = Some(id);
        self
    }
}

/// Kernel-level event payloads. Higher-level crates (Intent, Planner,
/// Workflow, Executor) define and publish their own `EventKind`
/// variants in their own crates via the same `Event` envelope —
/// this enum only carries events that the Kernel itself is
/// responsible for (Part 2A §14, boot/module/permission lifecycle).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventKind {
    KernelBootStarted,
    KernelBootStageCompleted { stage: String, millis: u64 },
    KernelReady,
    KernelShuttingDown,
    KernelShutdownComplete,

    ModuleRegistered { module: String },
    ModuleUnregistered { module: String },
    ModuleHealthChanged { module: String, healthy: bool },

    ServiceRegistered { service: String },
    ServiceUnregistered { service: String },

    PermissionGranted { capability: String, session_id: Uuid },
    PermissionDenied { capability: String, session_id: Uuid },

    RecoveryStarted { reason: String },
    RecoveryFinished { success: bool },

    /// Escape hatch for other crates to publish arbitrary structured
    /// events without the kernel needing to know their shape ahead
    /// of time. `topic` acts as a namespaced event name
    /// (e.g. "intent.created", "workflow.task.failed").
    External {
        topic: String,
        payload: serde_json::Value,
    },
}

/// The Event Bus itself. Cheap to clone (internally an `Arc`-backed
/// broadcast sender), so it can be handed to every subsystem freely.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// `capacity` is the number of events retained for slow
    /// subscribers before they start lagging (see `EventBusLagged`).
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all current subscribers. Publishing never
    /// blocks and never fails just because there are zero subscribers.
    pub fn publish(&self, event: Event) {
        let _ = self.sender.send(event);
    }

    /// Subscribe to the event stream. Each subscriber gets its own
    /// independent receiver and will see every event published after
    /// this call.
    pub fn subscribe(&self) -> EventSubscription {
        EventSubscription {
            receiver: self.sender.subscribe(),
        }
    }

    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// A handle for consuming events from the bus.
pub struct EventSubscription {
    receiver: broadcast::Receiver<Event>,
}

impl EventSubscription {
    /// Await the next event. Returns `EventBusLagged(n)` if this
    /// subscriber fell behind and `n` events were dropped for it —
    /// callers should treat this as recoverable and keep consuming.
    pub async fn recv(&mut self) -> KernelResult<Event> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => return Ok(event),
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!(dropped = n, "event bus subscriber lagged");
                    return Err(KernelError::EventBusLagged(n));
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return Err(KernelError::EventBusClosed);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn publish_and_receive() {
        let bus = EventBus::default();
        let mut sub = bus.subscribe();
        bus.publish(Event::new("test", EventKind::KernelReady));
        let event = sub.recv().await.unwrap();
        matches!(event.kind, EventKind::KernelReady);
    }

    #[tokio::test]
    async fn multiple_subscribers_each_get_event() {
        let bus = EventBus::default();
        let mut sub_a = bus.subscribe();
        let mut sub_b = bus.subscribe();
        bus.publish(Event::new("test", EventKind::KernelBootStarted));
        assert!(sub_a.recv().await.is_ok());
        assert!(sub_b.recv().await.is_ok());
    }

    #[tokio::test]
    async fn publish_with_no_subscribers_does_not_panic() {
        let bus = EventBus::default();
        bus.publish(Event::new("test", EventKind::KernelReady));
    }
}
