//! Context Analysis, Resource Discovery, Capability Discovery
//! (Architecture Bible, Part 4 §6, §7, §8).
//!
//! "Ye data khud collect nahi karega. Ye Kernel Services se lega."
//! This module defines the shapes the Planner reasons over; the
//! actual environment probing (filesystem checks, network checks)
//! is delegated to pluggable `EnvironmentProbe` implementations so
//! the Planner's decision logic stays testable without touching a
//! real filesystem or network.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Environmental facts the Planner reasons over (Part 4 §6).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlanningContext {
    pub working_directory: String,
    pub is_android_project: bool,
    pub is_rust_project: bool,
    pub is_git_repository: bool,
    pub internet_available: bool,
    pub battery_low: bool,
    pub disk_space_mb: u64,
}

/// Abstracts environment probing so the Planner never calls the
/// filesystem/network directly (Part 4 §6: "Ye data khud collect
/// nahi karega"). A real implementation lives in a higher-level
/// crate or the CLI app; tests use a fake/mock implementation.
#[async_trait]
pub trait EnvironmentProbe: Send + Sync {
    async fn probe(&self, working_directory: &str) -> PlanningContext;
}

/// Metadata about something the Planner might need to operate on
/// (Part 4 §7). The Planner only ever sees this metadata — actual
/// access goes through the Capability layer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub name: String,
    pub kind: String,
    pub available: bool,
}

/// A pluggable source of "what resources currently exist" (Part 4 §7).
#[async_trait]
pub trait ResourceDiscovery: Send + Sync {
    async fn discover(&self, ctx: &PlanningContext) -> Vec<ResourceInfo>;
}

/// The set of capability names currently installed/available
/// (Part 4 §8). The Planner compares a goal's required capabilities
/// against this set to decide if a plan is even possible.
#[async_trait]
pub trait CapabilityDiscovery: Send + Sync {
    async fn available_capabilities(&self) -> HashSet<String>;
}

/// A simple in-memory implementation useful for tests and for a
/// minimal first running system before a real Capability Registry
/// (Part 7) exists.
pub struct StaticCapabilityDiscovery {
    capabilities: HashSet<String>,
}

impl StaticCapabilityDiscovery {
    pub fn new(capabilities: impl IntoIterator<Item = String>) -> Self {
        Self {
            capabilities: capabilities.into_iter().collect(),
        }
    }
}

#[async_trait]
impl CapabilityDiscovery for StaticCapabilityDiscovery {
    async fn available_capabilities(&self) -> HashSet<String> {
        self.capabilities.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn static_capability_discovery_returns_configured_set() {
        let discovery = StaticCapabilityDiscovery::new(vec![
            "android.build".to_string(),
            "filesystem.read".to_string(),
        ]);
        let caps = discovery.available_capabilities().await;
        assert!(caps.contains("android.build"));
        assert!(!caps.contains("git.push"));
    }
}
