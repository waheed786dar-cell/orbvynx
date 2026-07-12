//! Universal Identity (Architecture Bible, Part 1 §12).
//!
//! Every object in ORBVYNX carries the same identity shape:
//! a global ID, version, creation time, owner, metadata and tags.
//! This module defines that shape once so every other crate reuses it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// A globally unique, stable identifier for any ORBVYNX object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub Uuid);

impl ObjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Universal identity block attached to every kernel-managed object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub id: ObjectId,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub owner: Option<String>,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
}

impl Identity {
    pub fn new() -> Self {
        Self {
            id: ObjectId::new(),
            version: 1,
            created_at: Utc::now(),
            owner: None,
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    pub fn with_owner(mut self, owner: impl Into<String>) -> Self {
        self.owner = Some(owner.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

impl Default for Identity {
    fn default() -> Self {
        Self::new()
    }
}
