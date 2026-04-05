use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use super::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: i64,
    pub trigger_app: String,
    pub trigger_title_contains: String,
    pub trigger_time_range: Option<String>,
    pub trigger_day_of_week: Option<String>,
    pub meaning: String,
    pub confidence: f64,
    pub last_confirmed: String,
    pub deleted_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug)]
pub struct NewPattern {
    pub trigger_app: String,
    pub trigger_title_contains: String,
    pub trigger_time_range: Option<String>,
    pub trigger_day_of_week: Option<String>,
    pub meaning: String,
    pub confidence: f64,
    pub last_confirmed: String,
}

impl Pattern {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Pattern> {
        Ok(Pattern {
            id: row.get(0)?,
            trigger_app: row.get(1)?,
            trigger_title_contains: row.get(2)?,
            trigger_time_range: row.get(3)?,
            trigger_day_of_week: row.get(4)?,
            meaning: row.get(5)?,
            confidence: row.get(6)?,
            last_confirmed: row.get(7)?,
            deleted_at: row.get(8)?,
            created_at: row.get(9)?,
        })
    }
}

impl Database {
    pub fn create_pattern(&self, pattern: &NewPattern) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO patterns (trigger_app, trigger_title_contains, trigger_time_range, trigger_day_of_week, meaning, confidence, last_confirmed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                pattern.trigger_app,
                pattern.trigger_title_contains,
                pattern.trigger_time_range,
                pattern.trigger_day_of_week,
                pattern.meaning,
                pattern.confidence,
                pattern.last_confirmed,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn find_matching_pattern(&self, app: &str, title: &str) -> Result<Option<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, trigger_app, trigger_title_contains, trigger_time_range, trigger_day_of_week,
                    meaning, confidence, last_confirmed, deleted_at, created_at
             FROM patterns
             WHERE deleted_at IS NULL
               AND trigger_app = ?1
             ORDER BY confidence DESC",
        )?;

        let patterns = stmt.query_map(params![app], Pattern::from_row)?;

        for pattern in patterns {
            let p = pattern?;
            if p.trigger_title_contains.is_empty()
                || title.to_lowercase().contains(&p.trigger_title_contains.to_lowercase())
            {
                return Ok(Some(p));
            }
        }

        Ok(None)
    }

    pub fn get_all_active_patterns(&self) -> Result<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, trigger_app, trigger_title_contains, trigger_time_range, trigger_day_of_week,
                    meaning, confidence, last_confirmed, deleted_at, created_at
             FROM patterns
             WHERE deleted_at IS NULL",
        )?;

        let patterns = stmt.query_map([], Pattern::from_row)?;

        patterns.collect()
    }

    pub fn update_pattern_confidence(&self, id: i64, confidence: f64, last_confirmed: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE patterns SET confidence = ?1, last_confirmed = ?2 WHERE id = ?3",
            params![confidence, last_confirmed, id],
        )?;
        Ok(())
    }

    pub fn soft_delete_pattern(&self, id: i64, deleted_at: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE patterns SET deleted_at = ?1 WHERE id = ?2",
            params![deleted_at, id],
        )?;
        Ok(())
    }

    pub fn restore_pattern(&self, id: i64, new_confidence: f64, last_confirmed: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE patterns SET deleted_at = NULL, confidence = ?1, last_confirmed = ?2 WHERE id = ?3",
            params![new_confidence, last_confirmed, id],
        )?;
        Ok(())
    }

    pub fn purge_expired_soft_deleted_patterns(&self, cutoff_date: &str) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM patterns WHERE deleted_at IS NOT NULL AND deleted_at <= ?1",
            params![cutoff_date],
        )?;
        Ok(count)
    }

    pub fn find_exact_active_pattern(&self, app: &str, title_contains: &str) -> Result<Option<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, trigger_app, trigger_title_contains, trigger_time_range, trigger_day_of_week,
                    meaning, confidence, last_confirmed, deleted_at, created_at
             FROM patterns
             WHERE deleted_at IS NULL
               AND trigger_app = ?1
               AND trigger_title_contains = ?2
             LIMIT 1",
        )?;

        let mut rows = stmt.query_map(params![app, title_contains], Pattern::from_row)?;

        match rows.next() {
            Some(Ok(p)) => Ok(Some(p)),
            _ => Ok(None),
        }
    }

    pub fn count_active_patterns(&self) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM patterns WHERE deleted_at IS NULL",
            [],
            |row| row.get(0),
        )
    }

    pub fn get_active_patterns_paginated(&self, limit: i64, offset: i64) -> Result<Vec<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, trigger_app, trigger_title_contains, trigger_time_range, trigger_day_of_week,
                    meaning, confidence, last_confirmed, deleted_at, created_at
             FROM patterns
             WHERE deleted_at IS NULL
             ORDER BY id ASC
             LIMIT ?1 OFFSET ?2",
        )?;

        let patterns = stmt.query_map(params![limit, offset], Pattern::from_row)?;

        patterns.collect()
    }

    pub fn find_soft_deleted_pattern_by_trigger(&self, app: &str, title_contains: &str) -> Result<Option<Pattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, trigger_app, trigger_title_contains, trigger_time_range, trigger_day_of_week,
                    meaning, confidence, last_confirmed, deleted_at, created_at
             FROM patterns
             WHERE deleted_at IS NOT NULL
               AND trigger_app = ?1
               AND trigger_title_contains = ?2
             LIMIT 1",
        )?;

        let mut rows = stmt.query_map(params![app, title_contains], Pattern::from_row)?;

        match rows.next() {
            Some(Ok(p)) => Ok(Some(p)),
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Database {
        Database::open_in_memory().unwrap()
    }

    fn sample_pattern() -> NewPattern {
        NewPattern {
            trigger_app: "VS Code".to_string(),
            trigger_title_contains: "shiritagari".to_string(),
            trigger_time_range: None,
            trigger_day_of_week: None,
            meaning: "Shiritagariプロジェクトの開発".to_string(),
            confidence: 0.9,
            last_confirmed: "2026-04-01T10:00:00".to_string(),
        }
    }

    #[test]
    fn test_create_and_find_pattern() {
        let db = setup();
        let id = db.create_pattern(&sample_pattern()).unwrap();
        assert!(id > 0);

        let found = db.find_matching_pattern("VS Code", "main.rs - shiritagari").unwrap();
        assert!(found.is_some());
        let p = found.unwrap();
        assert_eq!(p.meaning, "Shiritagariプロジェクトの開発");
    }

    #[test]
    fn test_no_match_wrong_app() {
        let db = setup();
        db.create_pattern(&sample_pattern()).unwrap();

        let found = db.find_matching_pattern("Chrome", "shiritagari").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_no_match_wrong_title() {
        let db = setup();
        db.create_pattern(&sample_pattern()).unwrap();

        let found = db.find_matching_pattern("VS Code", "other-project").unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_update_confidence() {
        let db = setup();
        let id = db.create_pattern(&sample_pattern()).unwrap();

        db.update_pattern_confidence(id, 0.95, "2026-04-02T10:00:00").unwrap();

        let found = db.find_matching_pattern("VS Code", "shiritagari").unwrap().unwrap();
        assert!((found.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_soft_delete_and_restore() {
        let db = setup();
        let id = db.create_pattern(&sample_pattern()).unwrap();

        db.soft_delete_pattern(id, "2026-04-02T10:00:00").unwrap();

        // Should not be found in active patterns
        let found = db.find_matching_pattern("VS Code", "shiritagari").unwrap();
        assert!(found.is_none());

        let active = db.get_all_active_patterns().unwrap();
        assert!(active.is_empty());

        // Should be found in soft-deleted
        let soft = db.find_soft_deleted_pattern_by_trigger("VS Code", "shiritagari").unwrap();
        assert!(soft.is_some());

        // Restore
        db.restore_pattern(id, 0.7, "2026-04-02T11:00:00").unwrap();
        let found = db.find_matching_pattern("VS Code", "shiritagari").unwrap();
        assert!(found.is_some());
    }

    #[test]
    fn test_count_active_patterns() {
        let db = setup();
        assert_eq!(db.count_active_patterns().unwrap(), 0);

        db.create_pattern(&sample_pattern()).unwrap();
        assert_eq!(db.count_active_patterns().unwrap(), 1);

        let id2 = db.create_pattern(&NewPattern {
            trigger_app: "Chrome".to_string(),
            trigger_title_contains: "docs".to_string(),
            trigger_time_range: None,
            trigger_day_of_week: None,
            meaning: "ドキュメント閲覧".to_string(),
            confidence: 0.8,
            last_confirmed: "2026-04-01T10:00:00".to_string(),
        }).unwrap();
        assert_eq!(db.count_active_patterns().unwrap(), 2);

        // Soft-deleted patterns should not be counted
        db.soft_delete_pattern(id2, "2026-04-02T00:00:00").unwrap();
        assert_eq!(db.count_active_patterns().unwrap(), 1);
    }

    #[test]
    fn test_get_active_patterns_paginated() {
        let db = setup();

        // Create 3 patterns
        for i in 0..3 {
            db.create_pattern(&NewPattern {
                trigger_app: format!("App{}", i),
                trigger_title_contains: "".to_string(),
                trigger_time_range: None,
                trigger_day_of_week: None,
                meaning: format!("Meaning {}", i),
                confidence: 0.9,
                last_confirmed: "2026-04-01T10:00:00".to_string(),
            }).unwrap();
        }

        // Page 1 with limit 2
        let page1 = db.get_active_patterns_paginated(2, 0).unwrap();
        assert_eq!(page1.len(), 2);

        // Page 2 with limit 2
        let page2 = db.get_active_patterns_paginated(2, 2).unwrap();
        assert_eq!(page2.len(), 1);

        // Beyond range
        let page3 = db.get_active_patterns_paginated(2, 4).unwrap();
        assert!(page3.is_empty());
    }

    #[test]
    fn test_purge_expired() {
        let db = setup();
        let id = db.create_pattern(&sample_pattern()).unwrap();
        db.soft_delete_pattern(id, "2026-03-01T00:00:00").unwrap();

        let purged = db.purge_expired_soft_deleted_patterns("2026-04-01T00:00:00").unwrap();
        assert_eq!(purged, 1);
    }
}
