use crate::error::{PluginError, PluginResult};
use crate::manifest::PluginManifest;
use std::path::PathBuf;
use tokio::process::Command;

pub struct PluginProcess {
    pub executable_path: PathBuf,
    pub manifest: PluginManifest,
}

impl PluginProcess {
    pub async fn load(executable_path: PathBuf) -> PluginResult<Self> {
        if !executable_path.exists() {
            return Err(PluginError::NotFound(executable_path.display().to_string()));
        }

        let output = Command::new(&executable_path)
            .arg("--orbvynx-manifest")
            .output()
            .await
            .map_err(|e| PluginError::SpawnFailed(executable_path.display().to_string(), e.to_string()))?;

        if !output.status.success() {
            return Err(PluginError::InvalidManifest(
                executable_path.display().to_string(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        let manifest: PluginManifest = serde_json::from_slice(&output.stdout).map_err(|e| {
            PluginError::InvalidManifest(executable_path.display().to_string(), e.to_string())
        })?;

        Ok(Self { executable_path, manifest })
    }

    pub async fn invoke(&self, input: serde_json::Value) -> PluginResult<serde_json::Value> {
        use tokio::io::AsyncWriteExt;
        use std::process::Stdio;

        let mut child = Command::new(&self.executable_path)
            .arg("--orbvynx-invoke")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| PluginError::SpawnFailed(self.manifest.name.clone(), e.to_string()))?;

        if let Some(mut stdin) = child.stdin.take() {
            let payload = serde_json::to_vec(&input).unwrap_or_default();
            let _ = stdin.write_all(&payload).await;
        }

        let output = child.wait_with_output().await
            .map_err(|e| PluginError::SpawnFailed(self.manifest.name.clone(), e.to_string()))?;

        if !output.status.success() {
            return Err(PluginError::ExecutionFailed(
                self.manifest.name.clone(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        serde_json::from_slice(&output.stdout)
            .map_err(|e| PluginError::InvalidOutput(self.manifest.name.clone(), e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_plugin_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/plugins/echo_plugin.sh")
    }

    #[tokio::test]
    async fn loads_manifest_from_real_plugin_script() {
        let plugin = PluginProcess::load(example_plugin_path()).await.unwrap();
        assert_eq!(plugin.manifest.name, "echo_plugin");
        assert_eq!(plugin.manifest.capability_name, "plugin.echo");
    }

    #[tokio::test]
    async fn invokes_plugin_and_gets_json_output() {
        let plugin = PluginProcess::load(example_plugin_path()).await.unwrap();
        let result = plugin.invoke(serde_json::json!({"hello": "world"})).await.unwrap();
        assert_eq!(result["received"]["hello"], "world");
    }

    #[tokio::test]
    async fn missing_plugin_file_is_rejected() {
        let result = PluginProcess::load(PathBuf::from("/nonexistent/plugin")).await;
        assert!(result.is_err());
    }
}
