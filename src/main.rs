//! OpenCode Forger - Autonomous Coding for OpenCode
//!
//! A CLI tool that scaffolds autonomous coding projects and runs
//! the vibe loop to implement features automatically.
//!
//! For architectural details, see [ARCHITECTURE.md](ARCHITECTURE.md).

#![deny(warnings)]

mod autonomous;
mod common;
mod services;

mod cli;
mod conductor;
mod config;
mod config_tui;
mod db;
mod docs;
// mod generator; // moved to services

mod regression;
// mod scaffold; // moved to services

mod ipc;
mod spec;
mod template_xml;
mod templates;
mod theming;
mod tui;
mod updater;
mod utils;
mod validation;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    cli::handlers::run(cli)
}
