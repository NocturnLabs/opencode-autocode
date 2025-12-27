//! Documentation module for opencode-autocode
//! Provides access to embedded documentation and examples.

/// Embedded documentation and examples
// Registration of embedded files

// Existing examples (previously hardcoded in main.rs)
const DB_INSERT_DOC: &str = include_str!("../templates/examples/db_insert.md");
const DB_QUERY_DOC: &str = include_str!("../templates/examples/db_query.md");
const VERIFY_DOC: &str = include_str!("../templates/examples/verify.md");
const CONFIG_DOC: &str = include_str!("../templates/examples/config.md");
const CONDUCTOR_DOC: &str = include_str!("../templates/examples/conductor.md");
const WORKFLOW_DOC: &str = include_str!("../templates/examples/workflow.md");
const SPEC_DOC: &str = include_str!("../templates/examples/spec.md");

// Core documentation files
const IDENTITY_DOC: &str = include_str!("../templates/core/identity.md");
const SECURITY_DOC: &str = include_str!("../templates/core/security.md");
const MCP_GUIDE_DOC: &str = include_str!("../templates/core/mcp_guide.md");
const ARCH_DOC: &str = include_str!("../ARCHITECTURE.md");

// Language/Module guides
const RUST_DOC: &str = include_str!("../templates/modules/rust.md");
const JS_DOC: &str = include_str!("../templates/modules/javascript.md");
const TESTING_DOC: &str = include_str!("../templates/modules/testing.md");
const RECOVERY_DOC: &str = include_str!("../templates/modules/recovery.md");

/// Get a doc by name
pub fn get_doc(name: &str) -> Option<&'static str> {
    match name {
        "db_insert" => Some(DB_INSERT_DOC),
        "db_query" => Some(DB_QUERY_DOC),
        "verify" => Some(VERIFY_DOC),
        "config" => Some(CONFIG_DOC),
        "conductor" => Some(CONDUCTOR_DOC),
        "workflow" => Some(WORKFLOW_DOC),
        "spec" => Some(SPEC_DOC),
        "identity" => Some(IDENTITY_DOC),
        "security" => Some(SECURITY_DOC),
        "mcp" => Some(MCP_GUIDE_DOC),
        "arch" => Some(ARCH_DOC),
        "rust" => Some(RUST_DOC),
        "js" => Some(JS_DOC),
        "testing" => Some(TESTING_DOC),
        "recovery" => Some(RECOVERY_DOC),
        _ => None,
    }
}
