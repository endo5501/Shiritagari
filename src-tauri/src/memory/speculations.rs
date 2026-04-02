use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use super::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Speculation {
    pub id: i64,
    pub timestamp: String,
    pub observed_app: String,
    pub observed_title: String,
    pub inference: String,
    pub confidence: f64,
    pub asked_user: bool,
    pub matched_pattern_id: Option<i64>,
    pub expires_at: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct NewSpeculation {
    pub timestamp: String,
    pub observed_app: String,
    pub observed_title: String,
    pub inference: String,
    pub confidence: f64,
    pub asked_user: bool,
    pub matched_pattern_id: Option<i64>,
    pub expires_at: String,
}

impl Database {
    pub fn create_speculation(&self, spec: &NewSpeculation) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO speculations (timestamp, observed_app, observed_title, inference, confidence, asked_user, matched_pattern_id, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                spec.timestamp,
                spec.observed_app,
                spec.observed_title,
                spec.inference,
                spec.confidence,
                spec.asked_user as i32,
                spec.matched_pattern_id,
                spec.expires_at,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent_speculations(&self, limit: usize) -> Result<Vec<Speculation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, observed_app, observed_title, inference, confidence,
                    asked_user, matched_pattern_id, expires_at, created_at
             FROM speculations
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let specs = stmt.query_map(params![limit as i64], |row| {
            Ok(Speculation {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                observed_app: row.get(2)?,
                observed_title: row.get(3)?,
                inference: row.get(4)?,
                confidence: row.get(5)?,
                asked_user: row.get::<_, i32>(6)? != 0,
                matched_pattern_id: row.get(7)?,
                expires_at: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        specs.collect()
    }

    pub fn delete_expired_speculations(&self, now: &str) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM speculations WHERE expires_at <= ?1",
            params![now],
        )?;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn sample_speculation() -> NewSpeculation {
        NewSpeculation {
            timestamp: "2026-04-02T10:15:00".to_string(),
            observed_app: "Terminal".to_string(),
            observed_title: "zsh - npm run dev".to_string(),
            inference: "開発サーバーを起動してテスト中".to_string(),
            confidence: 0.7,
            asked_user: false,
            matched_pattern_id: None,
            expires_at: "2026-04-05T10:15:00".to_string(),
        }
    }

    #[test]
    fn test_create_and_get_recent() {
        let db = setup();
        db.create_speculation(&sample_speculation()).unwrap();

        let specs = db.get_recent_speculations(10).unwrap();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].observed_app, "Terminal");
        assert!(!specs[0].asked_user);
    }

    #[test]
    fn test_delete_expired() {
        let db = setup();
        db.create_speculation(&sample_speculation()).unwrap();

        // Not expired yet
        let deleted = db.delete_expired_speculations("2026-04-03T00:00:00").unwrap();
        assert_eq!(deleted, 0);

        // Now expired
        let deleted = db.delete_expired_speculations("2026-04-06T00:00:00").unwrap();
        assert_eq!(deleted, 1);
    }
}
