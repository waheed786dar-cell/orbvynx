#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SandboxLevel {
    None,
    Process,
    FilesystemScoped,
    NetworkScoped,
    Full,
}

pub struct SandboxPolicy {
    pub level: SandboxLevel,
    pub allowed_paths: Vec<String>,
    pub allowed_hosts: Vec<String>,
}

impl SandboxPolicy {
    pub fn trusted() -> Self {
        Self { level: SandboxLevel::None, allowed_paths: vec![], allowed_hosts: vec![] }
    }

    pub fn plugin_default() -> Self {
        Self { level: SandboxLevel::Full, allowed_paths: vec![], allowed_hosts: vec![] }
    }
}
