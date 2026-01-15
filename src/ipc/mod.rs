//! IPC protocol types and communication for the Go TUI client.
//!
//! This module defines the JSON-RPC-style protocol used to communicate
//! between the Rust engine and the Go Bubble Tea frontend.

mod protocol;
mod server;

pub use protocol::*;
pub use server::*;
