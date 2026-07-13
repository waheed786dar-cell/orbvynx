mod cli;
mod ui;

use clap::Parser;
use cli::{Cli, Commands, EncodeAction, FsAction, GitAction, HttpAction, JsonAction, PluginAction, SysAction};
use orbvynx_executor::capabilities::{
    Base64DecodeCapability, Base64EncodeCapability, CurrentTimeCapability, EnvGetCapability,
    FileExistsCapability, FilesystemReadCapability, FilesystemWriteCapability, GitCommitCapability,
    GitPushCapability, GitStatusCapability, HttpGetCapability, HttpPostCapability,
    JsonParseCapability, JsonQueryCapability, ListDirectoryCapability, Sha256Capability,
    UrlEncodeCapability, UuidGenerateCapability, ZipCompressCapability,
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

const KNOWN_CAPABILITIES: &[&str] = &[
    "git.status", "git.commit", "git.push",
    "http.get", "http.post",
    "hash.sha256", "archive.compress",
    "text.base64_encode", "text.base64_decode", "text.url_encode", "text.regex_match",
    "system.uuid_generate", "system.env_get", "system.current_time",
    "filesystem.list_directory", "filesystem.exists",
    "json.parse", "json.query",
];

fn build_registry(cwd: PathBuf) -> CapabilityRegistry {
    let mut registry = CapabilityRegistry::new();
    registry.register(Arc::new(FilesystemReadCapability::new(vec![cwd.clone()])));
    registry.register(Arc::new(FilesystemWriteCapability::new(vec![cwd])));
    registry.register(Arc::new(GitStatusCapability));
    registry.register(Arc::new(GitCommitCapability));
    registry.register(Arc::new(GitPushCapability));
    registry.register(Arc::new(HttpGetCapability));
    registry.register(Arc::new(HttpPostCapability));
    registry.register(Arc::new(Sha256Capability));
    registry.register(Arc::new(ZipCompressCapability));
    registry.register(Arc::new(Base64EncodeCapability));
    registry.register(Arc::new(Base64DecodeCapability));
    registry.register(Arc::new(UrlEncodeCapability));
    registry.register(Arc::new(UuidGenerateCapability));
    registry.register(Arc::new(EnvGetCapability));
    registry.register(Arc::new(CurrentTimeCapability));
    registry.register(Arc::new(ListDirectoryCapability));
    registry.register(Arc::new(FileExistsCapability));
    registry.register(Arc::new(JsonParseCapability));
    registry.register(Arc::new(JsonQueryCapability));
    registry
}

async fn load_plugins(registry: &mut CapabilityRegistry, plugins_dir: PathBuf) -> Vec<String> {
    let mut plugin_registry = PluginRegistry::new();
    let count = plugin_registry.load_from_directory(plugins_dir).await.unwrap_or(0);
    let mut names = Vec::new();
    for name in plugin_registry.list() {
        if let Some(plugin) = plugin_registry.get(&name) {
            registry.register(Arc::new(PluginCapability::new(plugin.clone())));
            names.push(plugin.manifest.capability_name.clone());
        }
    }
    if count > 0 {
        ui::info(&format!("loaded {count} plugin(s)"));
    }
    names
}

async fn run_capability(required_capability: &str, params: HashMap<String, serde_json::Value>, goal_hint: &str) -> anyhow::Result<()> {
    let cwd = std::env::current_dir()?;
    let (kernel, boot_report) = orbvynx_kernel::Kernel::boot().await?;
    ui::step(&format!("booted in {}ms", boot_report.total_millis));

    let mut registry = build_registry(cwd.clone());
    let plugin_caps = load_plugins(&mut registry, cwd.join("examples").join("plugins")).await;

    let session_id = Uuid::new_v4();
    let intent_engine = IntentEngine::new(kernel.event_bus.clone());
    let intent = intent_engine.intake(goal_hint, IntentSource::Cli, session_id)?;

    let mut known: Vec<String> = KNOWN_CAPABILITIES.iter().map(|s| s.to_string()).collect();
    known.extend(plugin_caps);

    let discovery = Arc::new(StaticCapabilityDiscovery::new(known));
    let policy = PolicyEvaluator::new(PolicyConstraints::default());
    let pipeline = PlanningPipeline::new(discovery, policy);
    let ctx = PlanningContext { internet_available: true, working_directory: cwd.to_string_lossy().to_string(), ..Default::default() };

    let plan = pipeline.plan(&intent, &ctx, vec![required_capability.to_string()]).await?;
    ui::step(&format!("plan ready ({} step(s), risk {})", plan.steps.len(), plan.risk_score.0));

    let tasks: Vec<Task> = plan.steps.iter()
        .map(|s| Task::new(format!("step-{}", s.order), s.description.clone(), s.required_capability.clone()))
        .collect();
    let graph = TaskGraph::new(tasks);
    let mut workflow = Workflow::from_plan(&plan, graph);
    workflow.validate()?;

    let executor = Executor::new(registry, kernel.event_bus.clone());

    for task in workflow.graph.tasks.values() {
        let mut task_params = params.clone();
        task_params.entry("cwd".to_string()).or_insert_with(|| serde_json::json!(cwd.to_string_lossy()));

        let result = executor.execute(task, task_params).await?;
        match result.outcome {
            orbvynx_executor::TaskOutcome::Success => {
                ui::success(&format!("{} succeeded", task.id));
                if let Some(output) = result.output {
                    ui::output(&output.to_string());
                }
            }
            _ => {
                ui::failure(&format!("{} failed", task.id));
                if let Some(err) = result.error {
                    ui::output(&err);
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Status => {
            let (kernel, report) = orbvynx_kernel::Kernel::boot().await?;
            ui::header("ORBVYNX Status");
            ui::field("Boot time", &format!("{}ms", report.total_millis));
            ui::field("Registered modules", &kernel.module_registry.list().len().to_string());
            ui::field("Event bus subscribers", &kernel.event_bus.subscriber_count().to_string());
        }

        Commands::Run { goal } => {
            let goal_str = goal.join(" ");
            if goal_str.trim().is_empty() {
                anyhow::bail!("provide a goal, e.g. 'orbvynx run build my app'");
            }
            run_capability("git.status", HashMap::new(), &goal_str).await?;
        }

        Commands::Git { action } => match action {
            GitAction::Status => run_capability("git.status", HashMap::new(), "git status").await?,
            GitAction::Commit { message } => {
                let mut params = HashMap::new();
                params.insert("message".to_string(), serde_json::json!(message));
                run_capability("git.commit", params, "git commit").await?;
            }
            GitAction::Push => run_capability("git.push", HashMap::new(), "git push").await?,
        },

        Commands::Plugin { action } => match action {
            PluginAction::List => {
                let cwd = std::env::current_dir()?;
                let mut plugin_registry = PluginRegistry::new();
                let count = plugin_registry.load_from_directory(cwd.join("examples").join("plugins")).await.unwrap_or(0);
                ui::header("Plugins");
                if count == 0 {
                    ui::info("none found in ./examples/plugins");
                } else {
                    for name in plugin_registry.list() {
                        if let Some(plugin) = plugin_registry.get(&name) {
                            ui::field(&plugin.manifest.name, &format!("{} — {}", plugin.manifest.capability_name, plugin.manifest.description));
                        }
                    }
                }
            }
        },

        Commands::Hash { text, file } => {
            let mut params = HashMap::new();
            if let Some(path) = file {
                params.insert("path".to_string(), serde_json::json!(path));
            } else if let Some(t) = text {
                params.insert("text".to_string(), serde_json::json!(t));
            } else {
                anyhow::bail!("provide either text or --file <path>");
            }
            run_capability("hash.sha256", params, "hash").await?;
        }

        Commands::Http { action } => match action {
            HttpAction::Get { url } => {
                let mut params = HashMap::new();
                params.insert("url".to_string(), serde_json::json!(url));
                run_capability("http.get", params, "http get").await?;
            }
        },

        Commands::Encode { action } => match action {
            EncodeAction::Base64 { text } => {
                let mut params = HashMap::new();
                params.insert("text".to_string(), serde_json::json!(text));
                run_capability("text.base64_encode", params, "encode").await?;
            }
            EncodeAction::Base64Decode { text } => {
                let mut params = HashMap::new();
                params.insert("text".to_string(), serde_json::json!(text));
                run_capability("text.base64_decode", params, "decode").await?;
            }
            EncodeAction::Url { text } => {
                let mut params = HashMap::new();
                params.insert("text".to_string(), serde_json::json!(text));
                run_capability("text.url_encode", params, "url encode").await?;
            }
        },

        Commands::Fs { action } => match action {
            FsAction::List { path, depth } => {
                let mut params = HashMap::new();
                params.insert("path".to_string(), serde_json::json!(path));
                params.insert("max_depth".to_string(), serde_json::json!(depth));
                run_capability("filesystem.list_directory", params, "list directory").await?;
            }
            FsAction::Exists { path } => {
                let mut params = HashMap::new();
                params.insert("path".to_string(), serde_json::json!(path));
                run_capability("filesystem.exists", params, "check exists").await?;
            }
        },

        Commands::Json { action } => match action {
            JsonAction::Parse { text } => {
                let mut params = HashMap::new();
                params.insert("text".to_string(), serde_json::json!(text));
                run_capability("json.parse", params, "parse json").await?;
            }
            JsonAction::Query { text, path } => {
                let value: serde_json::Value = serde_json::from_str(&text)
                    .map_err(|e| anyhow::anyhow!("invalid JSON: {e}"))?;
                let mut params = HashMap::new();
                params.insert("value".to_string(), value);
                params.insert("path".to_string(), serde_json::json!(path));
                run_capability("json.query", params, "query json").await?;
            }
        },

        Commands::Sys { action } => match action {
            SysAction::Uuid => run_capability("system.uuid_generate", HashMap::new(), "generate uuid").await?,
            SysAction::Time => run_capability("system.current_time", HashMap::new(), "current time").await?,
            SysAction::Env { key } => {
                let mut params = HashMap::new();
                params.insert("key".to_string(), serde_json::json!(key));
                run_capability("system.env_get", params, "env get").await?;
            }
        },
    }

    Ok(())
}
