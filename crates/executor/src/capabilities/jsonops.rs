use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;

pub struct JsonParseCapability;

#[async_trait]
impl Capability for JsonParseCapability {
    fn name(&self) -> &str {
        "json.parse"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;
        let parsed: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("invalid JSON: {e}"))?;
        Ok(CapabilityOutput { data: serde_json::json!({ "parsed": parsed }) })
    }
}

pub struct JsonQueryCapability;

#[async_trait]
impl Capability for JsonQueryCapability {
    fn name(&self) -> &str {
        "json.query"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let value = input.params.get("value").ok_or("missing 'value' parameter")?;
        let path = input.params.get("path").and_then(|v| v.as_str()).ok_or("missing 'path' parameter")?;

        let mut current = value;
        for segment in path.split('.').filter(|s| !s.is_empty()) {
            current = current.get(segment).ok_or_else(|| format!("path segment '{segment}' not found"))?;
        }

        Ok(CapabilityOutput { data: serde_json::json!({ "result": current }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn parses_valid_json() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!(r#"{"a": 1, "b": [1,2,3]}"#));
        let output = JsonParseCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["parsed"]["a"], 1);
    }

    #[tokio::test]
    async fn invalid_json_is_rejected() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("{not valid json"));
        assert!(JsonParseCapability.invoke(CapabilityInput { params }).await.is_err());
    }

    #[tokio::test]
    async fn query_navigates_nested_path() {
        let mut params = HashMap::new();
        params.insert("value".to_string(), serde_json::json!({"user": {"name": "waheed"}}));
        params.insert("path".to_string(), serde_json::json!("user.name"));
        let output = JsonQueryCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["result"], "waheed");
    }
}
