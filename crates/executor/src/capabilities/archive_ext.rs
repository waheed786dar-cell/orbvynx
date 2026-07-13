use crate::capability::{Capability, CapabilityInput, CapabilityOutput};
use async_trait::async_trait;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::io::Read;

pub struct GzipCompressCapability;

#[async_trait]
impl Capability for GzipCompressCapability {
    fn name(&self) -> &str {
        "archive.gzip_compress"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let source = input.params.get("source_path").and_then(|v| v.as_str())
            .ok_or("missing 'source_path' parameter")?.to_string();
        let dest = input.params.get("dest_path").and_then(|v| v.as_str())
            .ok_or("missing 'dest_path' parameter")?.to_string();

        tokio::task::spawn_blocking(move || -> Result<(), String> {
            let mut source_file = File::open(&source).map_err(|e| format!("failed to open source: {e}"))?;
            let mut contents = Vec::new();
            source_file.read_to_end(&mut contents).map_err(|e| format!("failed to read source: {e}"))?;

            let dest_file = File::create(&dest).map_err(|e| format!("failed to create dest: {e}"))?;
            let mut encoder = GzEncoder::new(dest_file, Compression::default());
            std::io::Write::write_all(&mut encoder, &contents).map_err(|e| format!("failed to write gzip data: {e}"))?;
            encoder.finish().map_err(|e| format!("failed to finalize gzip: {e}"))?;
            Ok(())
        })
        .await
        .map_err(|e| format!("task join error: {e}"))??;

        Ok(CapabilityOutput { data: serde_json::json!({ "compressed": true }) })
    }
}

pub struct MimeGuessCapability;

#[async_trait]
impl Capability for MimeGuessCapability {
    fn name(&self) -> &str {
        "filesystem.mime_guess"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let path = input.params.get("path").and_then(|v| v.as_str()).ok_or("missing 'path' parameter")?;
        let guess = mime_guess::from_path(path).first_or_octet_stream();
        Ok(CapabilityOutput { data: serde_json::json!({ "mime_type": guess.to_string() }) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn gzip_compresses_a_real_file() {
        let dir = std::env::temp_dir();
        let source = dir.join("orbvynx_gzip_test_source.txt");
        let dest = dir.join("orbvynx_gzip_test_output.gz");
        tokio::fs::write(&source, "hello gzip").await.unwrap();

        let mut params = HashMap::new();
        params.insert("source_path".to_string(), serde_json::json!(source.to_str().unwrap()));
        params.insert("dest_path".to_string(), serde_json::json!(dest.to_str().unwrap()));

        let result = GzipCompressCapability.invoke(CapabilityInput { params }).await;
        assert!(result.is_ok());
        assert!(dest.exists());

        let _ = tokio::fs::remove_file(&source).await;
        let _ = tokio::fs::remove_file(&dest).await;
    }

    #[tokio::test]
    async fn mime_guess_detects_common_types() {
        let mut params = HashMap::new();
        params.insert("path".to_string(), serde_json::json!("file.json"));
        let output = MimeGuessCapability.invoke(CapabilityInput { params }).await.unwrap();
        assert_eq!(output.data["mime_type"], "application/json");
    }
}
