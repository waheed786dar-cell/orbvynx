use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use tokio::process::Command;

pub struct GradleBuildCapability;

#[async_trait]
impl Capability for GradleBuildCapability {
    fn name(&self) -> &str {
        "android.gradle_build"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let cwd = input.params.get("cwd")
            .and_then(|v| v.as_str())
            .ok_or("missing 'cwd' parameter")?;
        let task = input.params.get("task")
            .and_then(|v| v.as_str())
            .unwrap_or("assembleDebug");

        let gradlew = format!("{cwd}/gradlew");
        let output = Command::new(&gradlew)
            .arg(task)
            .current_dir(cwd)
            .output()
            .await
            .map_err(|e| format!("failed to spawn gradlew: {e}"))?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(CapabilityOutput {
            data: serde_json::json!({ "output": String::from_utf8_lossy(&output.stdout) }),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn missing_cwd_is_rejected() {
        let cap = GradleBuildCapability;
        let result = cap.invoke(CapabilityInput { params: HashMap::new() }).await;
        assert!(result.is_err());
    }
}
