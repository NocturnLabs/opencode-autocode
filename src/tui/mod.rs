//! Interactive TUI for building app specs
//!
//! Provides multiple modes for creating project specifications:
//! - Generated: AI creates spec from idea
//! - Manual: Step-by-step form
//! - From file: Use existing spec
//! - Default: Built-in template

mod actions;
pub mod components;
mod fullscreen;
mod generated;
mod manual;
pub mod prompts;
pub mod stats;
mod validation;

// Re-export the main entry points
pub use fullscreen::run_interactive;

// Re-export the config editor
pub use fullscreen::run_fullscreen_config_editor;
