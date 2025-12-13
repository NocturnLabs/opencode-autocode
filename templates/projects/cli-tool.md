<project_specification>
<project_name>{{PROJECT_NAME}}</project_name>

<overview>
A command-line interface tool for {{DESCRIPTION}}.
Built with Rust for performance and reliability, featuring subcommands,
configuration files, and colored output.
</overview>

<technology_stack>
<language>Rust</language>
<cli_framework>clap v4</cli_framework>
<output>console + colored</output>
<serialization>serde + serde_json + toml</serialization>
<error_handling>anyhow + thiserror</error_handling>
</technology_stack>

<prerequisites>
<environment_setup>
- Rust toolchain (rustup)
- Cargo for package management
</environment_setup>
</prerequisites>

<core_features>
<cli_interface>
- Subcommand architecture
- Global and subcommand-specific flags
- Help text and version info
- Bash/zsh completion scripts
</cli_interface>

<configuration>
- TOML configuration file support
- XDG config directory conventions
- Environment variable overrides
- Default values
</configuration>

<output>
- Colored terminal output
- Progress indicators
- Table formatting
- JSON output option
</output>

<error_handling>
- User-friendly error messages
- Exit codes for scripting
- Verbose/debug mode
- Stack traces in debug builds
</error_handling>
</core_features>

<implementation_steps>
<step number="1">
<title>Project Setup</title>
<tasks>
- Initialize Cargo project
- Add dependencies to Cargo.toml
- Set up CLI argument parsing with clap
- Create module structure
</tasks>
</step>

<step number="2">
<title>Core CLI</title>
<tasks>
- Define CLI struct with derive macros
- Implement subcommands
- Add global flags (verbose, quiet, config)
- Create help text
</tasks>
</step>

<step number="3">
<title>Configuration</title>
<tasks>
- Create config struct with serde
- Implement config file loading
- Add environment variable support
- Create default config
</tasks>
</step>

<step number="4">
<title>Core Logic</title>
<tasks>
- Implement main functionality
- Add error handling
- Create output formatting
- Add progress indicators
</tasks>
</step>
</implementation_steps>

<success_criteria>
<functionality>
- All subcommands work correctly
- Configuration file is loaded
- Help text is comprehensive
- Exit codes are correct
</functionality>
<performance>
- Startup time under 50ms
- Handles large inputs efficiently
</performance>
</success_criteria>
</project_specification>
