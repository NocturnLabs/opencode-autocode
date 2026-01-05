//! Shared utilities for file I/O operations
//!
//! Centralizes file read/write with consistent error context to eliminate
//! duplicate `fs::write(...).with_context(...)` patterns across the codebase.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Write content to a file, creating parent directories if needed.
///
/// This is the standard way to write files throughout the codebase,
/// providing consistent error messages that include the file path.
pub fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
    }
    fs::write(path, content).with_context(|| format!("Failed to write file: {}", path.display()))
}

/// Read a file's contents as a string with consistent error context.
pub fn read_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_file_creates_parent_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir.path().join("a/b/c/file.txt");

        write_file(&nested_path, "hello").unwrap();

        assert!(nested_path.exists());
        assert_eq!(std::fs::read_to_string(&nested_path).unwrap(), "hello");
    }

    #[test]
    fn test_read_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "world").unwrap();

        let content = read_file(&file_path).unwrap();
        assert_eq!(content, "world");
    }

    #[test]
    fn test_read_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let missing = temp_dir.path().join("missing.txt");

        let result = read_file(&missing);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Failed to read file"));
    }
}
