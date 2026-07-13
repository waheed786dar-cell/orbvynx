use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;

pub struct GlobMatchCapability;

#[async_trait]
impl Capability for GlobMatchCapability {
    fn name(&self) -> &str {
        "filesystem.glob_match"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let pattern = input.params.get("pattern").and_then(|v| v.as_str()).ok_or("missing 'pattern' parameter")?.to_string();

        let matches: Result<Vec<String>, String> = tokio::task::spawn_blocking(move || {
            glob::glob(&pattern)
                .map_err(|e| format!("invalid glob pattern: {e}"))?
                .filter_map(|entry| entry.ok())
                .map(|path| Ok(path.display().to_string()))
                .collect()
        })
        .await
        .map_err(|e| format!("task join error: {e}"))?;

        let matches = matches?;
        Ok(CapabilityOutput { data: serde_json::json!({ "matches": matches }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn glob_matches_cargo_toml_files() {
        // Crate ki directory ka absolute path lein taake sandbox environment (Termux) mein fail na ho
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let pattern = format!("{}/Cargo.toml", manifest_dir);

        let mut params = HashMap::new();
        params.insert("pattern".to_string(), serde_json::json!(pattern));
        let output = GlobMatchCapability.invoke(CapabilityInput { params }).await.unwrap();
        let matches = output.data["matches"].as_array().unwrap();
        assert!(!matches.is_empty());
    }

    #[tokio::test]
    async fn invalid_pattern_is_rejected() {
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), serde_json::json!("[invalid"));
        assert!(GlobMatchCapability.invoke(CapabilityInput { params }).await.is_err());
    }
}
