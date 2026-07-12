use dashmap::DashMap;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Default)]
pub struct PermissionStore {
    grants: DashMap<Uuid, HashSet<String>>,
}

impl PermissionStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&self, session_id: Uuid, capability: impl Into<String>) {
        self.grants.entry(session_id).or_default().insert(capability.into());
    }

    pub fn revoke(&self, session_id: Uuid, capability: &str) {
        if let Some(mut set) = self.grants.get_mut(&session_id) {
            set.remove(capability);
        }
    }

    pub fn has(&self, session_id: Uuid, capability: &str) -> bool {
        self.grants.get(&session_id).map(|s| s.contains(capability)).unwrap_or(false)
    }

    pub fn grant_all(&self, session_id: Uuid, capabilities: impl IntoIterator<Item = String>) {
        let mut set = self.grants.entry(session_id).or_default();
        set.extend(capabilities);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_and_check() {
        let store = PermissionStore::new();
        let session = Uuid::new_v4();
        assert!(!store.has(session, "git.push"));
        store.grant(session, "git.push");
        assert!(store.has(session, "git.push"));
    }

    #[test]
    fn revoke_removes_permission() {
        let store = PermissionStore::new();
        let session = Uuid::new_v4();
        store.grant(session, "git.push");
        store.revoke(session, "git.push");
        assert!(!store.has(session, "git.push"));
    }

    #[test]
    fn permissions_are_session_scoped() {
        let store = PermissionStore::new();
        let session_a = Uuid::new_v4();
        let session_b = Uuid::new_v4();
        store.grant(session_a, "git.push");
        assert!(store.has(session_a, "git.push"));
        assert!(!store.has(session_b, "git.push"));
    }
}
