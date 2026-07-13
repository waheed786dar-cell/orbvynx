use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use regex::Regex;

pub struct GradleDependencyListCapability;

#[async_trait]
impl Capability for GradleDependencyListCapability {
    fn name(&self) -> &str {
        "android.gradle_dependency_list"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str())
            .unwrap_or("app/build.gradle.kts").to_string();

        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| format!("failed to read '{path}': {e}"))?;

        let re = Regex::new(r#"(?:implementation|api|testImplementation|androidTestImplementation)\s*\(?\s*["']([^"']+)["']"#)
            .map_err(|e| format!("regex error: {e}"))?;

        let deps: Vec<String> = re.captures_iter(&content)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect();

        Ok(CapabilityOutput { data: serde_json::json!({ "dependency_count": deps.len(), "dependencies": deps }) })
    }
}

pub struct GradleVersionCheckCapability;

#[async_trait]
impl Capability for GradleVersionCheckCapability {
    fn name(&self) -> &str {
        "android.gradle_version_check"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str())
            .unwrap_or("gradle/wrapper/gradle-wrapper.properties").to_string();

        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| format!("failed to read '{path}': {e}"))?;

        let re = Regex::new(r"gradle-(\d+\.\d+(?:\.\d+)?)-").map_err(|e| format!("regex error: {e}"))?;
        let version = re.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());

        Ok(CapabilityOutput { data: serde_json::json!({ "gradle_version": version }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn extracts_dependencies_from_kts() {
        let path = std::env::temp_dir().join("orbvynx_test_build.gradle.kts");
        tokio::fs::write(&path, r#"
dependencies {
    implementation("androidx.core:core-ktx:1.12.0")
    implementation("com.squareup.retrofit2:retrofit:2.9.0")
    testImplementation("junit:junit:4.13.2")
}
"#).await.unwrap();

        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path.to_str().unwrap()));
        let output = GradleDependencyListCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["dependency_count"], 3);

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn extracts_gradle_wrapper_version() {
        let path = std::env::temp_dir().join("orbvynx_test_wrapper.properties");
        tokio::fs::write(&path, "distributionUrl=https\\://services.gradle.org/distributions/gradle-8.10-bin.zip").await.unwrap();

        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path.to_str().unwrap()));
        let output = GradleVersionCheckCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["gradle_version"], "8.10");

        let _ = tokio::fs::remove_file(&path).await;
    }
}
