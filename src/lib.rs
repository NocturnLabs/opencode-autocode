//! OpenCode Forger - Library exports for testing
//!
//! This library module exposes internal functions for integration tests.

pub mod autonomous;
pub mod common;
pub mod services;
pub use services::generator;
pub use services::scaffold;

pub mod cli;
pub mod conductor;
pub mod config;
pub mod config_tui;
pub mod db;
pub mod docs;
// pub mod generator; // moved to services

pub mod regression;
// pub mod scaffold; // moved to services

pub mod spec;
pub mod templates;
pub mod theming;
pub mod tui;
pub mod updater;
pub mod utils;
pub mod validation;
pub mod web;
