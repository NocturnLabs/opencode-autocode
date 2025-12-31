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
mod docs;
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
    println!("   3. opencode-autocode example   # See agent-centric examples and guides");
    println!("   4. opencode-autocode vibe      # Start autonomous loop");
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

            let summary = regression::run_regression_check(&features, None, false)?;
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
                cli::KnowledgeAction::Set {
                    key,
                    value,
                    category,
                    description,
                } => {
                    let cat = category.as_deref().unwrap_or("general");
                    repo.set(key, value, cat, description.as_deref())?;
                    println!("✅ Fact saved: {} = {}", key, value);
                }
                cli::KnowledgeAction::Get { key } => {
                    if let Some(fact) = repo.get(key)? {
                        println!("{}={}", fact.key, fact.value);
                        if let Some(desc) = fact.description {
                            println!("# {}", desc);
                        }
                    } else {
                        println!("Fact '{}' not found.", key);
                    }
                }
                cli::KnowledgeAction::List { category } => {
                    let facts = repo.list(category.as_deref())?;
                    if facts.is_empty() {
                        println!("No facts found.");
                    } else {
                        for fact in facts {
                            println!("[{}] {} = {}", fact.category, fact.key, fact.value);
                        }
                    }
                }
                cli::KnowledgeAction::Delete { key } => {
                    repo.delete(key)?;
                    println!("🗑️ Fact '{}' deleted.", key);
                }
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
