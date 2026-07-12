//! Module Registry & Service Registry (Architecture Bible, Part 2A §9, §10).
//!
//! The kernel never hardcodes which engines exist. Every subsystem
//! (Planner, Executor, Plugin Runtime, ...) registers itself here at
//! boot, and the kernel exposes both to whoever needs them by name —
//! never by direct compiled-in reference.

use crate::error::{KernelError, KernelResult};
use dashmap::DashMap;
use std::sync::Arc;

/// Metadata describing a registered module. Modules themselves are
/// opaque to the kernel — it only tracks identity and health, never
/// their internal logic (Kernel Rule: "no business logic").
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub healthy: bool,
}

/// Thread-safe registry of modules known to the kernel.
#[derive(Clone, Default)]
pub struct ModuleRegistry {
    modules: Arc<DashMap<String, ModuleInfo>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, info: ModuleInfo) -> KernelResult<()> {
        if self.modules.contains_key(&info.name) {
            return Err(KernelError::ModuleAlreadyRegistered(info.name));
        }
        self.modules.insert(info.name.clone(), info);
        Ok(())
    }

    pub fn unregister(&self, name: &str) -> KernelResult<()> {
        self.modules
            .remove(name)
            .map(|_| ())
            .ok_or_else(|| KernelError::ModuleNotFound(name.to_string()))
    }

    pub fn get(&self, name: &str) -> KernelResult<ModuleInfo> {
        self.modules
            .get(name)
            .map(|entry| entry.clone())
            .ok_or_else(|| KernelError::ModuleNotFound(name.to_string()))
    }

    pub fn set_health(&self, name: &str, healthy: bool) -> KernelResult<()> {
        let mut entry = self
            .modules
            .get_mut(name)
            .ok_or_else(|| KernelError::ModuleNotFound(name.to_string()))?;
        entry.healthy = healthy;
        Ok(())
    }

    pub fn list(&self) -> Vec<ModuleInfo> {
        self.modules.iter().map(|e| e.value().clone()).collect()
    }
}

/// Runtime services (Clock, Logger, Storage, ...) exposed via
/// dynamic dispatch so any module can be swapped for another
/// implementation without recompiling its consumers
/// (Part 2A §10, "Everything is replaceable").
pub trait Service: Send + Sync {
    fn name(&self) -> &str;
}

#[derive(Clone, Default)]
pub struct ServiceRegistry {
    services: Arc<DashMap<String, Arc<dyn Service>>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, service: Arc<dyn Service>) {
        self.services.insert(service.name().to_string(), service);
    }

    pub fn get(&self, name: &str) -> KernelResult<Arc<dyn Service>> {
        self.services
            .get(name)
            .map(|entry| entry.clone())
            .ok_or_else(|| KernelError::ServiceNotFound(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_get_module() {
        let registry = ModuleRegistry::new();
        registry
            .register(ModuleInfo {
                name: "planner".into(),
                version: "0.1.0".into(),
                healthy: true,
            })
            .unwrap();
        let info = registry.get("planner").unwrap();
        assert_eq!(info.version, "0.1.0");
    }

    #[test]
    fn duplicate_registration_fails() {
        let registry = ModuleRegistry::new();
        let info = ModuleInfo {
            name: "planner".into(),
            version: "0.1.0".into(),
            healthy: true,
        };
        registry.register(info.clone()).unwrap();
        assert!(registry.register(info).is_err());
    }

    #[test]
    fn unregister_missing_module_fails() {
        let registry = ModuleRegistry::new();
        assert!(registry.unregister("ghost").is_err());
    }
}
