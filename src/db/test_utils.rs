//! Shared test utilities for database modules
//!
//! Centralizes database setup and configuration for tests to prevent
//! duplicate implementation of boilerplate like `setup_test_db`.

#[cfg(test)]
pub mod tests {
    use crate::db::Database;
    use tempfile::TempDir;

    /// Create a temporary database for testing, returning the temp directory
    /// (which cleans up on drop) and the Database handle.
    pub fn setup_test_db() -> (TempDir, Database) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir for test database");
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path).expect("Failed to open test database");
        (temp_dir, db)
    }
}
