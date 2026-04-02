use chrono::{Duration, Utc};
use rusqlite::Result;

use super::db::Database;

pub struct CleanupResult {
    pub episodes_deleted: usize,
    pub speculations_deleted: usize,
    pub patterns_purged: usize,
}

impl Database {
    pub fn run_cleanup(&self) -> Result<CleanupResult> {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();

        // Episodes older than 1 month
        let one_month_ago = (now - Duration::days(30))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();
        let episodes_deleted = self.delete_episodes_older_than(&one_month_ago)?;

        // Expired speculations
        let speculations_deleted = self.delete_expired_speculations(&now_str)?;

        // Soft-deleted patterns older than 30 days
        let thirty_days_ago = (now - Duration::days(30))
            .format("%Y-%m-%dT%H:%M:%S")
            .to_string();
        let patterns_purged = self.purge_expired_soft_deleted_patterns(&thirty_days_ago)?;

        Ok(CleanupResult {
            episodes_deleted,
            speculations_deleted,
            patterns_purged,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{
        db::Database,
        episodes::NewEpisode,
        patterns::NewPattern,
        speculations::NewSpeculation,
    };

    #[test]
    fn test_cleanup_deletes_old_data() {
        let db = Database::open_in_memory().unwrap();

        // Create old episode (simulate old created_at by direct SQL)
        db.create_episode(&NewEpisode {
            timestamp: "2026-02-01T10:00:00".to_string(),
            context_app: "Chrome".to_string(),
            context_title: "old page".to_string(),
            context_duration_minutes: None,
            question: "q".to_string(),
            answer: "a".to_string(),
            tags: vec![],
        })
        .unwrap();
        db.conn
            .execute(
                "UPDATE episodes SET created_at = '2026-02-01T10:00:00' WHERE id = 1",
                [],
            )
            .unwrap();

        // Create expired speculation
        db.create_speculation(&NewSpeculation {
            timestamp: "2026-03-01T10:00:00".to_string(),
            observed_app: "Terminal".to_string(),
            observed_title: "test".to_string(),
            inference: "testing".to_string(),
            confidence: 0.5,
            asked_user: false,
            matched_pattern_id: None,
            expires_at: "2026-03-04T10:00:00".to_string(),
        })
        .unwrap();

        // Create soft-deleted pattern (old)
        let pid = db
            .create_pattern(&NewPattern {
                trigger_app: "VS Code".to_string(),
                trigger_title_contains: "old".to_string(),
                trigger_time_range: None,
                trigger_day_of_week: None,
                meaning: "old work".to_string(),
                confidence: 0.2,
                last_confirmed: "2026-01-01T10:00:00".to_string(),
            })
            .unwrap();
        db.soft_delete_pattern(pid, "2026-02-01T10:00:00").unwrap();

        let result = db.run_cleanup().unwrap();
        assert_eq!(result.episodes_deleted, 1);
        assert_eq!(result.speculations_deleted, 1);
        assert_eq!(result.patterns_purged, 1);
    }

    #[test]
    fn test_cleanup_preserves_recent_data() {
        let db = Database::open_in_memory().unwrap();

        // Recent episode (created_at defaults to now)
        db.create_episode(&NewEpisode {
            timestamp: "2026-04-02T10:00:00".to_string(),
            context_app: "Chrome".to_string(),
            context_title: "new page".to_string(),
            context_duration_minutes: None,
            question: "q".to_string(),
            answer: "a".to_string(),
            tags: vec![],
        })
        .unwrap();

        // Not-yet-expired speculation
        db.create_speculation(&NewSpeculation {
            timestamp: "2026-04-02T10:00:00".to_string(),
            observed_app: "Terminal".to_string(),
            observed_title: "test".to_string(),
            inference: "testing".to_string(),
            confidence: 0.5,
            asked_user: false,
            matched_pattern_id: None,
            expires_at: "2099-04-05T10:00:00".to_string(),
        })
        .unwrap();

        let result = db.run_cleanup().unwrap();
        assert_eq!(result.episodes_deleted, 0);
        assert_eq!(result.speculations_deleted, 0);
        assert_eq!(result.patterns_purged, 0);
    }
}
