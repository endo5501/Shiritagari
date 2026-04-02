use rusqlite::Result;

use super::db::Database;
use super::patterns::NewPattern;

impl Database {
    /// Check if an app+title combination has enough episodes to warrant pattern promotion.
    /// If a soft-deleted pattern with the same trigger exists, restore it instead.
    /// Returns the pattern ID if promoted/restored, None otherwise.
    pub fn try_promote_to_pattern(
        &self,
        app: &str,
        title_contains: &str,
        meaning: &str,
        initial_confidence: f64,
        now: &str,
        required_episodes: usize,
    ) -> Result<Option<i64>> {
        let count = self.count_episodes_by_app_and_title(app, title_contains)?;
        if count < required_episodes {
            return Ok(None);
        }

        // Check if there's already an active pattern
        if let Some(existing) = self.find_matching_pattern(app, title_contains)? {
            return Ok(Some(existing.id));
        }

        // Check for soft-deleted pattern to restore
        if let Some(deleted) = self.find_soft_deleted_pattern_by_trigger(app, title_contains)? {
            self.restore_pattern(deleted.id, initial_confidence, now)?;
            return Ok(Some(deleted.id));
        }

        // Create new pattern
        let id = self.create_pattern(&NewPattern {
            trigger_app: app.to_string(),
            trigger_title_contains: title_contains.to_string(),
            trigger_time_range: None,
            trigger_day_of_week: None,
            meaning: meaning.to_string(),
            confidence: initial_confidence,
            last_confirmed: now.to_string(),
        })?;

        Ok(Some(id))
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::{db::Database, episodes::NewEpisode};

    fn setup_with_episodes(db: &Database, count: usize) {
        for i in 0..count {
            db.create_episode(&NewEpisode {
                timestamp: format!("2026-04-0{}T10:00:00", i + 1),
                context_app: "Slack".to_string(),
                context_title: "general - Slack".to_string(),
                context_duration_minutes: Some(30.0),
                question: "Slackで何してる？".to_string(),
                answer: "週次1on1ミーティング".to_string(),
                tags: vec!["slack".to_string(), "meeting".to_string()],
            })
            .unwrap();
        }
    }

    #[test]
    fn test_no_promotion_below_threshold() {
        let db = Database::open_in_memory().unwrap();
        setup_with_episodes(&db, 2);

        let result = db
            .try_promote_to_pattern("Slack", "Slack", "週次1on1", 0.85, "2026-04-03T10:00:00", 3)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_promotion_at_threshold() {
        let db = Database::open_in_memory().unwrap();
        setup_with_episodes(&db, 3);

        let result = db
            .try_promote_to_pattern("Slack", "Slack", "週次1on1", 0.85, "2026-04-03T10:00:00", 3)
            .unwrap();
        assert!(result.is_some());

        let patterns = db.get_all_active_patterns().unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].meaning, "週次1on1");
    }

    #[test]
    fn test_no_duplicate_promotion() {
        let db = Database::open_in_memory().unwrap();
        setup_with_episodes(&db, 5);

        let id1 = db
            .try_promote_to_pattern("Slack", "Slack", "週次1on1", 0.85, "2026-04-03T10:00:00", 3)
            .unwrap()
            .unwrap();
        let id2 = db
            .try_promote_to_pattern("Slack", "Slack", "週次1on1", 0.85, "2026-04-04T10:00:00", 3)
            .unwrap()
            .unwrap();

        assert_eq!(id1, id2);
        assert_eq!(db.get_all_active_patterns().unwrap().len(), 1);
    }

    #[test]
    fn test_restore_soft_deleted_on_promotion() {
        let db = Database::open_in_memory().unwrap();
        setup_with_episodes(&db, 3);

        // Create and soft-delete a pattern
        let id = db
            .try_promote_to_pattern("Slack", "Slack", "週次1on1", 0.85, "2026-04-01T10:00:00", 3)
            .unwrap()
            .unwrap();
        db.soft_delete_pattern(id, "2026-04-02T10:00:00").unwrap();

        // Add more episodes and try promote again
        setup_with_episodes(&db, 1);
        let restored_id = db
            .try_promote_to_pattern("Slack", "Slack", "週次1on1", 0.80, "2026-04-03T10:00:00", 3)
            .unwrap()
            .unwrap();

        assert_eq!(id, restored_id);
        let patterns = db.get_all_active_patterns().unwrap();
        assert_eq!(patterns.len(), 1);
        assert!(patterns[0].deleted_at.is_none());
    }
}
