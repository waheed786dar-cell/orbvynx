use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct FilesystemReadCapability {
    pub allowed_paths: Vec<PathBuf>,
}

impl FilesystemReadCapability {
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    fn is_allowed(&self, path: &Path) -> bool {
        self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
    }
}

#[async_trait]
impl Capability for FilesystemReadCapability {
    fn name(&self) -> &str {
        "filesystem.read_file"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path_str = input.params.get("path")
            .and_then(|v| v.as_str())
            .ok_or("missing 'path' parameter")?;
        let path = PathBuf::from(path_str);

        if !self.is_allowed(&path) {
            return Err(format!("path '{path_str}' is outside sandbox allowed_paths"));
        }

        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| format!("failed to read '{path_str}': {e}"))?;

        Ok(CapabilityOutput { data: serde_json::json!({ "content": content }) })
    }
}

pub struct FilesystemWriteCapability {
    pub allowed_paths: Vec<PathBuf>,
}

impl FilesystemWriteCapability {
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    fn is_allowed(&self, path: &Path) -> bool {
        self.allowed_paths.iter().any(|allowed| path.starts_with(allowed))
    }
}

#[async_trait]
impl Capability for FilesystemWriteCapability {
    fn name(&self) -> &str {
        "filesystem.write_file"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path_str = input.params.get("path")
            .and_then(|v| v.as_str())
            .ok_or("missing 'path' parameter")?;
        let content = input.params.get("content")
            .and_then(|v| v.as_str())
            .ok_or("missing 'content' parameter")?;
        let path = PathBuf::from(path_str);

        if !self.is_allowed(&path) {
            return Err(format!("path '{path_str}' is outside sandbox allowed_paths"));
        }

        tokio::fs::write(&path, content).await
            .map_err(|e| format!("failed to write '{path_str}': {e}"))?;

        Ok(CapabilityOutput { data: serde_json::json!({ "bytes_written": content.len() }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile_shim::temp_dir;

    mod tempfile_shim {
        use std::path::PathBuf;
        pub fn temp_dir() -> PathBuf {
            std::env::temp_dir()
        }
    }

    #[tokio::test]
    async fn write_then_read_roundtrip() {
        let dir = temp_dir();
        let write_cap = FilesystemWriteCapability::new(vec![dir.clone()]);
        let read_cap = FilesystemReadCapability::new(vec![dir.clone()]);

        let file_path = dir.join("orbvynx_test_file.txt");
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(file_path.to_str().unwrap()));
        params.insert("content".to_string(), serde_json::json!("hello orbvynx"));

        write_cap.invoke(CapabilityInput { params }).await.unwrap();

        let mut read_params = HashMap::new();
        read_params.insert("path".to_string(), serde_json::json!(file_path.to_str().unwrap()));
        let output = read_cap.invoke(CapabilityInput { params: read_params }).await.unwrap();

        assert_eq!(output.data["content"], "hello orbvynx");

        let _ = tokio::fs::remove_file(&file_path).await;
    }

    #[tokio::test]
    async fn path_outside_sandbox_is_rejected() {
        let read_cap = FilesystemReadCapability::new(vec![PathBuf::from("/only/this/dir")]);
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!("/etc/passwd"));

        let result = read_cap.invoke(CapabilityInput { params }).await;
        assert!(result.is_err());
    }
}
