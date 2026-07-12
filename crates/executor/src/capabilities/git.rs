use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use tokio::process::Command;

async fn run_git(args: &[&str], cwd: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .await
        .map_err(|e| format!("failed to spawn git: {e}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn cwd_from(input: &CapabilityInput) -> Result<String, String> {
    input.params.get("cwd")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or("missing 'cwd' parameter".to_string())
}

pub struct GitStatusCapability;

#[async_trait]
impl Capability for GitStatusCapability {
    fn name(&self) -> &str {
        "git.status"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let cwd = cwd_from(&input)?;
        let stdout = run_git(&["status", "--porcelain"], &cwd).await?;
        Ok(CapabilityOutput { data: serde_json::json!({ "status": stdout }) })
    }
}

pub struct GitCommitCapability;

#[async_trait]
impl Capability for GitCommitCapability {
    fn name(&self) -> &str {
        "git.commit"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let cwd = cwd_from(&input)?;
        let message = input.params.get("message")
            .and_then(|v| v.as_str())
            .ok_or("missing 'message' parameter")?;

        run_git(&["add", "."], &cwd).await?;
        let stdout = run_git(&["commit", "-m", message], &cwd).await?;
        Ok(CapabilityOutput { data: serde_json::json!({ "output": stdout }) })
    }
}

pub struct GitPushCapability;

#[async_trait]
impl Capability for GitPushCapability {
    fn name(&self) -> &str {
        "git.push"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let cwd = cwd_from(&input)?;
        let stdout = run_git(&["push"], &cwd).await?;
        Ok(CapabilityOutput { data: serde_json::json!({ "output": stdout }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    #[ignore = "spawns real git subprocess; run explicitly with -- --ignored"]
    async fn git_status_on_current_repo_succeeds() {
        let cap = GitStatusCapability;
        let mut params = HashMap::new();
        params.insert("cwd".to_string(), serde_json::json!("."));

        let result = cap.invoke(CapabilityInput { params }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn missing_cwd_param_is_rejected() {
        let cap = GitStatusCapability;
        let result = cap.invoke(CapabilityInput { params: HashMap::new() }).await;
        assert!(result.is_err());
    }
}
