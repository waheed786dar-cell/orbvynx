//! Boot Manager (Architecture Bible, Part 2A §7, Part 2B).
//!
//! Drives the kernel through its fixed boot sequence, emitting a
//! `KernelBootStageCompleted` event (with timing) for every stage,
//! so failures are traceable to the exact stage that broke —
//! matching the `orbvynx doctor` boot-metrics vision in Part 2B §6.

use crate::error::{KernelError, KernelResult};
use crate::events::{Event, EventBus, EventKind};
use crate::lifecycle::{LifecycleMachine, LifecycleState};
use crate::registry::{ModuleRegistry, ServiceRegistry};
use std::time::Instant;

/// A single named boot stage. Stages run strictly in the order
/// they're added — no reordering, no parallel boot stages, per
/// Part 2B §5 (Module Loading Sequence is fixed by design).
pub struct BootStage {
    pub name: String,
    pub action: Box<dyn FnOnce(&BootContext) -> KernelResult<()> + Send>,
}

/// Shared context handed to every boot stage so stages can register
/// modules/services as they initialize.
pub struct BootContext {
    pub event_bus: EventBus,
    pub module_registry: ModuleRegistry,
    pub service_registry: ServiceRegistry,
}

pub struct BootManager {
    stages: Vec<BootStage>,
    lifecycle: LifecycleMachine,
}

#[derive(Debug)]
pub struct BootReport {
    pub total_millis: u128,
    pub stage_timings: Vec<(String, u128)>,
}

impl BootManager {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            lifecycle: LifecycleMachine::new(),
        }
    }

    pub fn add_stage<F>(&mut self, name: impl Into<String>, action: F)
    where
        F: FnOnce(&BootContext) -> KernelResult<()> + Send + 'static,
    {
        self.stages.push(BootStage {
            name: name.into(),
            action: Box::new(action),
        });
    }

    /// Runs every registered stage in order. On the first failing
    /// stage, boot stops immediately and returns `BootFailed` naming
    /// exactly which stage broke — the kernel never continues booting
    /// past a failed stage.
    pub fn boot(mut self, ctx: BootContext) -> KernelResult<(BootReport, BootContext)> {
        self.lifecycle
            .transition_to(LifecycleState::Initializing)
            .map_err(|e| KernelError::BootFailed {
                stage: "lifecycle".into(),
                reason: e.to_string(),
            })?;

        ctx.event_bus
            .publish(Event::new("kernel.boot", EventKind::KernelBootStarted));

        let overall_start = Instant::now();
        let mut stage_timings = Vec::new();

        for stage in self.stages {
            let stage_start = Instant::now();
            let name = stage.name.clone();

            (stage.action)(&ctx).map_err(|e| KernelError::BootFailed {
                stage: name.clone(),
                reason: e.to_string(),
            })?;

            let millis = stage_start.elapsed().as_millis();
            stage_timings.push((name.clone(), millis));

            ctx.event_bus.publish(Event::new(
                "kernel.boot",
                EventKind::KernelBootStageCompleted {
                    stage: name,
                    millis: millis as u64,
                },
            ));
        }

        self.lifecycle
            .transition_to(LifecycleState::Ready)
            .map_err(|e| KernelError::BootFailed {
                stage: "lifecycle".into(),
                reason: e.to_string(),
            })?;

        ctx.event_bus
            .publish(Event::new("kernel.boot", EventKind::KernelReady));

        Ok((
            BootReport {
                total_millis: overall_start.elapsed().as_millis(),
                stage_timings,
            },
            ctx,
        ))
    }
}

impl Default for BootManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_runs_stages_in_order_and_reports_timings() {
        let mut boot = BootManager::new();
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

        let log_a = log.clone();
        boot.add_stage("stage_a", move |_ctx| {
            log_a.lock().unwrap().push("a");
            Ok(())
        });

        let log_b = log.clone();
        boot.add_stage("stage_b", move |_ctx| {
            log_b.lock().unwrap().push("b");
            Ok(())
        });

        let ctx = BootContext {
            event_bus: EventBus::default(),
            module_registry: ModuleRegistry::new(),
            service_registry: ServiceRegistry::new(),
        };

        let (report, _ctx) = boot.boot(ctx).unwrap();
        assert_eq!(*log.lock().unwrap(), vec!["a", "b"]);
        assert_eq!(report.stage_timings.len(), 2);
    }

    #[test]
    fn boot_stops_at_first_failing_stage() {
        let mut boot = BootManager::new();

        boot.add_stage("good_stage", |_ctx| Ok(()));
        boot.add_stage("bad_stage", |_ctx| {
            Err(KernelError::Internal("simulated failure".into()))
        });
        boot.add_stage("never_runs", |_ctx| {
            panic!("this stage must never execute");
        });

        let ctx = BootContext {
            event_bus: EventBus::default(),
            module_registry: ModuleRegistry::new(),
            service_registry: ServiceRegistry::new(),
        };

        match boot.boot(ctx) {
            Err(KernelError::BootFailed { stage, .. }) => assert_eq!(stage, "bad_stage"),
            Err(other) => panic!("expected BootFailed, got {other:?}"),
            Ok(_) => panic!("expected boot to fail"),
        }
    }
}
