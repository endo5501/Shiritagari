use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use super::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub timestamp: String,
    pub context_app: String,
    pub context_title: String,
    pub context_duration_minutes: Option<f64>,
    pub question: String,
    pub answer: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug)]
pub struct NewEpisode {
    pub timestamp: String,
    pub context_app: String,
    pub context_title: String,
    pub context_duration_minutes: Option<f64>,
    pub question: String,
    pub answer: String,
    pub tags: Vec<String>,
}

impl Episode {
    fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Episode> {
        let tags_str: String = row.get(7)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        Ok(Episode {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            context_app: row.get(2)?,
            context_title: row.get(3)?,
            context_duration_minutes: row.get(4)?,
            question: row.get(5)?,
            answer: row.get(6)?,
            tags,
            created_at: row.get(8)?,
        })
    }
}

impl Database {
    pub fn create_episode(&self, episode: &NewEpisode) -> Result<i64> {
        let tags_json = serde_json::to_string(&episode.tags).unwrap_or_else(|_| "[]".to_string());
        self.conn.execute(
            "INSERT INTO episodes (timestamp, context_app, context_title, context_duration_minutes, question, answer, tags)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                episode.timestamp,
                episode.context_app,
                episode.context_title,
                episode.context_duration_minutes,
                episode.question,
                episode.answer,
                tags_json,
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_recent_episodes(&self, limit: usize) -> Result<Vec<Episode>> {
        self.get_episodes_paginated(limit as i64, 0)
    }

    pub fn find_episodes_by_app(&self, app: &str) -> Result<Vec<Episode>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, context_app, context_title, context_duration_minutes,
                    question, answer, tags, created_at
             FROM episodes
             WHERE context_app = ?1
             ORDER BY timestamp DESC",
        )?;

        let episodes = stmt.query_map(params![app], Episode::from_row)?;

        episodes.collect()
    }

    pub fn find_episodes_by_tag(&self, tag: &str) -> Result<Vec<Episode>> {
        let pattern = format!("%\"{}\"% ", tag);
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, context_app, context_title, context_duration_minutes,
                    question, answer, tags, created_at
             FROM episodes
             WHERE tags LIKE ?1
             ORDER BY timestamp DESC",
        )?;

        let episodes = stmt.query_map(params![pattern], Episode::from_row)?;

        episodes.collect()
    }

    pub fn count_episodes_by_app_and_title(&self, app: &str, title_contains: &str) -> Result<usize> {
        let pattern = format!("%{}%", title_contains);
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM episodes WHERE context_app = ?1 AND context_title LIKE ?2",
            params![app, pattern],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn count_episodes(&self) -> Result<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM episodes",
            [],
            |row| row.get(0),
        )
    }

    pub fn get_episodes_paginated(&self, limit: i64, offset: i64) -> Result<Vec<Episode>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, context_app, context_title, context_duration_minutes,
                    question, answer, tags, created_at
             FROM episodes
             ORDER BY timestamp DESC
             LIMIT ?1 OFFSET ?2",
        )?;

        let episodes = stmt.query_map(params![limit, offset], Episode::from_row)?;

        episodes.collect()
    }

    pub fn delete_episodes_older_than(&self, cutoff_date: &str) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM episodes WHERE created_at <= ?1",
            params![cutoff_date],
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

    fn sample_episode() -> NewEpisode {
        NewEpisode {
            timestamp: "2026-04-01T14:00:00".to_string(),
            context_app: "Chrome".to_string(),
            context_title: "Q3計画 - Google Docs".to_string(),
            context_duration_minutes: Some(45.0),
            question: "Google Docsで何を書いていますか？".to_string(),
            answer: "来期のチーム計画をまとめてる".to_string(),
            tags: vec!["docs".to_string(), "planning".to_string()],
        }
    }

    #[test]
    fn test_create_and_get_recent() {
        let db = setup();
        db.create_episode(&sample_episode()).unwrap();

        let episodes = db.get_recent_episodes(10).unwrap();
        assert_eq!(episodes.len(), 1);
        assert_eq!(episodes[0].context_app, "Chrome");
        assert_eq!(episodes[0].tags, vec!["docs", "planning"]);
    }

    #[test]
    fn test_find_by_app() {
        let db = setup();
        db.create_episode(&sample_episode()).unwrap();

        let found = db.find_episodes_by_app("Chrome").unwrap();
        assert_eq!(found.len(), 1);

        let not_found = db.find_episodes_by_app("VS Code").unwrap();
        assert!(not_found.is_empty());
    }

    #[test]
    fn test_count_by_app_and_title() {
        let db = setup();
        db.create_episode(&sample_episode()).unwrap();

        let count = db.count_episodes_by_app_and_title("Chrome", "Google Docs").unwrap();
        assert_eq!(count, 1);

        let count = db.count_episodes_by_app_and_title("Chrome", "Slack").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_episodes() {
        let db = setup();
        assert_eq!(db.count_episodes().unwrap(), 0);

        db.create_episode(&sample_episode()).unwrap();
        assert_eq!(db.count_episodes().unwrap(), 1);

        db.create_episode(&NewEpisode {
            timestamp: "2026-04-02T10:00:00".to_string(),
            context_app: "VS Code".to_string(),
            context_title: "main.rs".to_string(),
            context_duration_minutes: Some(30.0),
            question: "何を開発中？".to_string(),
            answer: "新機能の実装".to_string(),
            tags: vec!["dev".to_string()],
        }).unwrap();
        assert_eq!(db.count_episodes().unwrap(), 2);
    }

    #[test]
    fn test_get_episodes_paginated() {
        let db = setup();

        // Create 3 episodes
        for i in 0..3 {
            db.create_episode(&NewEpisode {
                timestamp: format!("2026-04-0{}T10:00:00", i + 1),
                context_app: format!("App{}", i),
                context_title: "title".to_string(),
                context_duration_minutes: Some(10.0),
                question: format!("Q{}", i),
                answer: format!("A{}", i),
                tags: vec![],
            }).unwrap();
        }

        // Page 1 with limit 2 (newest first)
        let page1 = db.get_episodes_paginated(2, 0).unwrap();
        assert_eq!(page1.len(), 2);
        assert_eq!(page1[0].context_app, "App2"); // newest first

        // Page 2 with limit 2
        let page2 = db.get_episodes_paginated(2, 2).unwrap();
        assert_eq!(page2.len(), 1);
        assert_eq!(page2[0].context_app, "App0"); // oldest

        // Beyond range
        let page3 = db.get_episodes_paginated(2, 4).unwrap();
        assert!(page3.is_empty());
    }

    #[test]
    fn test_delete_old_episodes() {
        let db = setup();
        db.create_episode(&sample_episode()).unwrap();

        let deleted = db.delete_episodes_older_than("2099-01-01T00:00:00").unwrap();
        assert_eq!(deleted, 1);

        let episodes = db.get_recent_episodes(10).unwrap();
        assert!(episodes.is_empty());
    }
}
