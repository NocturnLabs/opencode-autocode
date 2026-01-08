use std::fs;
use std::path::Path;

fn main() {
    // Ensure the dist directory exists for RustEmbed so compilation doesn't fail
    // if the directory is missing (it's gitignored).
    let dist_path = Path::new("src/web/frontend/dist");
    if !dist_path.exists() {
        // Create directory
        if let Err(e) = fs::create_dir_all(dist_path) {
            println!("cargo:warning=Failed to create dist directory: {}", e);
        }
    }

    // Ensure index.html exists as a placeholder
    let index_path = dist_path.join("index.html");
    if !index_path.exists() {
        if let Err(e) = fs::write(&index_path, "<!-- Placeholder for compilation -->") {
            println!(
                "cargo:warning=Failed to create placeholder index.html: {}",
                e
            );
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}
