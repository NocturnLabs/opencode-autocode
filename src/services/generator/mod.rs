//! AI-based spec generation using OpenCode CLI
//!
//! We handle the translation of a user's raw idea into a structured project specification.
//! Refactored into submodules for parser, prompts, and executor logic.
//!
//! # Modules
//!
//! - `executor`: Core execution logic for spec generation and refinement
//! - `parser`: XML parsing and extraction utilities
//! - `prompts`: Prompt generation for different scenarios
//! - `sanitize`: XML sanitization for AI-generated specifications

pub mod executor;
pub mod parser;
pub mod prompts;
pub mod sanitize;

// Re-export main entry points for backward compatibility (mostly)
pub use executor::{generate_spec_from_idea, refine_spec_from_idea};
