//! # orbvynx-kernel
//!
//! The ORBVYNX microkernel. Per the Architecture Bible (Part 2A):
//!
//! > "Kernel ka ek hi mission hai: provide a deterministic execution
//! > environment. Bas. Kernel kabhi decision nahi leta."
//!
//! This crate deliberately contains **zero** business logic. It does
//! not know what Git, Android, or AI are. It only provides:
//!
//! - [`events`] — the async Event Bus (the system's nervous system)
//! - [`registry`] — Module Registry & Service Registry
//! - [`lifecycle`] — the universal lifecycle state machine
//! - [`boot`] — the Boot Manager and staged boot sequence
//! - [`clock`] — the single system-wide Clock Service
//! - [`session`] — Session Manager
//! - [`identity`] — the universal identity block shared by all objects
//! - [`error`] — the canonical `KernelError` type
//!
//! Every other ORBVYNX crate (Intent, Planner, Workflow, Executor,
//! Capability, Plugin Runtime) depends on this crate — never the
//! other way around.

pub mod boot;
pub mod clock;
pub mod error;
pub mod events;
pub mod identity;
pub mod lifecycle;
pub mod registry;
pub mod session;

pub use error::{KernelError, KernelResult};
pub use events::{Event, EventBus, EventKind, EventSubscription};
pub use identity::{Identity, ObjectId};
pub use lifecycle::{LifecycleMachine, LifecycleState};
pub use registry::{ModuleInfo, ModuleRegistry, Service, ServiceRegistry};
pub use session::{Platform, Session};

use boot::{BootContext, BootManager, BootReport};
use clock::SystemClock;
use std::sync::Arc;

/// The fully-booted kernel handle. This is what every other engine
/// (Planner, Executor, ...) receives at startup to access shared
/// kernel services — never a global singleton, always passed
/// explicitly.
#[derive(Clone)]
pub struct Kernel {
    pub event_bus: EventBus,
    pub module_registry: ModuleRegistry,
    pub service_registry: ServiceRegistry,
}

impl Kernel {
    /// Boots the kernel through the standard stage sequence.
    /// Additional stages from higher-level crates should be appended
    /// via [`Kernel::standard_boot_manager`] before booting.
    pub async fn boot() -> error::KernelResult<(Self, BootReport)> {
        let manager = Self::standard_boot_manager();

        let ctx = BootContext {
            event_bus: EventBus::default(),
            module_registry: ModuleRegistry::new(),
            service_registry: ServiceRegistry::new(),
        };

        let (report, ctx) = manager.boot(ctx)?;

        Ok((
            Kernel {
                event_bus: ctx.event_bus,
                module_registry: ctx.module_registry,
                service_registry: ctx.service_registry,
            },
            report,
        ))
    }

    /// The fixed, minimal kernel-owned boot stages. Business-logic
    /// crates append their own stages on top of this rather than
    /// replacing it.
    pub fn standard_boot_manager() -> BootManager {
        let mut manager = BootManager::new();

        manager.add_stage("clock", |ctx: &BootContext| {
            ctx.service_registry
                .register(Arc::new(SystemClock::new()));
            Ok(())
        });

        manager.add_stage("event_bus", |_ctx: &BootContext| Ok(()));

        manager.add_stage("registries", |_ctx: &BootContext| Ok(()));

        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn kernel_boots_successfully() {
        let (kernel, report) = Kernel::boot().await.unwrap();
        assert!(!report.stage_timings.is_empty());
        assert_eq!(kernel.module_registry.list().len(), 0);
    }
}
