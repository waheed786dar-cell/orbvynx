//! Session Manager (Architecture Bible, Part 2A §14).
//!
//! Every execution happens inside a Session — the unit that carries
//! environment, platform, and permission context through the rest
//! of the pipeline (Intent -> Planner -> Workflow -> Executor).

use crate::identity::Identity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Linux,
    Android,
    Windows,
    MacOs,
    Bsd,
    Unknown,
}

impl Platform {
    /// Detects the current platform. Termux presents itself as
    /// `target_os = "linux"` to `cfg!`, so we check for the
    /// `TERMUX_VERSION` environment variable first to distinguish
    /// real Linux from Android/Termux.
    pub fn detect() -> Self {
        if std::env::var("TERMUX_VERSION").is_ok() {
            Platform::Android
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else if cfg!(target_os = "windows") {
            Platform::Windows
        } else if cfg!(target_os = "macos") {
            Platform::MacOs
        } else if cfg!(target_os = "freebsd") {
            Platform::Bsd
        } else {
            Platform::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub identity: Identity,
    pub started_at: DateTime<Utc>,
    pub platform: Platform,
    pub working_directory: String,
    pub user: Option<String>,
    pub permissions: Vec<String>,
}

impl Session {
    pub fn new(working_directory: impl Into<String>) -> Self {
        Self {
            identity: Identity::new(),
            started_at: Utc::now(),
            platform: Platform::detect(),
            working_directory: working_directory.into(),
            user: std::env::var("USER").ok(),
            permissions: Vec::new(),
        }
    }

    pub fn id(&self) -> Uuid {
        self.identity.id.0
    }

    pub fn has_permission(&self, capability: &str) -> bool {
        self.permissions.iter().any(|p| p == capability)
    }

    pub fn grant(&mut self, capability: impl Into<String>) {
        self.permissions.push(capability.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_permission_check() {
        let mut session = Session::new("/home/claude");
        assert!(!session.has_permission("filesystem.write"));
        session.grant("filesystem.write");
        assert!(session.has_permission("filesystem.write"));
    }
}
