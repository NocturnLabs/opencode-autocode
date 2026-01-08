use anyhow::Result;
use iocraft::prelude::*;
use std::path::PathBuf;

use crate::cli::DbAction;
use crate::config::Config;
use crate::db;
use crate::regression;

/// Handle database subcommands
pub fn handle_db(action: &DbAction) -> Result<()> {
    // Load config to get database_file path (from .forger/config.toml)
    let config = Config::load(None).unwrap_or_default();
    let default_db_path = PathBuf::from(&config.paths.database_file);

    match action {
        DbAction::Init { path } => {
            let db_path = path.clone().unwrap_or(default_db_path);

            if db_path.exists() {
                println!("⚠️  Database already exists: {}", db_path.display());
                println!("   Use 'db migrate' to import features from JSON.");
                return Ok(());
            }

            println!("🗃️  Initializing database: {}", db_path.display());
            let _db = db::Database::open(&db_path)?;
            println!("✅ Database initialized successfully!");
            println!("\n📋 Next steps:");
            println!("   1. Run 'opencode-forger db migrate feature_list.json' to import existing features");
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
            if !default_db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    default_db_path.display()
                );
            }
            let db = db::Database::open(&default_db_path)?;
            let output = db.read_query(sql)?;
            print!("{}", output);
            Ok(())
        }
        DbAction::Exec { sql } => {
            if !default_db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    default_db_path.display()
                );
            }
            let db = db::Database::open(&default_db_path)?;

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
            if !default_db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    default_db_path.display()
                );
            }
            let db = db::Database::open(&default_db_path)?;
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
            if !default_db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    default_db_path.display()
                );
            }
            let db = db::Database::open(&default_db_path)?;
            let tables = db.list_tables()?;
            for table in tables {
                println!("{}", table);
            }
            Ok(())
        }
        DbAction::Schema { table } => {
            if !default_db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    default_db_path.display()
                );
            }
            let db = db::Database::open(&default_db_path)?;
            let schema = db.describe_table(table)?;
            print!("{}", schema);
            Ok(())
        }
        DbAction::NextFeature => {
            if !default_db_path.exists() {
                anyhow::bail!(
                    "Database not found: {}. Run 'db init' first.",
                    default_db_path.display()
                );
            }
            let db = db::Database::open(&default_db_path)?;
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
                crate::cli::KnowledgeAction::Set {
                    key,
                    value,
                    category,
                    description,
                } => {
                    let cat = category.as_deref().unwrap_or("general");
                    repo.set(key, value, cat, description.as_deref())?;
                    println!("✅ Fact saved: {} = {}", key, value);
                }
                crate::cli::KnowledgeAction::Get { key } => {
                    if let Some(fact) = repo.get(key)? {
                        println!("{}={}", fact.key, fact.value);
                        if let Some(desc) = fact.description {
                            println!("# {}", desc);
                        }
                    } else {
                        println!("Fact '{}' not found.", key);
                    }
                }
                crate::cli::KnowledgeAction::List { category } => {
                    let facts = repo.list(category.as_deref())?;
                    if facts.is_empty() {
                        println!("No facts found.");
                    } else {
                        for fact in facts {
                            println!("[{}] {} = {}", fact.category, fact.key, fact.value);
                        }
                    }
                }
                crate::cli::KnowledgeAction::Delete { key } => {
                    repo.delete(key)?;
                    println!("🗑️ Fact '{}' deleted.", key);
                }
                crate::cli::KnowledgeAction::TrackServer { port, pid } => {
                    repo.track_server(*port, *pid)?;
                    println!("✅ Tracking server on port {} (PID: {})", port, pid);
                }
                crate::cli::KnowledgeAction::GetServer { port } => {
                    if let Some(pid) = repo.get_tracked_server(*port)? {
                        println!("port={}  pid={}", port, pid);
                    } else {
                        println!("No server tracked on port {}", port);
                    }
                }
                crate::cli::KnowledgeAction::UntrackServer { port } => {
                    repo.untrack_server(*port)?;
                    println!("🗑️ Untracked server on port {}", port);
                }
            }
            Ok(())
        }
    }
}
