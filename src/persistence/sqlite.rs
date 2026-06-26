use super::{SessionInfo, ThoughtStore};
use crate::types::ThoughtData;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, Utc};

pub struct SqliteThoughtStore {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteThoughtStore {
    pub fn new(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path)
            .map_err(|e| format!("Failed to open SQLite database: {}", e))?;

        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON;", [])
            .map_err(|e| format!("Failed to enable foreign keys: {}", e))?;

        // Initialize schema
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                total_thoughts INTEGER DEFAULT 0
            );",
            [],
        )
        .map_err(|e| format!("Failed to create sessions table: {}", e))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS thoughts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                thought_number INTEGER NOT NULL,
                thought_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
            [],
        )
        .map_err(|e| format!("Failed to create thoughts table: {}", e))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_thoughts_session ON thoughts(session_id);",
            [],
        )
        .map_err(|e| format!("Failed to create index on thoughts: {}", e))?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

impl ThoughtStore for SqliteThoughtStore {
    fn save_thought(&mut self, session_id: &str, thought: &ThoughtData) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();

        let now = Utc::now().to_rfc3339();

        // Insert or update session
        conn.execute(
            "INSERT INTO sessions (id, created_at, updated_at, total_thoughts)
             VALUES (?1, ?2, ?3, 1)
             ON CONFLICT(id) DO UPDATE SET
                updated_at = excluded.updated_at,
                total_thoughts = total_thoughts + 1;",
            params![session_id, now, now],
        )
        .map_err(|e| format!("Failed to save session record: {}", e))?;

        // Serialize thought to JSON
        let thought_json = serde_json::to_string(thought)
            .map_err(|e| format!("Failed to serialize thought to JSON: {}", e))?;

        conn.execute(
            "INSERT INTO thoughts (session_id, thought_number, thought_json, created_at)
             VALUES (?1, ?2, ?3, ?4);",
            params![session_id, thought.thought_number, thought_json, now],
        )
        .map_err(|e| format!("Failed to insert thought: {}", e))?;

        Ok(())
    }

    fn load_session(&self, session_id: &str) -> Result<Vec<ThoughtData>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT thought_json FROM thoughts 
                 WHERE session_id = ?1 
                 ORDER BY thought_number ASC, id ASC;",
            )
            .map_err(|e| format!("Failed to prepare load statement: {}", e))?;

        let rows = stmt
            .query_map(params![session_id], |row| {
                let json_str: String = row.get(0)?;
                Ok(json_str)
            })
            .map_err(|e| format!("Failed to execute load query: {}", e))?;

        let mut thoughts = Vec::new();
        for r in rows {
            let json_str = r.map_err(|e| format!("Database read error: {}", e))?;
            let thought: ThoughtData = serde_json::from_str(&json_str)
                .map_err(|e| format!("Failed to deserialize thought: {}", e))?;
            thoughts.push(thought);
        }

        Ok(thoughts)
    }

    fn list_sessions(&self) -> Result<Vec<SessionInfo>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, created_at, updated_at, total_thoughts 
                 FROM sessions 
                 ORDER BY updated_at DESC;",
            )
            .map_err(|e| format!("Failed to prepare list statement: {}", e))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let created_str: String = row.get(1)?;
                let updated_str: String = row.get(2)?;
                let total_thoughts: i64 = row.get(3)?;
                Ok((id, created_str, updated_str, total_thoughts))
            })
            .map_err(|e| format!("Failed to execute list query: {}", e))?;

        let mut sessions = Vec::new();
        for r in rows {
            let (id, created_str, updated_str, total_thoughts) =
                r.map_err(|e| format!("Database read error: {}", e))?;

            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            sessions.push(SessionInfo {
                id,
                created_at,
                updated_at,
                total_thoughts: total_thoughts as usize,
            });
        }

        Ok(sessions)
    }

    fn delete_session(&mut self, session_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE id = ?1;", params![session_id])
            .map_err(|e| format!("Failed to delete session: {}", e))?;
        Ok(())
    }
}
