use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use std::fs::File;
use zip::ZipArchive;

pub struct ApkListEntriesCapability;

#[async_trait]
impl Capability for ApkListEntriesCapability {
    fn name(&self) -> &str {
        "android.apk_list_entries"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str()).ok_or("missing 'path' parameter")?.to_string();

        let entries: Result<Vec<String>, String> = tokio::task::spawn_blocking(move || {
            let file = File::open(&path).map_err(|e| format!("failed to open APK: {e}"))?;
            let mut archive = ZipArchive::new(file).map_err(|e| format!("not a valid APK/ZIP: {e}"))?;
            let mut names = Vec::new();
            for i in 0..archive.len() {
                let entry = archive.by_index(i).map_err(|e| format!("failed to read entry {i}: {e}"))?;
                names.push(entry.name().to_string());
            }
            Ok(names)
        })
        .await
        .map_err(|e| format!("task join error: {e}"))?;

        let entries = entries?;
        Ok(CapabilityOutput { data: serde_json::json!({ "entry_count": entries.len(), "entries": entries }) })
    }
}

pub struct ApkSizeReportCapability;

#[async_trait]
impl Capability for ApkSizeReportCapability {
    fn name(&self) -> &str {
        "android.apk_size_report"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str()).ok_or("missing 'path' parameter")?.to_string();

        let report: Result<serde_json::Value, String> = tokio::task::spawn_blocking(move || {
            let file = File::open(&path).map_err(|e| format!("failed to open APK: {e}"))?;
            let total_size = file.metadata().map_err(|e| format!("failed to stat APK: {e}"))?.len();
            let mut archive = ZipArchive::new(file).map_err(|e| format!("not a valid APK/ZIP: {e}"))?;

            let mut by_category: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
            for i in 0..archive.len() {
                let entry = archive.by_index(i).map_err(|e| format!("failed to read entry {i}: {e}"))?;
                let category = if entry.name().starts_with("res/") { "resources" }
                    else if entry.name().ends_with(".dex") { "dex_code" }
                    else if entry.name().starts_with("lib/") { "native_libs" }
                    else if entry.name().starts_with("assets/") { "assets" }
                    else { "other" };
                *by_category.entry(category.to_string()).or_insert(0) += entry.size();
            }

            Ok(serde_json::json!({ "total_bytes": total_size, "by_category_uncompressed_bytes": by_category }))
        })
        .await
        .map_err(|e| format!("task join error: {e}"))?;

        Ok(CapabilityOutput { data: report? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::capability::CapabilityInput;
    use std::collections::HashMap;
    use std::io::Write;

    fn make_test_apk(unique_suffix: &str) -> std::path::PathBuf {
        // Unique suffix lagane se parallel tests aik doosre ki file delete nahi karenge
        let filename = format!("orbvynx_test_sample_{}.apk", unique_suffix);
        let path = std::env::temp_dir().join(filename);
        let file = File::create(&path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("AndroidManifest.xml", options).unwrap();
        zip.write_all(b"<manifest/>").unwrap();
        zip.start_file("classes.dex", options).unwrap();
        zip.write_all(b"fake dex bytes").unwrap();
        zip.finish().unwrap();
        
        // Path ko absolute aur canonicalized bna kar return karein
        std::fs::canonicalize(path).unwrap()
    }

    #[tokio::test]
    async fn lists_apk_entries() {
        let path = make_test_apk("list");
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path.to_str().unwrap()));
        let output = ApkListEntriesCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["entry_count"], 2);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn size_report_categorizes_dex() {
        let path = make_test_apk("size");
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!(path.to_str().unwrap()));
        let output = ApkSizeReportCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert!(output.data["by_category_uncompressed_bytes"]["dex_code"].as_u64().unwrap() > 0);
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn invalid_apk_path_is_rejected() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!("/nonexistent/app.apk"));
        assert!(ApkListEntriesCapability.invoke(CapabilityInput { params }).await.is_err());
    }
}
