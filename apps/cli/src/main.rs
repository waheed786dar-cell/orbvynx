mod cli;

use clap::Parser;
use cli::{Cli, Commands, GitAction, PluginAction};
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

async fn load_plugins(registry: &mut CapabilityRegistry, plugins_dir: PathBuf) -> Vec<(String, String)> {
    let mut plugin_registry = PluginRegistry::new();
    let _ = plugin_registry.load_from_directory(plugins_dir).await;

    let mut loaded = Vec::new();
    for name in plugin_registry.list() {
        if let Some(plugin) = plugin_registry.get(&name) {
            let capability_name = plugin.manifest.capability_name.clone();
            registry.register(Arc::new(PluginCapability::new(plugin)));
            loaded.push((name, capability_name));
        }
    }
    loaded
}

async fn run_goal(goal: String, required_capability: String, params: HashMap<String, serde_json::Value>) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let (kernel, boot_report) = orbvynx_kernel::Kernel::boot().await?;
    println!("ORBVYNX booted in {}ms", boot_report.total_millis);

    let mut registry = build_registry(cwd.clone());
    let plugins_dir = cwd.join("examples").join("plugins");
    let plugins = load_plugins(&mut registry, plugins_dir).await;
    if !plugins.is_empty() {
        let names: Vec<String> = plugins.iter().map(|(n, _)| n.clone()).collect();
        println!("Loaded {} plugin(s): {}", plugins.len(), names.join(", "));
    }

    let session_id = Uuid::new_v4();
    let intent_engine = IntentEngine::new(kernel.event_bus.clone());
    let intent = intent_engine.intake(goal, IntentSource::Cli, session_id)?;
    println!("Intent: {} -> {}", intent.original_goal, intent.effective_goal());

    let mut known: Vec<String> = vec!["git.status".into(), "git.commit".into(), "git.push".into()];
    known.extend(plugins.iter().map(|(_, cap)| cap.clone()));

    let discovery = Arc::new(StaticCapabilityDiscovery::new(known));
    let policy = PolicyEvaluator::new(PolicyConstraints::default());
    let pipeline = PlanningPipeline::new(discovery, policy);
    let ctx = PlanningContext { internet_available: true, working_directory: cwd.to_string_lossy().to_string(), ..Default::default() };

    let plan = pipeline.plan(&intent, &ctx, vec![required_capability]).await?;
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
        let mut task_params = params.clone();
        task_params.entry("cwd".to_string()).or_insert_with(|| serde_json::json!(cwd.to_string_lossy()));

        let result = executor.execute(task, task_params).await?;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            let (kernel, report) = orbvynx_kernel::Kernel::boot().await?;
            println!("ORBVYNX status: booted in {}ms", report.total_millis);
            println!("Registered modules: {}", kernel.module_registry.list().len());
            println!("Event bus subscribers: {}", kernel.event_bus.subscriber_count());
        }

        Commands::Run { goal } => {
            let goal_str = goal.join(" ");
            if goal_str.trim().is_empty() {
                anyhow::bail!("provide a goal, e.g. 'orbvynx run build my app'");
            }
            run_goal(goal_str, "git.status".to_string(), HashMap::new()).await?;
        }

        Commands::Git { action } => match action {
            GitAction::Status => {
                run_goal("git status".to_string(), "git.status".to_string(), HashMap::new()).await?;
            }
            GitAction::Commit { message } => {
                let mut params = HashMap::new();
                params.insert("message".to_string(), serde_json::json!(message));
                run_goal("git commit".to_string(), "git.commit".to_string(), params).await?;
            }
            GitAction::Push => {
                run_goal("git push".to_string(), "git.push".to_string(), HashMap::new()).await?;
            }
        },

        Commands::Plugin { action } => match action {
            PluginAction::List => {
                let cwd = std::env::current_dir()?;
                let mut plugin_registry = PluginRegistry::new();
                let count = plugin_registry.load_from_directory(cwd.join("examples").join("plugins")).await.unwrap_or(0);
                if count == 0 {
                    println!("No plugins found in ./examples/plugins");
                } else {
                    println!("Found {count} plugin(s):");
                    for name in plugin_registry.list() {
                        if let Some(plugin) = plugin_registry.get(&name) {
                            println!("  {} -> capability: {} ({})", plugin.manifest.name, plugin.manifest.capability_name, plugin.manifest.description);
                        }
                    }
                }
            }
        },
    }

    Ok(())
}
