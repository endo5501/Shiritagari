use chrono::{Duration, Utc};
use rusqlite::Result;

use super::db::Database;
use super::patterns::NewPattern;

pub struct CleanupResult {
    pub episodes_deleted: usize,
    pub speculations_deleted: usize,
    pub patterns_purged: usize,
    pub speculations_promoted: usize,
}

const SPECULATION_PROMOTION_MIN_COUNT: i64 = 6;
const SPECULATION_PROMOTION_CONFIDENCE: f64 = 0.75;

impl Database {
    /// Check for speculation groups that meet the promotion threshold
    /// and promote them to patterns using exact trigger matching.
    /// Returns the number of new patterns created or restored.
    pub fn promote_speculations_to_patterns(&self, required_count: i64) -> Result<usize> {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        let candidates = self.get_speculation_promotion_candidates(required_count, &now)?;

        let mut promoted = 0;
        for (app, title, inference) in &candidates {
            // Use exact matching to avoid broader patterns blocking specific promotions
            if self.find_exact_active_pattern(app, title)?.is_some() {
                continue;
            }

            // Restore soft-deleted pattern if one exists with the same exact trigger
            if let Some(deleted) = self.find_soft_deleted_pattern_by_trigger(app, title)? {
                self.restore_pattern(deleted.id, SPECULATION_PROMOTION_CONFIDENCE, &now)?;
                promoted += 1;
                continue;
            }

            // Create new pattern
            self.create_pattern(&NewPattern {
                trigger_app: app.to_string(),
                trigger_title_contains: title.to_string(),
                trigger_time_range: None,
                trigger_day_of_week: None,
                meaning: inference.to_string(),
                confidence: SPECULATION_PROMOTION_CONFIDENCE,
                last_confirmed: now.to_string(),
            })?;
            promoted += 1;
        }

        Ok(promoted)
    }

    pub fn run_cleanup(&self) -> Result<CleanupResult> {
        let now = Utc::now();
        let now_str = now.format("%Y-%m-%dT%H:%M:%S").to_string();

        // Promote speculations before deleting expired ones
        let speculations_promoted = self.promote_speculations_to_patterns(SPECULATION_PROMOTION_MIN_COUNT)?;

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
            speculations_promoted,
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
    fn test_promote_speculations_basic() {
        let db = Database::open_in_memory().unwrap();

        // Create 6 speculations with same app+title
        for i in 0..6 {
            db.create_speculation(&NewSpeculation {
                timestamp: format!("2026-04-01T10:{:02}:00", i),
                observed_app: "VS Code".to_string(),
                observed_title: "main.rs".to_string(),
                inference: format!("Rust development {}", i),
                confidence: 0.7,
                asked_user: false,
                matched_pattern_id: None,
                expires_at: "2026-04-10T10:00:00".to_string(),
            })
            .unwrap();
        }

        let promoted = db.promote_speculations_to_patterns(6).unwrap();
        assert_eq!(promoted, 1);

        let patterns = db.get_all_active_patterns().unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].trigger_app, "VS Code");
        assert_eq!(patterns[0].trigger_title_contains, "main.rs");
        assert!(patterns[0].meaning.contains("Rust development 5")); // latest
    }

    #[test]
    fn test_promote_speculations_below_threshold() {
        let db = Database::open_in_memory().unwrap();

        // Only 5 speculations — not enough
        for i in 0..5 {
            db.create_speculation(&NewSpeculation {
                timestamp: format!("2026-04-01T10:{:02}:00", i),
                observed_app: "Chrome".to_string(),
                observed_title: "docs".to_string(),
                inference: format!("reading docs {}", i),
                confidence: 0.7,
                asked_user: false,
                matched_pattern_id: None,
                expires_at: "2026-04-10T10:00:00".to_string(),
            })
            .unwrap();
        }

        let promoted = db.promote_speculations_to_patterns(6).unwrap();
        assert_eq!(promoted, 0);
        assert!(db.get_all_active_patterns().unwrap().is_empty());
    }

    #[test]
    fn test_promote_speculations_no_duplicate() {
        let db = Database::open_in_memory().unwrap();

        // Create an existing pattern
        db.create_pattern(&NewPattern {
            trigger_app: "VS Code".to_string(),
            trigger_title_contains: "main.rs".to_string(),
            trigger_time_range: None,
            trigger_day_of_week: None,
            meaning: "existing pattern".to_string(),
            confidence: 0.9,
            last_confirmed: "2026-04-01T10:00:00".to_string(),
        })
        .unwrap();

        // Create 6 speculations with same trigger
        for i in 0..6 {
            db.create_speculation(&NewSpeculation {
                timestamp: format!("2026-04-01T10:{:02}:00", i),
                observed_app: "VS Code".to_string(),
                observed_title: "main.rs".to_string(),
                inference: format!("coding {}", i),
                confidence: 0.7,
                asked_user: false,
                matched_pattern_id: None,
                expires_at: "2026-04-10T10:00:00".to_string(),
            })
            .unwrap();
        }

        let promoted = db.promote_speculations_to_patterns(6).unwrap();
        assert_eq!(promoted, 0); // no new pattern created, existing one returned

        let patterns = db.get_all_active_patterns().unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].meaning, "existing pattern"); // unchanged
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
