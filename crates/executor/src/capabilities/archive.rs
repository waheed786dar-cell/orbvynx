use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use std::fs::File;
use std::io::{Read, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

pub struct ZipCompressCapability;

#[async_trait]
impl Capability for ZipCompressCapability {
    fn name(&self) -> &str {
        "archive.compress"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let source = input.params.get("source_path").and_then(|v| v.as_str())
            .ok_or("missing 'source_path' parameter")?.to_string();
        let dest = input.params.get("dest_path").and_then(|v| v.as_str())
            .ok_or("missing 'dest_path' parameter")?.to_string();

        tokio::task::spawn_blocking(move || -> Result<u64, String> {
            let mut source_file = File::open(&source).map_err(|e| format!("failed to open source: {e}"))?;
            let mut contents = Vec::new();
            source_file.read_to_end(&mut contents).map_err(|e| format!("failed to read source: {e}"))?;

            let file_name = std::path::Path::new(&source)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file")
                .to_string();

            let zip_file = File::create(&dest).map_err(|e| format!("failed to create archive: {e}"))?;
            let mut writer = ZipWriter::new(zip_file);
            let options = SimpleFileOptions::default();
            writer.start_file(file_name, options).map_err(|e| format!("failed to start zip entry: {e}"))?;
            writer.write_all(&contents).map_err(|e| format!("failed to write zip data: {e}"))?;
            writer.finish().map_err(|e| format!("failed to finalize archive: {e}"))?;

            Ok(contents.len() as u64)
        })
        .await
        .map_err(|e| format!("task join error: {e}"))??;

        Ok(CapabilityOutput { data: serde_json::json!({ "archive_created": true }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn compresses_a_real_file() {
        let dir = std::env::temp_dir();
        let source = dir.join("orbvynx_zip_test_source.txt");
        let dest = dir.join("orbvynx_zip_test_output.zip");

        tokio::fs::write(&source, "hello archive").await.unwrap();

        let cap = ZipCompressCapability;
        let mut params = HashMap::new();
        params.insert("source_path".to_string(), serde_json::json!(source.to_str().unwrap()));
        params.insert("dest_path".to_string(), serde_json::json!(dest.to_str().unwrap()));

        let result = cap.invoke(CapabilityInput { params }).await;
        assert!(result.is_ok());
        assert!(dest.exists());

        let _ = tokio::fs::remove_file(&source).await;
        let _ = tokio::fs::remove_file(&dest).await;
    }
}
