use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};

pub struct Base64EncodeCapability;

#[async_trait]
impl Capability for Base64EncodeCapability {
    fn name(&self) -> &str {
        "text.base64_encode"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;
        Ok(CapabilityOutput { data: serde_json::json!({ "encoded": STANDARD.encode(text) }) })
    }
}

pub struct Base64DecodeCapability;

#[async_trait]
impl Capability for Base64DecodeCapability {
    fn name(&self) -> &str {
        "text.base64_decode"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;
        let bytes = STANDARD.decode(text).map_err(|e| format!("invalid base64: {e}"))?;
        let decoded = String::from_utf8(bytes).map_err(|e| format!("decoded bytes are not valid utf8: {e}"))?;
        Ok(CapabilityOutput { data: serde_json::json!({ "decoded": decoded }) })
    }
}

pub struct UrlEncodeCapability;

#[async_trait]
impl Capability for UrlEncodeCapability {
    fn name(&self) -> &str {
        "text.url_encode"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;
        Ok(CapabilityOutput { data: serde_json::json!({ "encoded": urlencoding::encode(text) }) })
    }
}

pub struct RegexMatchCapability;

#[async_trait]
impl Capability for RegexMatchCapability {
    fn name(&self) -> &str {
        "text.regex_match"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let pattern = input.params.get("pattern").and_then(|v| v.as_str()).ok_or("missing 'pattern' parameter")?;
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;

        let re = regex::Regex::new(pattern).map_err(|e| format!("invalid regex: {e}"))?;
        let matches: Vec<String> = re.find_iter(text).map(|m| m.as_str().to_string()).collect();
        Ok(CapabilityOutput { data: serde_json::json!({ "matches": matches }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn base64_roundtrip() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("hello orbvynx"));
        let encoded = Base64EncodeCapability.invoke(CapabilityInput { params }).await.unwrap();
        let encoded_str = encoded.data["encoded"].as_str().unwrap().to_string();

        let mut params2 = HashMap::new();
        params2.insert("text".to_string(), serde_json::json!(encoded_str));
        let decoded = Base64DecodeCapability.invoke(CapabilityInput { params: params2 }).await.unwrap();
        assert_eq!(decoded.data["decoded"], "hello orbvynx");
    }

    #[tokio::test]
    async fn url_encode_escapes_special_chars() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("a b&c"));
        let output = UrlEncodeCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["encoded"], "a%20b%26c");
    }

    #[tokio::test]
    async fn regex_match_finds_all_occurrences() {
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), serde_json::json!(r"\d+"));
        params.insert("text".to_string(), serde_json::json!("order 12 has 3 items"));
        let output = RegexMatchCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["matches"], serde_json::json!(["12", "3"]));
    }

    #[tokio::test]
    async fn invalid_regex_is_rejected() {
        let mut params = HashMap::new();
        params.insert("pattern".to_string(), serde_json::json!("[unclosed"));
        params.insert("text".to_string(), serde_json::json!("test"));
        assert!(RegexMatchCapability.invoke(CapabilityInput { params }).await.is_err());
    }
}
