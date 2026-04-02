use rusqlite::{Connection, Result};
use std::path::Path;

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trigger_app TEXT NOT NULL,
                trigger_title_contains TEXT NOT NULL DEFAULT '',
                trigger_time_range TEXT,
                trigger_day_of_week TEXT,
                meaning TEXT NOT NULL,
                confidence REAL NOT NULL,
                last_confirmed TEXT NOT NULL,
                deleted_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS episodes (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                context_app TEXT NOT NULL,
                context_title TEXT NOT NULL DEFAULT '',
                context_duration_minutes REAL,
                question TEXT NOT NULL,
                answer TEXT NOT NULL,
                tags TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS speculations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                observed_app TEXT NOT NULL,
                observed_title TEXT NOT NULL DEFAULT '',
                inference TEXT NOT NULL,
                confidence REAL NOT NULL,
                asked_user INTEGER NOT NULL DEFAULT 0,
                matched_pattern_id INTEGER,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS user_profile (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                occupation TEXT,
                skills TEXT NOT NULL DEFAULT '[]',
                interests TEXT NOT NULL DEFAULT '[]',
                notes TEXT NOT NULL DEFAULT '',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS polling_cursors (
                bucket_id TEXT PRIMARY KEY,
                last_event_timestamp TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE TABLE IF NOT EXISTS processed_events (
                event_id TEXT NOT NULL,
                bucket_id TEXT NOT NULL,
                processed_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(event_id, bucket_id)
            );

            CREATE TABLE IF NOT EXISTS user_confirmations (
                key TEXT PRIMARY KEY,
                confirmed_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            ",
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_in_memory_and_migrate() {
        let db = Database::open_in_memory().unwrap();

        // Verify all tables exist
        let tables: Vec<String> = db
            .conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();

        assert!(tables.contains(&"patterns".to_string()));
        assert!(tables.contains(&"episodes".to_string()));
        assert!(tables.contains(&"speculations".to_string()));
        assert!(tables.contains(&"user_profile".to_string()));
        assert!(tables.contains(&"polling_cursors".to_string()));
        assert!(tables.contains(&"processed_events".to_string()));
    }

    #[test]
    fn test_migrate_is_idempotent() {
        let db = Database::open_in_memory().unwrap();
        // Running migrate again should not fail
        db.migrate().unwrap();
    }
}
