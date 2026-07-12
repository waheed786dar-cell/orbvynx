use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;

pub struct HttpGetCapability;

#[async_trait]
impl Capability for HttpGetCapability {
    fn name(&self) -> &str {
        "http.get"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let url = input.params.get("url")
            .and_then(|v| v.as_str())
            .ok_or("missing 'url' parameter")?;

        let response = reqwest::get(url).await.map_err(|e| format!("request failed: {e}"))?;
        let status = response.status().as_u16();
        let body = response.text().await.map_err(|e| format!("failed to read body: {e}"))?;

        Ok(CapabilityOutput { data: serde_json::json!({ "status": status, "body": body }) })
    }
}

pub struct HttpPostCapability;

#[async_trait]
impl Capability for HttpPostCapability {
    fn name(&self) -> &str {
        "http.post"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let url = input.params.get("url")
            .and_then(|v| v.as_str())
            .ok_or("missing 'url' parameter")?;
        let body = input.params.get("body").cloned().unwrap_or(serde_json::json!({}));

        let client = reqwest::Client::new();
        let response = client.post(url).json(&body).send().await.map_err(|e| format!("request failed: {e}"))?;
        let status = response.status().as_u16();
        let response_body = response.text().await.map_err(|e| format!("failed to read body: {e}"))?;

        Ok(CapabilityOutput { data: serde_json::json!({ "status": status, "body": response_body }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    #[ignore = "requires network access; run explicitly with -- --ignored"]
    async fn http_get_fetches_real_url() {
        let cap = HttpGetCapability;
        let mut params = HashMap::new();
        params.insert("url".to_string(), serde_json::json!("https://httpbin.org/get"));
        let result = cap.invoke(CapabilityInput { params }).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn http_get_missing_url_is_rejected() {
        let cap = HttpGetCapability;
        let result = cap.invoke(CapabilityInput { params: HashMap::new() }).await;
        assert!(result.is_err());
    }
}
