//! OpenCode Autocode - Autonomous Coding for OpenCode
//!
//! A CLI tool that scaffolds autonomous coding projects and runs
//! the vibe loop to implement features automatically.

#![deny(warnings)]

mod autonomous;
mod cli;
mod communication;
mod conductor;
mod config;
mod config_tui;
mod db;
mod generator;
mod regression;
mod scaffold;
mod spec;
mod templates;
mod tui;
mod validation;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, DbAction, ExampleTopic, Mode, TemplateAction};
use std::path::PathBuf;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine output directory
    let output_dir = cli.output.clone().unwrap_or_else(|| PathBuf::from("."));

    // Handle subcommands first (vibe is the main one)
    if let Some(command) = &cli.command {
        return match command {
            Commands::Vibe {
                limit,
                config_file,
                developer,
                single_model,
            } => autonomous::run(*limit, config_file.as_deref(), *developer, *single_model),
            Commands::Templates { action } => match action {
                TemplateAction::List => {
                    templates::list_templates();
                    Ok(())
                }
                TemplateAction::Use { name } => templates::use_template(name, &output_dir),
            },
            Commands::Db { action } => handle_db_command(action),
            Commands::Example { topic } => handle_example_command(topic),
        };
    }

    // Handle flag-based modes
    match cli.mode()? {
        Mode::Config => config_tui::run_config_tui(),
        Mode::RegressionCheck => {
            let feature_path = cli
                .feature_list
                .clone()
                .unwrap_or_else(|| PathBuf::from("feature_list.json"));

            if !feature_path.exists() {
                anyhow::bail!("Feature list not found: {}", feature_path.display());
            }

            println!(
                "🔍 Running regression check on {}...",
                feature_path.display()
            );

            let summary = regression::run_regression_check(&feature_path, None, cli.verbose)?;
            regression::report_results(&summary);

            if summary.automated_failed > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        Mode::Default => {
            if cli.dry_run {
                println!("🔍 Dry run mode - no files will be created");
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            println!("🚀 Scaffolding with default app spec...");
            scaffold::scaffold_default(&output_dir)?;
            print_next_steps(&output_dir);
            Ok(())
        }
        Mode::Custom(spec_path) => {
            if cli.dry_run {
                println!("🔍 Dry run mode - no files will be created");
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            println!("📄 Scaffolding with custom spec: {}", spec_path.display());
            scaffold::scaffold_custom(&output_dir, &spec_path)?;
            print_next_steps(&output_dir);
            Ok(())
        }
        Mode::Interactive => {
            if cli.dry_run {
                println!("🔍 Dry run mode - no files will be created");
                scaffold::preview_scaffold(&output_dir);
                return Ok(());
            }
            tui::run_interactive(&output_dir, !cli.no_subagents)?;
            Ok(())
        }
    }
}

fn print_next_steps(output_dir: &std::path::Path) {
    println!("\n✅ Scaffolding complete!");
    println!("   Output directory: {}", output_dir.display());
    println!("\n📋 Next steps:");
    println!("   1. cd {}", output_dir.display());
    println!("   2. opencode-autocode --config  # Configure settings");
    println!("   3. opencode-autocode vibe      # Start autonomous loop");
}

/// Handle database subcommands
fn handle_db_command(action: &DbAction) -> Result<()> {
    match action {
        DbAction::Init { path } => {
            let db_path = path
                .clone()
                .unwrap_or_else(|| PathBuf::from(db::DEFAULT_DB_PATH));

            if db_path.exists() {
                println!("⚠️  Database already exists: {}", db_path.display());
                println!("   Use 'db migrate' to import features from JSON.");
                return Ok(());
            }

            println!("🗃️  Initializing database: {}", db_path.display());
            let _db = db::Database::open(&db_path)?;
            println!("✅ Database initialized successfully!");
            println!("\n📋 Next steps:");
            println!("   1. Run 'opencode-autocode db migrate feature_list.json' to import existing features");
            Ok(())
        }
        DbAction::Migrate { json_path } => {
            let json_path = json_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("feature_list.json"));

            if !json_path.exists() {
                anyhow::bail!("Feature list not found: {}", json_path.display());
            }

            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            println!(
                "📥 Migrating features from {} to {}",
                json_path.display(),
                db_path.display()
            );

            let db = db::Database::open(&db_path)?;
            let count = db.features().import_from_json(&json_path)?;

            println!("✅ Migrated {} features successfully!", count);

            let (passing, remaining) = db.features().count()?;
            println!("   📊 Status: {} passing, {} remaining", passing, remaining);

            Ok(())
        }
        DbAction::Export { output } => {
            let output_path = output
                .clone()
                .unwrap_or_else(|| PathBuf::from("feature_list_export.json"));

            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }

            println!("📤 Exporting features to {}", output_path.display());

            let db = db::Database::open(&db_path)?;
            db.features().export_to_json(&output_path)?;

            let features = db.features().list_all()?;
            println!("✅ Exported {} features successfully!", features.len());

            Ok(())
        }
        DbAction::Stats => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }

            let db = db::Database::open(&db_path)?;

            // Feature stats
            let (passing, remaining) = db.features().count()?;
            let total = passing + remaining;

            // Session stats
            let session_stats = db.sessions().get_stats()?;

            println!("\n📊 Database Statistics");
            println!("═══════════════════════════════════════");
            println!();
            println!("Features:");
            println!("  Total:     {}", total);
            println!(
                "  Passing:   {} ({:.1}%)",
                passing,
                if total > 0 {
                    passing as f64 / total as f64 * 100.0
                } else {
                    0.0
                }
            );
            println!("  Remaining: {}", remaining);
            println!();
            println!("Sessions:");
            println!("  Total:             {}", session_stats.total_sessions);
            println!("  Completed:         {}", session_stats.completed_sessions);
            println!(
                "  Features completed: {}",
                session_stats.total_features_completed
            );
            println!();

            Ok(())
        }
        DbAction::Query { sql } => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }
            let db = db::Database::open(&db_path)?;
            let output = db.read_query(sql)?;
            print!("{}", output);
            Ok(())
        }
        DbAction::Exec { sql } => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }
            let db = db::Database::open(&db_path)?;
            let affected = db.write_query(sql)?;
            println!("{} row(s) affected", affected);
            Ok(())
        }
        DbAction::Tables => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }
            let db = db::Database::open(&db_path)?;
            let tables = db.list_tables()?;
            for table in tables {
                println!("{}", table);
            }
            Ok(())
        }
        DbAction::Schema { table } => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }
            let db = db::Database::open(&db_path)?;
            let schema = db.describe_table(table)?;
            print!("{}", schema);
            Ok(())
        }
        DbAction::NextFeature => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }
            let db = db::Database::open(&db_path)?;
            let next =
                db.read_query("SELECT id, description FROM features WHERE passes = 0 LIMIT 1")?;
            print!("{}", next);
            Ok(())
        }
        DbAction::MarkPass { id } => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }
            let db = db::Database::open(&db_path)?;
            let affected =
                db.write_query(&format!("UPDATE features SET passes = 1 WHERE id = {}", id))?;
            if affected > 0 {
                println!("Feature {} marked as passing", id);
            } else {
                println!("No feature found with id {}", id);
            }
            Ok(())
        }
    }
}

fn handle_example_command(topic: &ExampleTopic) -> Result<()> {
    match topic {
        ExampleTopic::Db { insert, query } => {
            if !insert && !query {
                println!("# Database examples (use --insert or --query for specific details)");
                println!("opencode-autocode example db --insert");
                println!("opencode-autocode example db --query");
            }

            if *insert {
                println!("# Example: Properly granular feature INSERTs\n");
                println!("# DON'T: One vague feature");
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Implement the game', 0, 'cargo build')""#);
                println!("\n# DO: Separate testable features (5-15 minimum)\n");
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Hero entity spawns and renders', 0, 'cargo test test_hero_spawn')""#);
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Hero moves upward automatically', 0, 'cargo test test_hero_movement')""#);
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Weapon fires projectiles', 0, 'cargo test test_weapon_firing')""#);
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Enemies spawn and move', 0, 'cargo test test_enemy_spawn')""#);
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Collision detection works', 0, 'cargo test test_collision')""#);
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('functional', 'Database persists scores', 0, 'cargo test test_persistence')""#);
                println!(r#"opencode-autocode db exec "INSERT INTO features (category, description, passes, verification_command) VALUES ('style', 'UI displays score', 0, 'cargo test test_ui')""#);
                println!("\n# Rules:");
                println!("# - Each feature = ONE testable behavior");
                println!("# - Use real test commands (not just 'cargo build')");
                println!("# - Mix 'functional' and 'style' categories");
            }

            if *query {
                if *insert {
                    println!("\n---\n");
                }
                println!("# Example: SQL queries for feature inspection\n");
                println!("# List all features:");
                println!(r#"opencode-autocode db query "SELECT id, description, passes FROM features""#);
                println!();
                println!("# Count passing/failing:");
                println!(r#"opencode-autocode db query "SELECT passes, COUNT(*) FROM features GROUP BY passes""#);
                println!();
                println!("# Get next feature to work on:");
                println!(r#"opencode-autocode db query "SELECT id, description FROM features WHERE passes = 0 LIMIT 1""#);
                println!();
                println!("# Features with weak verification:");
                println!(r#"opencode-autocode db query "SELECT id, description FROM features WHERE verification_command LIKE '%cargo build%'""#);
            }
            Ok(())
        }
        ExampleTopic::Verify => {
                println!("# Example: Verification commands by project type\n");
                println!("# Rust projects:");
                println!("cargo test test_feature_name");
                println!("cargo test --test integration_tests");
                println!();
                println!("# Web projects (Playwright):");
                println!("npx playwright test --grep \"feature description\"");
                println!("npx playwright test tests/e2e/login.spec.ts");
                println!();
                println!("# Node.js projects:");
                println!("npm test -- --grep \"feature\"");
                println!("npx vitest run --grep \"feature\"");
                println!();
                println!("# Python projects:");
                println!("pytest -k \"test_feature\"");
                println!("python -m pytest tests/test_module.py::test_feature");
                println!();
                println!("# Rules:");
                println!("# - NEVER use 'cargo build' or 'npm run dev' as verification");
                println!("# - Each command should test ONE specific behavior");
                println!("# - Commands must exit 0 on success, non-zero on failure");
                Ok(())
            }
            ExampleTopic::Config => {
                println!("# Example: autocode.toml configuration sections\n");
                println!("[models]");
                println!(r#"default = "anthropic/claude-sonnet-4"      # spec generation"#);
                println!(r#"autonomous = "anthropic/claude-sonnet-4"   # coding (@coder)"#);
                println!(r#"reasoning = "anthropic/claude-3.5-sonnet"  # planning/review"#);
                println!();
                println!("[autonomous]");
                println!("delay_between_sessions = 5");
                println!("max_iterations = 0        # 0 = unlimited");
                println!("session_timeout_minutes = 60");
                println!("auto_commit = true");
                println!();
                println!("[mcp]");
                println!("prefer_osgrep = true");
                println!("use_sequential_thinking = true");
                println!(r#"required_tools = ["chrome-devtools"]"#);
                println!();
                println!("[notifications]");
                println!("webhook_enabled = true");
                println!(r#"webhook_url = "https://discord.com/api/webhooks/...""#);
                Ok(())
            }
            ExampleTopic::Conductor => {
                println!("# Example: Conductor context files\n");
                println!("# .conductor/product.md:");
                println!("```markdown");
                println!("# Product Context");
                println!();
                println!("## What We're Building");
                println!("A fast-paced vertical scrolling game...");
                println!();
                println!("## Core Value Proposition");
                println!("Simple controls, high replayability...");
                println!();
                println!("## Target Users");
                println!("Casual gamers who enjoy arcade-style games...");
                println!("```\n");
                println!("# .conductor/tech_stack.md:");
                println!("```markdown");
                println!("# Technical Stack");
                println!();
                println!("## Languages & Frameworks");
                println!("- Rust 1.70+");
                println!("- Bevy 0.12 (game engine)");
                println!("- SQLite (local persistence)");
                println!();
                println!("## Key Patterns");
                println!("- ECS architecture");
                println!("- Component-based design");
                println!("```");
                Ok(())
            }
            ExampleTopic::Workflow => {
                println!("# Example: Vibe loop workflow phases\n");
                println!("Phase 1: auto-init");
                println!("  └─ Runs if database is empty");
                println!("  └─ Populates features, creates conductor context");
                println!();
                println!("Phase 2: auto-context");
                println!("  └─ Runs if .conductor/ doesn't exist");
                println!("  └─ Creates product.md, tech_stack.md");
                println!();
                println!("Phase 3: auto-continue (active track)");
                println!("  └─ Runs if tracks/<name>/plan.md has incomplete tasks");
                println!("  └─ Works on next task in the plan");
                println!();
                println!("Phase 4: completion check");
                println!("  └─ If all features pass → DONE!");
                println!();
                println!("Phase 5: auto-continue (new feature)");
                println!("  └─ Picks next failing feature from database");
                println!("  └─ Implements using @coder subagent");
                Ok(())
            }
            ExampleTopic::Spec => {
                println!("# Example: App spec structure\n");
                println!("<project_specification>");
                println!("  <project_name>My App</project_name>");
                println!("  <overview>Brief description...</overview>");
                println!("  <technology_stack>");
                println!("    - Frontend: React, TypeScript");
                println!("    - Backend: Node.js, Express");
                println!("    - Database: PostgreSQL");
                println!("  </technology_stack>");
                println!("  <core_features>");
                println!("    - User authentication with JWT");
                println!("    - CRUD operations for resources");
                println!("    - Real-time notifications");
                println!("  </core_features>");
                println!("  <database_schema>");
                println!("    - users: id, email, password_hash");
                println!("    - resources: id, name, user_id");
                println!("  </database_schema>");
                println!("  <success_criteria>");
                println!("    - All E2E tests pass");
                println!("    - <100ms API response time");
                println!("  </success_criteria>");
                println!("</project_specification>");
                Ok(())
            }
        }
    }
