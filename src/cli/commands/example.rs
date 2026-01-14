use crate::cli::ExampleTopic;
use crate::docs;
use anyhow::Result;

/// Handles the `example` subcommand for displaying documentation topics.
///
/// This function routes to the appropriate documentation topic based on the provided
/// `ExampleTopic`. It either shows a list of available topics or displays the
/// specific documentation content.
///
/// # Arguments
///
/// * `topic` - The example topic to display, wrapped in `ExampleTopic` enum.
///
/// # Returns
///
/// Result indicating success or containing an error from doc retrieval.
pub fn handle_example(topic: &ExampleTopic) -> Result<()> {
    match topic {
        ExampleTopic::Db { insert, query } => {
            if !insert && !query {
                println!("# Database examples (use --insert or --query for specific details)");
                println!("opencode-forger example db --insert");
                println!("opencode-forger example db --query");
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

/// Displays documentation for a given topic name.
///
/// # Arguments
///
/// * `name` - The name/key of the documentation topic to retrieve.
///
/// # Returns
///
/// Result indicating success (always Ok, even if doc not found).
fn show_doc(name: &str) -> Result<()> {
    match docs::get_doc(name) {
        Some(doc) => println!("{}", doc),
        None => println!("Documentation topic '{}' not found.", name),
    }
    Ok(())
}
