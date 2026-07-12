pub mod capability_adapter;
pub mod error;
pub mod manifest;
pub mod plugin;
pub mod registry;

pub use capability_adapter::PluginCapability;
pub use error::{PluginError, PluginResult};
pub use manifest::PluginManifest;
pub use plugin::PluginProcess;
pub use registry::PluginRegistry;
