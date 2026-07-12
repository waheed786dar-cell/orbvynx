use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use sha2::{Digest, Sha256};

pub struct Sha256Capability;

#[async_trait]
impl Capability for Sha256Capability {
    fn name(&self) -> &str {
        "hash.sha256"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        if let Some(path) = input.params.get("path").and_then(|v| v.as_str()) {
            let bytes = tokio::fs::read(path).await.map_err(|e| format!("failed to read '{path}': {e}"))?;
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let digest = hex::encode(hasher.finalize());
            return Ok(CapabilityOutput { data: serde_json::json!({ "sha256": digest }) });
        }

        if let Some(text) = input.params.get("text").and_then(|v| v.as_str()) {
            let mut hasher = Sha256::new();
            hasher.update(text.as_bytes());
            let digest = hex::encode(hasher.finalize());
            return Ok(CapabilityOutput { data: serde_json::json!({ "sha256": digest }) });
        }

        Err("provide either 'path' or 'text' parameter".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn hashes_text_correctly() {
        let cap = Sha256Capability;
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("hello"));
        let output = cap.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(
            output.data["sha256"],
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[tokio::test]
    async fn missing_both_params_is_rejected() {
        let cap = Sha256Capability;
        let result = cap.invoke(CapabilityInput { params: HashMap::new() }).await;
        assert!(result.is_err());
    }
}
