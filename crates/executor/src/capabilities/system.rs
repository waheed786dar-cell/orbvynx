use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use uuid::Uuid;

pub struct UuidGenerateCapability;

#[async_trait]
impl Capability for UuidGenerateCapability {
    fn name(&self) -> &str {
        "system.uuid_generate"
    }

    async fn invoke(&self, _input: CapabilityInput) -> Result<CapabilityOutput, String> {
        Ok(CapabilityOutput { data: serde_json::json!({ "uuid": Uuid::new_v4().to_string() }) })
    }
}

pub struct EnvGetCapability;

#[async_trait]
impl Capability for EnvGetCapability {
    fn name(&self) -> &str {
        "system.env_get"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let key = input.params.get("key").and_then(|v| v.as_str()).ok_or("missing 'key' parameter")?;
        let value = std::env::var(key).unwrap_or_default();
        Ok(CapabilityOutput { data: serde_json::json!({ "key": key, "value": value }) })
    }
}

pub struct CurrentTimeCapability;

#[async_trait]
impl Capability for CurrentTimeCapability {
    fn name(&self) -> &str {
        "system.current_time"
    }

    async fn invoke(&self, _input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let now = chrono::Utc::now();
        Ok(CapabilityOutput { data: serde_json::json!({ "iso8601": now.to_rfc3339(), "unix_seconds": now.timestamp() }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn uuid_generate_returns_valid_uuid() {
        let output = UuidGenerateCapability.invoke(CapabilityInput { params: HashMap::new() }).await.unwrap();
        let uuid_str = output.data["uuid"].as_str().unwrap();
        assert!(Uuid::parse_str(uuid_str).is_ok());
    }

    #[tokio::test]
    async fn env_get_missing_var_returns_empty() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), serde_json::json!("ORBVYNX_NONEXISTENT_VAR_XYZ"));
        let output = EnvGetCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["value"], "");
    }

    #[tokio::test]
    async fn current_time_returns_valid_iso8601() {
        let output = CurrentTimeCapability.invoke(CapabilityInput { params: HashMap::new() }).await.unwrap();
        assert!(output.data["iso8601"].as_str().unwrap().contains('T'));
    }
}
