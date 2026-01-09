//! AI-based spec generation using OpenCode CLI
//!
//! We handle the translation of a user's raw idea into a structured project specification.
//! Refactored into submodules for parser, prompts, and executor logic.

pub mod executor;
pub mod parser;
pub mod prompts;

// Re-export main entry points for backward compatibility (mostly)
pub use executor::{generate_spec_from_idea, refine_spec_from_idea};
