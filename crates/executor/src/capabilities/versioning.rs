use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use semver::Version;

pub struct SemverParseCapability;

#[async_trait]
impl Capability for SemverParseCapability {
    fn name(&self) -> &str {
        "version.parse"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let text = input.params.get("version").and_then(|v| v.as_str()).ok_or("missing 'version' parameter")?;
        let version = Version::parse(text).map_err(|e| format!("invalid semver: {e}"))?;
        Ok(CapabilityOutput { data: serde_json::json!({
            "major": version.major, "minor": version.minor, "patch": version.patch,
            "pre": version.pre.to_string(), "build": version.build.to_string(),
        })})
    }
}

pub struct SemverCompareCapability;

#[async_trait]
impl Capability for SemverCompareCapability {
    fn name(&self) -> &str {
        "version.compare"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let a_str = input.params.get("a").and_then(|v| v.as_str()).ok_or("missing 'a' parameter")?;
        let b_str = input.params.get("b").and_then(|v| v.as_str()).ok_or("missing 'b' parameter")?;
        let a = Version::parse(a_str).map_err(|e| format!("invalid semver 'a': {e}"))?;
        let b = Version::parse(b_str).map_err(|e| format!("invalid semver 'b': {e}"))?;

        let ordering = match a.cmp(&b) {
            std::cmp::Ordering::Less => "less",
            std::cmp::Ordering::Equal => "equal",
            std::cmp::Ordering::Greater => "greater",
        };
        Ok(CapabilityOutput { data: serde_json::json!({ "result": ordering }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn parses_semver_correctly() {
        let mut params = HashMap::new();
        params.insert("version".to_string(), serde_json::json!("1.2.3"));
        let output = SemverParseCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["major"], 1);
        assert_eq!(output.data["minor"], 2);
        assert_eq!(output.data["patch"], 3);
    }

    #[tokio::test]
    async fn compares_versions_correctly() {
        let mut params = HashMap::new();
        params.insert("a".to_string(), serde_json::json!("1.0.0"));
        params.insert("b".to_string(), serde_json::json!("2.0.0"));
        let output = SemverCompareCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["result"], "less");
    }

    #[tokio::test]
    async fn invalid_version_is_rejected() {
        let mut params = HashMap::new();
        params.insert("version".to_string(), serde_json::json!("not-a-version"));
        assert!(SemverParseCapability.invoke(CapabilityInput { params }).await.is_err());
    }
}
