/// Embedded default app spec template
use anyhow::Result;

use crate::template_xml;

pub const DEFAULT_APP_SPEC: &str = include_str!("../../../docs/examples/default_app_spec.md");

/// Embedded command templates
pub const AUTO_INIT_TEMPLATE: &str = include_str!("../../../templates/commands/auto-init.xml");
pub const AUTO_CONTINUE_TEMPLATE: &str =
    include_str!("../../../templates/commands/auto-continue.xml");
pub const AUTO_ENHANCE_TEMPLATE: &str =
    include_str!("../../../templates/commands/auto-enhance.xml");

/// Core modules for include directive resolution
pub const CORE_IDENTITY: &str = include_str!("../../../templates/core/identity.xml");
pub const CORE_SECURITY: &str = include_str!("../../../templates/core/security.xml");
pub const CORE_SIGNALING: &str = include_str!("../../../templates/core/signaling.xml");
pub const CORE_DATABASE: &str = include_str!("../../../templates/core/database.xml");
pub const CORE_MCP_GUIDE: &str = include_str!("../../../templates/core/mcp_guide.xml");

/// Embedded security allowlist
pub const SECURITY_ALLOWLIST: &str =
    include_str!("../../../templates/scripts/security-allowlist.json");

/// Embedded user configuration template
pub const USER_CONFIG_TEMPLATE: &str = include_str!("../../../templates/forger-user.toml");

/// Embedded subagent templates for parallel spec generation
pub const SPEC_PRODUCT_AGENT: &str =
    include_str!("../../../templates/scaffold/agents/spec-product.xml");
pub const SPEC_ARCHITECTURE_AGENT: &str =
    include_str!("../../../templates/scaffold/agents/spec-architecture.xml");
pub const SPEC_QUALITY_AGENT: &str =
    include_str!("../../../templates/scaffold/agents/spec-quality.xml");

/// Embedded coder subagent for dual-model architecture
pub const CODER_AGENT: &str = include_str!("../../../templates/scaffold/agents/coder.xml");

/// Embedded AGENTS.md template
pub const AGENTS_MD_TEMPLATE: &str = include_str!("../../../templates/AGENTS.md");

pub fn auto_init_template() -> Result<String> {
    template_xml::render_template(AUTO_INIT_TEMPLATE)
}

pub fn auto_continue_template() -> Result<String> {
    template_xml::render_template(AUTO_CONTINUE_TEMPLATE)
}

pub fn auto_enhance_template() -> Result<String> {
    template_xml::render_template(AUTO_ENHANCE_TEMPLATE)
}

pub fn core_identity() -> Result<String> {
    template_xml::render_template(CORE_IDENTITY)
}

pub fn core_security() -> Result<String> {
    template_xml::render_template(CORE_SECURITY)
}

pub fn core_signaling() -> Result<String> {
    template_xml::render_template(CORE_SIGNALING)
}

pub fn core_database() -> Result<String> {
    template_xml::render_template(CORE_DATABASE)
}

pub fn core_mcp_guide() -> Result<String> {
    template_xml::render_template(CORE_MCP_GUIDE)
}

pub fn spec_product_agent() -> Result<String> {
    template_xml::render_template(SPEC_PRODUCT_AGENT)
}

pub fn spec_architecture_agent() -> Result<String> {
    template_xml::render_template(SPEC_ARCHITECTURE_AGENT)
}

pub fn spec_quality_agent() -> Result<String> {
    template_xml::render_template(SPEC_QUALITY_AGENT)
}

pub fn coder_agent() -> Result<String> {
    template_xml::render_template(CODER_AGENT)
}
