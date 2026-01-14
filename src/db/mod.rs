//! SQLite database module for progress tracking
//!
//! Provides persistent storage for features, sessions, and audit logs.
//! Replaces the previous file-based tracking (feature_list.json).
//!
//! # Modules
//!
//! - `connection`: Database connection management
//! - `features`: Feature repository and models
//! - `knowledge`: Knowledge base storage
//! - `meta`: Metadata storage
//! - `query`: Database query utilities
//! - `sessions`: Session tracking and management
//! - `instances`: Instance management

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
pub use connection::Database;
pub use features::FeatureRepository;
pub use knowledge::KnowledgeRepository;
pub use meta::MetaRepository;
pub use sessions::SessionRepository;

pub mod instances;
pub use instances::InstanceRepository;
