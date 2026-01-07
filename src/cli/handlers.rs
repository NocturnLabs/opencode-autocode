//! CLI Command Handlers
//!
//! This module contains the implementation logic for CLI subcommands,
//! extracted from main.rs to improve modularity and testability.

use anyhow::Result;
use std::path::PathBuf;

use crate::config_tui;
use crate::db;
use crate::docs;
use crate::regression;
use crate::scaffold;
use crate::templates;
use crate::tui;
use crate::updater;
use iocraft::prelude::*;

use super::{Cli, Commands, DbAction, ExampleTopic, Mode, TemplateAction};

/// Main entry point for handling CLI commands.
///
/// Parses the CLI arguments and dispatches to the appropriate handler.
pub fn run(cli: Cli) -> Result<()> {
    let output_dir = cli.output.clone().unwrap_or_else(|| PathBuf::from("."));

    // Handle subcommands first
    if let Some(command) = &cli.command {
        return match command {
            Commands::Vibe {
                limit,
                config_file,
                developer,
                single_model,
                parallel,
                feature_id,
            } => handle_vibe(
                *limit,
                config_file.as_deref(),
                *developer,
                *single_model,
                *parallel,
                *feature_id,
            ),
            Commands::Enhance {
                limit,
                config_file,
                developer,
                single_model,
            } => crate::autonomous::run(
                *limit,
                config_file.as_deref(),
                *developer,
                *single_model,
                true,
                None,
            ),
            Commands::Init {
                default,
                spec,
                no_subagents,
            } => handle_init(&output_dir, *default, spec.as_deref(), *no_subagents),
            Commands::Templates { action } => handle_templates(action, &output_dir),
            Commands::Db { action } => handle_db(action),
            Commands::Example { topic } => handle_example(topic),
            Commands::Update => match updater::update() {
                Ok(_) => Ok(()),
                Err(e) => {
                    eprintln!("❌ Failed to update: {}", e);
                    std::process::exit(1);
                }
            },
        };
    }

    // Handle flag-based modes
    match cli.mode()? {
        Mode::Config => config_tui::run_config_tui(None).map(|_| ()),
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
        Mode::Reset => handle_reset(&output_dir),
        Mode::Interactive => {
            // Check for updates
            if let Ok(Some(new_version)) = updater::check_for_update() {
                println!(
                    "\n🚀 A new version is available: {} (Run 'opencode-autocode update' to upgrade)\n",
                    new_version
                );
            }

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

fn handle_vibe(
    limit: Option<usize>,
    config_file: Option<&std::path::Path>,
    developer: bool,
    single_model: bool,
    parallel: Option<usize>,
    feature_id: Option<i64>,
) -> Result<()> {
    if let Some(worker_count) = parallel {
        // Parallel mode using worktrees
        let count = if worker_count == 0 {
            num_cpus::get() / 2 // Auto-detect: half of CPU cores
        } else {
            worker_count
        };
        println!("🔀 Starting parallel mode with {} workers", count);
        crate::autonomous::run_parallel(count, limit, config_file, developer)
    } else {
        // Standard sequential mode
        crate::autonomous::run(
            limit,
            config_file,
            developer,
            single_model,
            false,
            feature_id,
        )
    }
}

fn handle_init(
    output_dir: &std::path::Path,
    default: bool,
    spec: Option<&std::path::Path>,
    no_subagents: bool,
) -> Result<()> {
    if default {
        println!("🚀 Scaffolding with default app spec...");
        scaffold::scaffold_default(output_dir)?;
        print_next_steps(output_dir);
        Ok(())
    } else if let Some(spec_path) = spec {
        if !spec_path.exists() {
            anyhow::bail!("Spec file not found: {}", spec_path.display());
        }
        println!("📄 Scaffolding with custom spec: {}", spec_path.display());
        scaffold::scaffold_custom(output_dir, spec_path)?;
        print_next_steps(output_dir);
        Ok(())
    } else {
        tui::run_interactive(output_dir, !no_subagents)?;
        Ok(())
    }
}

fn handle_templates(action: &TemplateAction, output_dir: &std::path::Path) -> Result<()> {
    match action {
        TemplateAction::List => {
            templates::list_templates();
            Ok(())
        }
        TemplateAction::Use { name } => templates::use_template(name, output_dir),
    }
}

fn handle_reset(output_dir: &std::path::Path) -> Result<()> {
    let spec_path = output_dir.join(".autocode/app_spec.md");
    if !spec_path.exists() {
        anyhow::bail!(
            "Cannot reset: .autocode/app_spec.md not found in {}",
            output_dir.display()
        );
    }

    println!("🔄 Resetting project (preserving database)...");

    // Clean up ONLY temporary/signal files - PRESERVE the .db!
    let files_to_remove = [
        output_dir.join(".opencode-signal"),
        output_dir.join(".opencode-stop"),
    ];
    for path in &files_to_remove {
        if path.exists() {
            std::fs::remove_file(path)?;
            println!("   🗑️  Removed {}", path.display());
        }
    }

    // Remove .opencode/command directory to get fresh templates
    let command_dir = output_dir.join(".opencode/command");
    if command_dir.exists() {
        std::fs::remove_dir_all(&command_dir)?;
        println!("   🗑️  Removed {}", command_dir.display());
    }

    // Re-scaffold with existing spec
    println!("   📄 Re-scaffolding with existing spec...");
    scaffold::scaffold_custom(output_dir, &spec_path)?;

    println!("\n✅ Reset complete! Project is ready for a fresh run.");
    println!("   Run 'opencode-autocode vibe' to start the autonomous loop.");
    Ok(())
}

/// Handle database subcommands
pub fn handle_db(action: &DbAction) -> Result<()> {
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
                .unwrap_or_else(|| PathBuf::from("feature_list.json"));

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

            // Render the component to stdout
            element!(crate::tui::stats::DbStatsView(
                total: total,
                passing: passing,
                remaining: remaining,
                session_stats: session_stats
            ))
            .print();

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

            // Auto-detect SELECT and redirect to read_query
            let trimmed = sql.trim().to_uppercase();
            if trimmed.starts_with("SELECT") || trimmed.starts_with("PRAGMA") {
                let output = db.read_query(sql)?;
                print!("{}", output);
            } else {
                let affected = db.write_query(sql)?;
                println!("{} row(s) affected", affected);
            }
            Ok(())
        }
        DbAction::Check { path: _ } => {
            let db_path = PathBuf::from(db::DEFAULT_DB_PATH);
            if !db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    db_path.display()
                );
            }
            let db = db::Database::open(&db_path)?;
            let features = db.features().list_all()?;

            println!(
                "🔍 Running regression check on {} feature(s)...",
                features.len()
            );

            let summary = regression::run_regression_check(&features, None, false, None)?;
            regression::report_results(&summary);

            if summary.automated_failed > 0 {
                std::process::exit(1);
            }
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
        DbAction::Knowledge { action } => {
            let db = db::Database::open_default()?;
            let repo = db.knowledge();

            match action {
                super::KnowledgeAction::Set {
                    key,
                    value,
                    category,
                    description,
                } => {
                    let cat = category.as_deref().unwrap_or("general");
                    repo.set(key, value, cat, description.as_deref())?;
                    println!("✅ Fact saved: {} = {}", key, value);
                }
                super::KnowledgeAction::Get { key } => {
                    if let Some(fact) = repo.get(key)? {
                        println!("{}={}", fact.key, fact.value);
                        if let Some(desc) = fact.description {
                            println!("# {}", desc);
                        }
                    } else {
                        println!("Fact '{}' not found.", key);
                    }
                }
                super::KnowledgeAction::List { category } => {
                    let facts = repo.list(category.as_deref())?;
                    if facts.is_empty() {
                        println!("No facts found.");
                    } else {
                        for fact in facts {
                            println!("[{}] {} = {}", fact.category, fact.key, fact.value);
                        }
                    }
                }
                super::KnowledgeAction::Delete { key } => {
                    repo.delete(key)?;
                    println!("🗑️ Fact '{}' deleted.", key);
                }
                super::KnowledgeAction::TrackServer { port, pid } => {
                    repo.track_server(*port, *pid)?;
                    println!("✅ Tracking server on port {} (PID: {})", port, pid);
                }
                super::KnowledgeAction::GetServer { port } => {
                    if let Some(pid) = repo.get_tracked_server(*port)? {
                        println!("port={}  pid={}", port, pid);
                    } else {
                        println!("No server tracked on port {}", port);
                    }
                }
                super::KnowledgeAction::UntrackServer { port } => {
                    repo.untrack_server(*port)?;
                    println!("🗑️ Untracked server on port {}", port);
                }
            }
            Ok(())
        }
    }
}

fn handle_example(topic: &ExampleTopic) -> Result<()> {
    match topic {
        ExampleTopic::Db { insert, query } => {
            if !insert && !query {
                println!("# Database examples (use --insert or --query for specific details)");
                println!("opencode-autocode example db --insert");
                println!("opencode-autocode example db --query");
                return Ok(());
            }

            if *insert {
                if let Some(doc) = docs::get_doc("db_insert") {
                    println!("{}", doc);
                }
            }

            if *query {
                if *insert {
                    println!("\n---\n");
                }
                if let Some(doc) = docs::get_doc("db_query") {
                    println!("{}", doc);
                }
            }
            Ok(())
        }
        ExampleTopic::Verify => show_doc("verify"),
        ExampleTopic::Config => show_doc("config"),
        ExampleTopic::Conductor => show_doc("conductor"),
        ExampleTopic::Workflow => show_doc("workflow"),
        ExampleTopic::Spec => show_doc("spec"),
        ExampleTopic::Identity => show_doc("identity"),
        ExampleTopic::Security => show_doc("security"),
        ExampleTopic::Mcp => show_doc("mcp"),
        ExampleTopic::Arch => show_doc("arch"),
        ExampleTopic::Rust => show_doc("rust"),
        ExampleTopic::Js => show_doc("js"),
        ExampleTopic::Testing => show_doc("testing"),
        ExampleTopic::Recovery => show_doc("recovery"),
        ExampleTopic::Vibe => show_doc("vibe"),
        ExampleTopic::Tracks => show_doc("tracks"),
        ExampleTopic::Interactive => show_doc("interactive"),
        ExampleTopic::TemplatesGuide => show_doc("templates-guide"),
    }
}

fn show_doc(name: &str) -> Result<()> {
    match docs::get_doc(name) {
        Some(doc) => println!("{}", doc),
        None => println!("Documentation topic '{}' not found.", name),
    }
    Ok(())
}

fn print_next_steps(output_dir: &std::path::Path) {
    println!("\n✅ Scaffolding complete!");
    println!("   Output directory: {}", output_dir.display());
    println!("\n📋 Next steps:");
    println!("   1. cd {}", output_dir.display());
    println!("   2. opencode-autocode --config  # Configure settings");
    println!("   3. opencode-autocode example   # See agent-centric examples and guides");
    println!("   4. opencode-autocode vibe      # Start autonomous loop");
}
