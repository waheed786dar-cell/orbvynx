use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CapabilityInput {
    pub params: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct CapabilityOutput {
    pub data: Value,
}

#[async_trait]
pub trait Capability: Send + Sync {
    fn name(&self) -> &str;
    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String>;
}

#[derive(Default)]
pub struct CapabilityRegistry {
    capabilities: HashMap<String, Arc<dyn Capability>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, capability: Arc<dyn Capability>) {
        self.capabilities.insert(capability.name().to_string(), capability);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn Capability>> {
        self.capabilities.get(name).cloned()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.capabilities.contains_key(name)
    }
}

pub struct EchoCapability;

#[async_trait]
impl Capability for EchoCapability {
    fn name(&self) -> &str {
        "test.echo"
    }

    async fn invoke(&self, input: CapabilityInput) -> Result<CapabilityOutput, String> {
        Ok(CapabilityOutput { data: serde_json::json!(input.params) })
    }
}

pub struct FailingCapability;

#[async_trait]
impl Capability for FailingCapability {
    fn name(&self) -> &str {
        "test.fail"
    }

    async fn invoke(&self, _input: CapabilityInput) -> Result<CapabilityOutput, String> {
        Err("simulated capability failure".to_string())
    }
}
