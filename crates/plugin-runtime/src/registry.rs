use crate::error::PluginResult;
use crate::plugin::PluginProcess;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Default)]
pub struct PluginRegistry {
    plugins: HashMap<String, Arc<PluginProcess>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn load_from_directory(&mut self, dir: PathBuf) -> PluginResult<usize> {
        let mut count = 0;
        let mut entries = match tokio::fs::read_dir(&dir).await {
            Ok(e) => e,
            Err(_) => return Ok(0),
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Ok(plugin) = PluginProcess::load(path).await {
                    self.plugins.insert(plugin.manifest.name.clone(), Arc::new(plugin));
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    pub fn register(&mut self, plugin: Arc<PluginProcess>) {
        self.plugins.insert(plugin.manifest.name.clone(), plugin);
    }

    pub fn get(&self, name: &str) -> Option<Arc<PluginProcess>> {
        self.plugins.get(name).cloned()
    }

    pub fn list(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}
