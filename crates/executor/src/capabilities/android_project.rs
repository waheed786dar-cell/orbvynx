use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub struct ManifestParseCapability;

#[async_trait]
impl Capability for ManifestParseCapability {
    fn name(&self) -> &str {
        "android.manifest_parse"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str())
            .unwrap_or("app/src/main/AndroidManifest.xml").to_string();

        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| format!("failed to read manifest at '{path}': {e}"))?;

        let mut reader = Reader::from_str(&content);
        reader.config_mut().trim_text(true);

        let mut package = None;
        let mut permissions = Vec::new();
        let mut activities = Vec::new();
        let mut min_sdk = None;
        let mut target_sdk = None;
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Empty(e)) | Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    for attr in e.attributes().flatten() {
                        let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let value = attr.unescape_value().unwrap_or_default().to_string();

                        match (name.as_str(), key.as_str()) {
                            ("manifest", "package") => package = Some(value),
                            ("uses-permission", k) if k.ends_with("name") => permissions.push(value),
                            ("activity", k) if k.ends_with("name") => activities.push(value),
                            ("uses-sdk", k) if k.ends_with("minSdkVersion") => min_sdk = Some(value),
                            ("uses-sdk", k) if k.ends_with("targetSdkVersion") => target_sdk = Some(value),
                            _ => {}
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML parse error: {e}")),
                _ => {}
            }
            buf.clear();
        }

        Ok(CapabilityOutput { data: serde_json::json!({
            "package": package, "permissions": permissions, "activities": activities,
            "min_sdk": min_sdk, "target_sdk": target_sdk,
        })})
    }
}

pub struct ManifestPermissionCheckCapability;

#[async_trait]
impl Capability for ManifestPermissionCheckCapability {
    fn name(&self) -> &str {
        "android.manifest_permission_check"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str())
            .unwrap_or("app/src/main/AndroidManifest.xml").to_string();
        let dangerous = [
            "READ_CONTACTS", "WRITE_CONTACTS", "ACCESS_FINE_LOCATION", "ACCESS_COARSE_LOCATION",
            "READ_SMS", "SEND_SMS", "CAMERA", "RECORD_AUDIO", "READ_EXTERNAL_STORAGE",
            "WRITE_EXTERNAL_STORAGE", "READ_CALL_LOG", "CALL_PHONE",
        ];

        let content = tokio::fs::read_to_string(&path).await
            .map_err(|e| format!("failed to read manifest at '{path}': {e}"))?;

        let found: Vec<&str> = dangerous.iter().filter(|d| content.contains(*d)).copied().collect();

        Ok(CapabilityOutput { data: serde_json::json!({
            "dangerous_permissions_found": found,
            "count": found.len(),
        })})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const SAMPLE_MANIFEST: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="com.waheed.app">
    <uses-sdk android:minSdkVersion="24" android:targetSdkVersion="34" />
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.CAMERA" />
    <application>
        <activity android:name=".MainActivity" />
    </application>
</manifest>"#;

    #[tokio::test]
    async fn parses_manifest_fields_correctly() {
        let path = std::env::temp_dir().join("orbvynx_test_manifest.xml");
        tokio::fs::write(&path, SAMPLE_MANIFEST).await.unwrap();

        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path.to_str().unwrap()));
        let output = ManifestParseCapability.invoke(CapabilityInput { params }).await.unwrap();

        assert_eq!(output.data["package"], "com.waheed.app");
        assert_eq!(output.data["min_sdk"], "24");
        assert!(output.data["permissions"].as_array().unwrap().len() >= 2);

        let _ = tokio::fs::remove_file(&path).await;
    }

    #[tokio::test]
    async fn detects_dangerous_permissions() {
        let path = std::env::temp_dir().join("orbvynx_test_manifest2.xml");
        tokio::fs::write(&path, SAMPLE_MANIFEST).await.unwrap();

        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path.to_str().unwrap()));
        let output = ManifestPermissionCheckCapability.invoke(CapabilityInput { params }).await.unwrap();

        assert_eq!(output.data["count"], 1);
        let _ = tokio::fs::remove_file(&path).await;
    }
}
