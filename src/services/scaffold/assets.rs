/// Embedded default app spec template
pub const DEFAULT_APP_SPEC: &str = include_str!("../../../docs/examples/default_app_spec.md");

/// Embedded command templates
pub const AUTO_INIT_TEMPLATE: &str = include_str!("../../../templates/commands/auto-init.md");
pub const AUTO_CONTINUE_TEMPLATE: &str =
    include_str!("../../../templates/commands/auto-continue.md");
pub const AUTO_ENHANCE_TEMPLATE: &str = include_str!("../../../templates/commands/auto-enhance.md");

/// Core modules for include directive resolution
pub const CORE_IDENTITY: &str = include_str!("../../../templates/core/identity.md");
pub const CORE_SECURITY: &str = include_str!("../../../templates/core/security.md");
pub const CORE_SIGNALING: &str = include_str!("../../../templates/core/signaling.md");
pub const CORE_DATABASE: &str = include_str!("../../../templates/core/database.md");
pub const CORE_COMMUNICATION: &str = include_str!("../../../templates/core/communication.md");
pub const CORE_MCP_GUIDE: &str = include_str!("../../../templates/core/mcp_guide.md");

/// Embedded security allowlist
pub const SECURITY_ALLOWLIST: &str =
    include_str!("../../../templates/scripts/security-allowlist.json");

/// Embedded user configuration template
pub const USER_CONFIG_TEMPLATE: &str = include_str!("../../../templates/forger-user.toml");

/// Embedded subagent templates for parallel spec generation
pub const SPEC_PRODUCT_AGENT: &str =
    include_str!("../../../templates/scaffold/agents/spec-product.md");
pub const SPEC_ARCHITECTURE_AGENT: &str =
    include_str!("../../../templates/scaffold/agents/spec-architecture.md");
pub const SPEC_QUALITY_AGENT: &str =
    include_str!("../../../templates/scaffold/agents/spec-quality.md");

/// Embedded coder subagent for dual-model architecture
pub const CODER_AGENT: &str = include_str!("../../../templates/scaffold/agents/coder.md");

/// Embedded AGENTS.md template
pub const AGENTS_MD_TEMPLATE: &str = include_str!("../../../templates/AGENTS.md");
