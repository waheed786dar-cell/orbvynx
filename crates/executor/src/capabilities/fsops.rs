use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use walkdir::WalkDir;

pub struct ListDirectoryCapability;

#[async_trait]
impl Capability for ListDirectoryCapability {
    fn name(&self) -> &str {
        "filesystem.list_directory"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str()).ok_or("missing 'path' parameter")?.to_string();
        let max_depth = input.params.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

        let entries: Vec<String> = tokio::task::spawn_blocking(move || {
            WalkDir::new(&path)
                .max_depth(max_depth)
                .into_iter()
                .filter_map(|e| e.ok())
                .map(|e| e.path().display().to_string())
                .collect()
        })
        .await
        .map_err(|e| format!("task join error: {e}"))?;

        Ok(CapabilityOutput { data: serde_json::json!({ "entries": entries }) })
    }
}

pub struct FileExistsCapability;

#[async_trait]
impl Capability for FileExistsCapability {
    fn name(&self) -> &str {
        "filesystem.exists"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str()).ok_or("missing 'path' parameter")?;
        let exists = tokio::fs::try_exists(path).await.unwrap_or(false);
        Ok(CapabilityOutput { data: serde_json::json!({ "exists": exists }) })
    }
}

pub struct FileDeleteCapability {
    pub allowed_paths: Vec<std::path::PathBuf>,
}

impl FileDeleteCapability {
    pub fn new(allowed_paths: Vec<std::path::PathBuf>) -> Self {
        Self { allowed_paths }
    }
}

#[async_trait]
impl Capability for FileDeleteCapability {
    fn name(&self) -> &str {
        "filesystem.delete_file"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path_str = input.params.get("path").and_then(|v| v.as_str()).ok_or("missing 'path' parameter")?;
        let path = std::path::PathBuf::from(path_str);

        if !self.allowed_paths.iter().any(|allowed| path.starts_with(allowed)) {
            return Err(format!("path '{path_str}' is outside sandbox allowed_paths"));
        }

        tokio::fs::remove_file(&path).await.map_err(|e| format!("failed to delete '{path_str}': {e}"))?;
        Ok(CapabilityOutput { data: serde_json::json!({ "deleted": true }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn list_directory_returns_entries() {
        let dir = std::env::temp_dir();
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(dir.to_str().unwrap()));
        let output = ListDirectoryCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert!(output.data["entries"].as_array().unwrap().len() > 0);
    }

    #[tokio::test]
    async fn file_exists_detects_real_and_missing_files() {
        let dir = std::env::temp_dir();
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(dir.to_str().unwrap()));
        let output = FileExistsCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["exists"], true);

        let mut params2 = HashMap::new();
        params2.insert("path".to_string(), serde_json::json!("/definitely/does/not/exist/xyz"));
        let output2 = FileExistsCapability.invoke(CapabilityInput { params: params2 }).await.unwrap();
        assert_eq!(output2.data["exists"], false);
    }

    #[tokio::test]
    async fn delete_outside_sandbox_is_rejected() {
        let cap = FileDeleteCapability::new(vec![std::path::PathBuf::from("/only/this")]);
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!("/etc/passwd"));
        assert!(cap.invoke(CapabilityInput { params }).await.is_err());
    }
}
