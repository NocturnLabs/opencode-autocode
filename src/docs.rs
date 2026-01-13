//! Documentation module for opencode-forger
//! Provides access to embedded documentation and examples.

// Embedded documentation and examples
// Registration of embedded files

use crate::template_xml;

// Existing examples (previously hardcoded in main.rs)
const DB_INSERT_DOC: &str = include_str!("../templates/examples/db_insert.xml");
const DB_QUERY_DOC: &str = include_str!("../templates/examples/db_query.xml");
const VERIFY_DOC: &str = include_str!("../templates/examples/verify.xml");
const CONFIG_DOC: &str = include_str!("../templates/examples/config.xml");
const CONDUCTOR_DOC: &str = include_str!("../templates/examples/conductor.xml");
const WORKFLOW_DOC: &str = include_str!("../templates/examples/workflow.xml");
const SPEC_DOC: &str = include_str!("../templates/examples/spec.xml");

// Core documentation files
const IDENTITY_DOC: &str = include_str!("../templates/core/identity.xml");
const SECURITY_DOC: &str = include_str!("../templates/core/security.xml");
const MCP_GUIDE_DOC: &str = include_str!("../templates/core/mcp_guide.xml");
const ARCH_DOC: &str = include_str!("../ARCHITECTURE.md");

// Language/Module guides
const RUST_DOC: &str = include_str!("../templates/modules/rust.xml");
const JS_DOC: &str = include_str!("../templates/modules/javascript.xml");
const TESTING_DOC: &str = include_str!("../templates/modules/testing.xml");
const RECOVERY_DOC: &str = include_str!("../templates/modules/recovery.xml");
const VIBE_DOC: &str = include_str!("../templates/examples/vibe.xml");
const TRACKS_DOC: &str = include_str!("../templates/examples/tracks.xml");
const INTERACTIVE_DOC: &str = include_str!("../templates/examples/interactive.xml");
const TEMPLATES_GUIDE_DOC: &str = include_str!("../templates/examples/templates_guide.xml");
const MEMORY_DOC: &str = include_str!("../templates/modules/memory.xml");

/// Get a doc by name
pub fn get_doc(name: &str) -> Option<String> {
    let content = match name {
        "db_insert" => DB_INSERT_DOC,
        "db_query" => DB_QUERY_DOC,
        "verify" => VERIFY_DOC,
        "config" => CONFIG_DOC,
        "conductor" => CONDUCTOR_DOC,
        "workflow" => WORKFLOW_DOC,
        "spec" => SPEC_DOC,
        "identity" => IDENTITY_DOC,
        "security" => SECURITY_DOC,
        "mcp" => MCP_GUIDE_DOC,
        "arch" => ARCH_DOC,
        "rust" => RUST_DOC,
        "js" => JS_DOC,
        "testing" => TESTING_DOC,
        "recovery" => RECOVERY_DOC,
        "vibe" => VIBE_DOC,
        "tracks" => TRACKS_DOC,
        "interactive" => INTERACTIVE_DOC,
        "templates-guide" => TEMPLATES_GUIDE_DOC,
        "memory" => MEMORY_DOC,
        _ => return None,
    };

    template_xml::render_template(content).ok()
}
