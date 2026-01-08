//! SQLite database module for progress tracking
//!
//! Provides persistent storage for features, sessions, and audit logs.
//! Replaces the previous file-based tracking (feature_list.json).

pub mod connection;
pub mod features;
pub mod knowledge;
pub mod meta;
pub mod query;
mod schema;
pub mod sessions;
#[cfg(test)]
pub mod test_utils;

// Re-export types used by main.rs
pub use connection::{Database, DEFAULT_DB_PATH};
pub use features::FeatureRepository;
pub use knowledge::KnowledgeRepository;
pub use meta::MetaRepository;
pub use sessions::SessionRepository;

pub mod instances;
pub use instances::InstanceRepository;
