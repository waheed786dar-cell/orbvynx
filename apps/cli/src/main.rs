use orbvynx_executor::capabilities::{
    FilesystemReadCapability, FilesystemWriteCapability, GitCommitCapability, GitPushCapability, GitStatusCapability,
};
use orbvynx_executor::{CapabilityRegistry, Executor};
use orbvynx_intent::{IntentEngine, IntentSource};
use orbvynx_planner::{PlanningContext, PlanningPipeline, PolicyConstraints, PolicyEvaluator, StaticCapabilityDiscovery};
use orbvynx_plugin_runtime::{PluginCapability, PluginRegistry};
use orbvynx_workflow::{Task, TaskGraph, Workflow};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

fn build_registry(cwd: PathBuf) -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::new();
    registry.register(Arc::new(FilesystemReadCapability::new(vec![cwd.clone()])));
    registry.register(Arc::new(FilesystemWriteCapability::new(vec![cwd])));
    registry.register(Arc::new(GitStatusCapability));
    registry.register(Arc::new(GitCommitCapability));
    registry.register(Arc::new(GitPushCapability));
    registry
}

async fn load_plugins(registry: &mut CapabilityRegistry, plugins_dir: PathBuf) -> Vec<String> {
    let mut plugin_registry = PluginRegistry::new();
    let loaded = plugin_registry.load_from_directory(plugins_dir).await.unwrap_or(0);

    let mut names = Vec::new();
    for name in plugin_registry.list() {
        if let Some(plugin) = plugin_registry.get(&name) {
            let capability_name = plugin.manifest.capability_name.clone();
            registry.register(Arc::new(PluginCapability::new(plugin)));
            names.push(capability_name);
        }
    }

    if loaded > 0 {
        println!("Loaded {loaded} plugin(s): {}", names.join(", "));
    }
    names
}

fn capability_for_goal(goal: &str, plugin_capabilities: &[String]) -> Vec<String> {
    let lower = goal.to_lowercase();

    for plugin_cap in plugin_capabilities {
        let short_name = plugin_cap.rsplit('.').next().unwrap_or(plugin_cap);
        if lower.contains(short_name) {
            return vec![plugin_cap.clone()];
        }
    }

    if lower.contains("push") {
        vec!["git.push".to_string()]
    } else if lower.contains("commit") {
        vec!["git.commit".to_string()]
    } else {
        vec!["git.status".to_string()]
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let goal: String = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let goal = if goal.trim().is_empty() { "git status".to_string() } else { goal };

    let cwd = std::env::current_dir()?;

    let (kernel, boot_report) = orbvynx_kernel::Kernel::boot().await?;
    println!("ORBVYNX booted in {}ms", boot_report.total_millis);

    let mut registry = build_registry(cwd.clone());
    let plugins_dir = cwd.join("examples").join("plugins");
    let plugin_capabilities = load_plugins(&mut registry, plugins_dir).await;

    let session_id = Uuid::new_v4();
    let intent_engine = IntentEngine::new(kernel.event_bus.clone());
    let intent = intent_engine.intake(goal.clone(), IntentSource::Cli, session_id)?;
    println!("Intent: {} -> {}", intent.original_goal, intent.effective_goal());

    let required = capability_for_goal(&goal, &plugin_capabilities);

    let mut known_capabilities = vec![
        "git.status".to_string(),
        "git.commit".to_string(),
        "git.push".to_string(),
    ];
    known_capabilities.extend(plugin_capabilities.clone());

    let discovery = Arc::new(StaticCapabilityDiscovery::new(known_capabilities));
    let policy = PolicyEvaluator::new(PolicyConstraints::default());
    let pipeline = PlanningPipeline::new(discovery, policy);
    let ctx = PlanningContext { internet_available: true, working_directory: cwd.to_string_lossy().to_string(), ..Default::default() };

    let plan = pipeline.plan(&intent, &ctx, required).await?;
    println!("Plan generated: {} step(s), risk={}", plan.steps.len(), plan.risk_score.0);

    let tasks: Vec<Task> = plan.steps.iter()
        .map(|s| Task::new(format!("step-{}", s.order), s.description.clone(), s.required_capability.clone()))
        .collect();
    let graph = TaskGraph::new(tasks);
    let mut workflow = Workflow::from_plan(&plan, graph);
    workflow.validate()?;
    println!("Workflow validated: {} tasks", workflow.graph.tasks.len());

    let executor = Executor::new(registry, kernel.event_bus.clone());

    for task in workflow.graph.tasks.values() {
        let mut params = HashMap::new();
        params.insert("cwd".to_string(), serde_json::json!(cwd.to_string_lossy()));
        if task.required_capability == "git.commit" {
            params.insert("message".to_string(), serde_json::json!("ORBVYNX automated commit"));
        }

        let result = executor.execute(task, params).await?;
        println!("Task {} -> {:?}", task.id, result.outcome);
        if let Some(output) = result.output {
            println!("  output: {output}");
        }
        if let Some(err) = result.error {
            println!("  error: {err}");
        }
    }

    println!("ORBVYNX run complete.");
    Ok(())
}
