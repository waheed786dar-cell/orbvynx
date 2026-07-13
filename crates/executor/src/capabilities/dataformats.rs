use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;

pub struct TomlToJsonCapability;

#[async_trait]
impl Capability for TomlToJsonCapability {
    fn name(&self) -> &str {
        "convert.toml_to_json"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;
        let value: toml::Value = toml::from_str(text).map_err(|e| format!("invalid TOML: {e}"))?;
        let json = serde_json::to_value(&value).map_err(|e| format!("conversion failed: {e}"))?;
        Ok(CapabilityOutput { data: serde_json::json!({ "json": json }) })
    }
}

pub struct YamlToJsonCapability;

#[async_trait]
impl Capability for YamlToJsonCapability {
    fn name(&self) -> &str {
        "convert.yaml_to_json"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;
        let value: serde_yaml::Value = serde_yaml::from_str(text).map_err(|e| format!("invalid YAML: {e}"))?;
        let json = serde_json::to_value(&value).map_err(|e| format!("conversion failed: {e}"))?;
        Ok(CapabilityOutput { data: serde_json::json!({ "json": json }) })
    }
}

pub struct JsonToYamlCapability;

#[async_trait]
impl Capability for JsonToYamlCapability {
    fn name(&self) -> &str {
        "convert.json_to_yaml"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let value = input.params.get("value").ok_or("missing 'value' parameter")?;
        let yaml = serde_yaml::to_string(value).map_err(|e| format!("conversion failed: {e}"))?;
        Ok(CapabilityOutput { data: serde_json::json!({ "yaml": yaml }) })
    }
}

pub struct CsvParseCapability;

#[async_trait]
impl Capability for CsvParseCapability {
    fn name(&self) -> &str {
        "convert.csv_parse"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("text").and_then(|v| v.as_str()).ok_or("missing 'text' parameter")?;
        let mut reader = csv::Reader::from_reader(text.as_bytes());

        let headers: Vec<String> = reader.headers().map_err(|e| format!("invalid CSV headers: {e}"))?
            .iter().map(|s| s.to_string()).collect();

        let mut rows = Vec::new();
        for result in reader.records() {
            let record = result.map_err(|e| format!("invalid CSV row: {e}"))?;
            let row: serde_json::Map<String, serde_json::Value> = headers.iter()
                .zip(record.iter())
                .map(|(h, v)| (h.clone(), serde_json::json!(v)))
                .collect();
            rows.push(serde_json::Value::Object(row));
        }

        Ok(CapabilityOutput { data: serde_json::json!({ "headers": headers, "rows": rows }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn toml_to_json_converts_correctly() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("name = \"orbvynx\"\nversion = \"0.1.0\""));
        let output = TomlToJsonCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["json"]["name"], "orbvynx");
    }

    #[tokio::test]
    async fn yaml_to_json_converts_correctly() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("name: orbvynx\nversion: 0.1.0"));
        let output = YamlToJsonCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["json"]["name"], "orbvynx");
    }

    #[tokio::test]
    async fn csv_parse_extracts_headers_and_rows() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("name,age\nwaheed,25\nali,30"));
        let output = CsvParseCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["headers"], serde_json::json!(["name", "age"]));
        assert_eq!(output.data["rows"][0]["name"], "waheed");
    }

    #[tokio::test]
    async fn invalid_toml_is_rejected() {
        let mut params = HashMap::new();
        params.insert("text".to_string(), serde_json::json!("not = valid = toml = ="));
        assert!(TomlToJsonCapability.invoke(CapabilityInput { params }).await.is_err());
    }
}
