use orbvynx_executor::{CapabilityRegistry, EchoCapability, Executor};
use orbvynx_intent::{IntentEngine, IntentSource};
use orbvynx_planner::{PlanningContext, PlanningPipeline, PolicyConstraints, PolicyEvaluator, StaticCapabilityDiscovery};
use orbvynx_workflow::{Task, TaskGraph, Workflow};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (kernel, boot_report) = orbvynx_kernel::Kernel::boot().await?;
    println!("ORBVYNX booted in {}ms", boot_report.total_millis);

    let session_id = Uuid::new_v4();
    let intent_engine = IntentEngine::new(kernel.event_bus.clone());
    let intent = intent_engine.intake("build app", IntentSource::Cli, session_id)?;
    println!("Intent: {} -> {}", intent.original_goal, intent.effective_goal());

    let discovery = Arc::new(StaticCapabilityDiscovery::new(vec!["test.echo".to_string()]));
    let policy = PolicyEvaluator::new(PolicyConstraints::default());
    let pipeline = PlanningPipeline::new(discovery, policy);
    let ctx = PlanningContext { internet_available: true, ..Default::default() };

    let plan = pipeline.plan(&intent, &ctx, vec!["test.echo".to_string()]).await?;
    println!("Plan generated: {} step(s), risk={}", plan.steps.len(), plan.risk_score.0);

    let tasks: Vec<Task> = plan.steps.iter()
        .map(|s| Task::new(format!("step-{}", s.order), s.description.clone(), s.required_capability.clone()))
        .collect();
    let graph = TaskGraph::new(tasks);
    let mut workflow = Workflow::from_plan(&plan, graph);
    workflow.validate()?;
    println!("Workflow validated: {} tasks", workflow.graph.tasks.len());

    let mut registry = CapabilityRegistry::new();
    registry.register(Arc::new(EchoCapability));
    let executor = Executor::new(registry, kernel.event_bus.clone());

    for task in workflow.graph.tasks.values() {
        let result = executor.execute(task).await?;
        println!("Task {} -> {:?}", task.id, result.outcome);
    }

    println!("ORBVYNX run complete.");
    Ok(())
}
