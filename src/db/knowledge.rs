//! Knowledge repository for persistent agent facts
//!
//! Allows agents to store and retrieve discovered information (e.g. ports, paths).

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// A single unit of knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Knowledge {
    pub key: String,
    pub value: String,
    pub category: String,
    pub description: Option<String>,
}

/// Repository for knowledge operations
pub struct KnowledgeRepository {
    conn: Arc<Mutex<Connection>>,
}

impl KnowledgeRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Set a fact (insert or replace)
    pub fn set(&self, key: &str, value: &str, category: &str, description: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO knowledge (key, value, category, description, updated_at) 
             VALUES (?1, ?2, ?3, ?4, datetime('now'))",
            params![key, value, category, description],
        ).context("Failed to set knowledge")?;
        Ok(())
    }

    /// Get a fact by key
    pub fn get(&self, key: &str) -> Result<Option<Knowledge>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value, category, description FROM knowledge WHERE key = ?1")?;
        
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Knowledge {
                key: row.get(0)?,
                value: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all knowledge, optionally filtered by category
    pub fn list(&self, category_filter: Option<&str>) -> Result<Vec<Knowledge>> {
        let conn = self.conn.lock().unwrap();
        let mut sql = "SELECT key, value, category, description FROM knowledge".to_string();
        
        if category_filter.is_some() {
            sql.push_str(" WHERE category = ?1");
        }
        sql.push_str(" ORDER BY category, key");

        let mut stmt = conn.prepare(&sql)?;
        
        // Use a helper closure to map rows
        let mapper = |row: &rusqlite::Row| -> rusqlite::Result<Knowledge> {
            Ok(Knowledge {
                key: row.get(0)?,
                value: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
            })
        };

        // Execute query
        let iter = if let Some(cat) = category_filter {
            stmt.query_map(params![cat], mapper)?
        } else {
            stmt.query_map([], mapper)?
        };

        let mut result = Vec::new();
        for item in iter {
            result.push(item?);
        }
        Ok(result)
    }

    /// Delete a fact
    pub fn delete(&self, key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM knowledge WHERE key = ?1", params![key])?;
        Ok(())
    }
}
