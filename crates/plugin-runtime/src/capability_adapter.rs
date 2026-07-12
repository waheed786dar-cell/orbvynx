use crate::plugin::PluginProcess;
use async_trait::async_trait;
use orbvynx_executor::{Capability, CapabilityInput, CapabilityOutput};
use std::sync::Arc;

pub struct PluginCapability {
    plugin: Arc<PluginProcess>,
}

impl PluginCapability {
    pub fn new(plugin: Arc<PluginProcess>) -> Self {
        Self { plugin }
    }
}

#[async_trait]
impl Capability for PluginCapability {
    fn name(&self) -> &str {
        &self.plugin.manifest.capability_name
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        let payload = serde_json::json!(input.params);
        let result = self.plugin.invoke(payload).await.map_err(|e| e.to_string())?;
        Ok(CapabilityOutput { data: result })
    }
}
