use anyhow::Result;

use super::connection::Database;

// ============================================================
// MCP-equivalent operations (replaces SQLite MCP server)
// ============================================================

impl Database {
    /// Execute a read-only SELECT query, returns formatted output.
    /// This function enforces read-only mode by rejecting any non-SELECT statements.
    ///
    /// PRAGMA statements are restricted to a safe subset of read-only introspection
    /// commands, since some PRAGMAs can mutate database state.
    pub fn read_query(&self, sql: &str) -> Result<String> {
        let sql_upper = sql.trim().to_uppercase();

        // Allowlist: SELECT queries and safe read-only PRAGMAs for introspection
        let is_safe = sql_upper.starts_with("SELECT")
            || sql_upper.starts_with("PRAGMA TABLE_INFO")
            || sql_upper.starts_with("PRAGMA DATABASE_LIST")
            || sql_upper.starts_with("PRAGMA INDEX_LIST")
            || sql_upper.starts_with("PRAGMA INDEX_INFO")
            || sql_upper.starts_with("PRAGMA FOREIGN_KEY_LIST")
            || sql_upper.starts_with("PRAGMA TABLE_LIST");

        if !is_safe {
            anyhow::bail!(
                "read_query only allows SELECT and safe PRAGMA statements. Use 'db exec' for modifications."
            );
        }

        let conn = self.connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(sql)?;
        let column_count = stmt.column_count();
        let column_names: Vec<String> = stmt.column_names().iter().map(|s| s.to_string()).collect();

        let rows: Vec<Vec<String>> = stmt
            .query_map([], |row| {
                let mut values = Vec::new();
                for i in 0..column_count {
                    let value: String = row
                        .get::<_, rusqlite::types::Value>(i)
                        .map(|v| format_value(&v))
                        .unwrap_or_else(|_| "NULL".to_string());
                    values.push(value);
                }
                Ok(values)
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Format as table
        Ok(format_table(&column_names, &rows))
    }

    /// Execute a write query (INSERT, UPDATE, DELETE, CREATE), returns rows affected
    /// Uses execute_batch to support multi-value INSERTs
    pub fn write_query(&self, sql: &str) -> Result<usize> {
        let conn = self.connection();
        let conn = conn.lock().unwrap();
        // Use execute_batch for multi-statement/multi-value support
        conn.execute_batch(sql)?;
        // Return changes from last statement
        Ok(conn.changes() as usize)
    }

    /// List all table names in the database
    pub fn list_tables(&self) -> Result<Vec<String>> {
        let conn = self.connection();
        let conn = conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let tables: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(tables)
    }

    /// Get the schema (CREATE statement) for a table
    pub fn describe_table(&self, table_name: &str) -> Result<String> {
        let conn = self.connection();
        let conn = conn.lock().unwrap();

        // Get column info
        let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table_name))?;
        let columns: Vec<(String, String, bool, bool)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(1)?,   // name
                    row.get::<_, String>(2)?,   // type
                    row.get::<_, i32>(3)? != 0, // notnull
                    row.get::<_, i32>(5)? != 0, // pk
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        if columns.is_empty() {
            anyhow::bail!("Table '{}' not found", table_name);
        }

        let mut output = format!("Table: {}\n", table_name);
        output.push_str("Columns:\n");
        for (name, col_type, notnull, pk) in columns {
            let mut flags = Vec::new();
            if pk {
                flags.push("PRIMARY KEY");
            }
            if notnull {
                flags.push("NOT NULL");
            }
            let flags_str = if flags.is_empty() {
                String::new()
            } else {
                format!(" [{}]", flags.join(", "))
            };
            output.push_str(&format!("  - {}: {}{}\n", name, col_type, flags_str));
        }

        Ok(output)
    }
}

/// Format a SQLite value as a string (DRY helper)
fn format_value(value: &rusqlite::types::Value) -> String {
    match value {
        rusqlite::types::Value::Null => "NULL".to_string(),
        rusqlite::types::Value::Integer(i) => i.to_string(),
        rusqlite::types::Value::Real(f) => f.to_string(),
        rusqlite::types::Value::Text(s) => s.clone(),
        rusqlite::types::Value::Blob(_) => "[BLOB]".to_string(),
    }
}

/// Format rows as a simple table (DRY helper)
fn format_table(headers: &[String], rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return "(no rows)\n".to_string();
    }

    // Calculate column widths
    let col_widths: Vec<usize> = (0..headers.len())
        .map(|i| {
            let header_len = headers[i].len();
            let max_row_len = rows
                .iter()
                .map(|r| r.get(i).map_or(0, |s| s.len()))
                .max()
                .unwrap_or(0);
            header_len.max(max_row_len)
        })
        .collect();

    let mut output = String::new();

    // Header
    for (i, h) in headers.iter().enumerate() {
        output.push_str(&format!("{:width$}", h, width = col_widths[i] + 2));
    }
    output.push('\n');

    // Separator
    for w in &col_widths {
        output.push_str(&format!("{:-<width$}", "", width = w + 2));
    }
    output.push('\n');

    // Rows
    for row in rows {
        for (i, v) in row.iter().enumerate() {
            output.push_str(&format!("{:width$}", v, width = col_widths[i] + 2));
        }
        output.push('\n');
    }

    output
}
