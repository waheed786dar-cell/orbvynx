use crate::error::{SecurityError, SecurityResult};
use crate::permissions::PermissionStore;
use crate::policy::SecurityPolicy;
use orbvynx_kernel::{Event, EventBus, EventKind};
use serde_json::json;
use uuid::Uuid;

pub struct PermissionDispatcher {
    permissions: PermissionStore,
    policy: SecurityPolicy,
    event_bus: EventBus,
}

impl PermissionDispatcher {
    pub fn new(policy: SecurityPolicy, event_bus: EventBus) -> Self {
        Self { permissions: PermissionStore::new(), policy, event_bus }
    }

    pub fn grant(&self, session_id: Uuid, capability: impl Into<String>) {
        self.permissions.grant(session_id, capability);
    }

    pub fn check(&self, session_id: Uuid, capability: &str) -> SecurityResult<()> {
        self.policy.check_capability(capability)?;

        if self.policy.require_explicit_grant && !self.permissions.has(session_id, capability) {
            self.publish_denied(session_id, capability);
            return Err(SecurityError::PermissionDenied {
                session_id,
                capability: capability.to_string(),
            });
        }

        self.publish_granted(session_id, capability);
        Ok(())
    }

    fn publish_granted(&self, session_id: Uuid, capability: &str) {
        self.event_bus.publish(Event::new("security", EventKind::PermissionGranted {
            capability: capability.to_string(),
            session_id,
        }));
    }

    fn publish_denied(&self, session_id: Uuid, capability: &str) {
        self.event_bus.publish(Event::new("security", EventKind::PermissionDenied {
            capability: capability.to_string(),
            session_id,
        }));
        let _ = json!({ "capability": capability, "session_id": session_id.to_string() });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permissive_policy_allows_without_explicit_grant() {
        let dispatcher = PermissionDispatcher::new(SecurityPolicy::permissive(), EventBus::default());
        let session = Uuid::new_v4();
        assert!(dispatcher.check(session, "git.push").is_ok());
    }

    #[test]
    fn strict_policy_requires_explicit_grant() {
        let dispatcher = PermissionDispatcher::new(SecurityPolicy::strict(), EventBus::default());
        let session = Uuid::new_v4();
        assert!(dispatcher.check(session, "git.push").is_err());

        dispatcher.grant(session, "git.push");
        assert!(dispatcher.check(session, "git.push").is_ok());
    }

    #[test]
    fn denied_capability_fails_even_with_grant() {
        let mut policy = SecurityPolicy::permissive();
        policy.denied_capabilities.push("git.push".to_string());
        let dispatcher = PermissionDispatcher::new(policy, EventBus::default());
        let session = Uuid::new_v4();
        dispatcher.grant(session, "git.push");
        assert!(dispatcher.check(session, "git.push").is_err());
    }
}
