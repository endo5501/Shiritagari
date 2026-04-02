use rusqlite::{params, Result};

use crate::memory::Database;

impl Database {
    pub fn get_cursor(&self, bucket_id: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT last_event_timestamp FROM polling_cursors WHERE bucket_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![bucket_id], |row| row.get::<_, String>(0))?;
        match rows.next() {
            Some(Ok(ts)) => Ok(Some(ts)),
            _ => Ok(None),
        }
    }

    pub fn update_cursor(&self, bucket_id: &str, last_event_timestamp: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO polling_cursors (bucket_id, last_event_timestamp, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(bucket_id) DO UPDATE SET
               last_event_timestamp = ?2,
               updated_at = datetime('now')",
            params![bucket_id, last_event_timestamp],
        )?;
        Ok(())
    }

    pub fn is_event_processed(&self, event_id: &str, bucket_id: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM processed_events WHERE event_id = ?1 AND bucket_id = ?2",
            params![event_id, bucket_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn mark_event_processed(&self, event_id: &str, bucket_id: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO processed_events (event_id, bucket_id) VALUES (?1, ?2)",
            params![event_id, bucket_id],
        )?;
        Ok(())
    }

    /// Acknowledge multiple events and update cursor in a single transaction.
    pub fn acknowledge_events_tx(&self, events: &[(String, String)], bucket_id: &str, latest_timestamp: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        for (event_id, _bucket) in events {
            tx.execute(
                "INSERT OR IGNORE INTO processed_events (event_id, bucket_id) VALUES (?1, ?2)",
                params![event_id, bucket_id],
            )?;
        }
        tx.execute(
            "INSERT INTO polling_cursors (bucket_id, last_event_timestamp, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(bucket_id) DO UPDATE SET
               last_event_timestamp = ?2,
               updated_at = datetime('now')",
            params![bucket_id, latest_timestamp],
        )?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::Database;

    #[test]
    fn test_cursor_operations() {
        let db = Database::open_in_memory().unwrap();

        assert!(db.get_cursor("aw-watcher-window_host").unwrap().is_none());

        db.update_cursor("aw-watcher-window_host", "2026-04-01T10:00:00")
            .unwrap();
        assert_eq!(
            db.get_cursor("aw-watcher-window_host").unwrap().as_deref(),
            Some("2026-04-01T10:00:00")
        );

        // Update cursor
        db.update_cursor("aw-watcher-window_host", "2026-04-01T11:00:00")
            .unwrap();
        assert_eq!(
            db.get_cursor("aw-watcher-window_host").unwrap().as_deref(),
            Some("2026-04-01T11:00:00")
        );
    }

    #[test]
    fn test_event_deduplication() {
        let db = Database::open_in_memory().unwrap();

        assert!(!db.is_event_processed("evt-1", "bucket-a").unwrap());

        db.mark_event_processed("evt-1", "bucket-a").unwrap();
        assert!(db.is_event_processed("evt-1", "bucket-a").unwrap());

        // Different bucket
        assert!(!db.is_event_processed("evt-1", "bucket-b").unwrap());

        // Duplicate insert should not fail (INSERT OR IGNORE)
        db.mark_event_processed("evt-1", "bucket-a").unwrap();
    }

    #[test]
    fn test_acknowledge_events_tx_atomic() {
        let db = Database::open_in_memory().unwrap();
        let bucket = "aw-watcher-window_host";

        let events = vec![
            ("evt-1".to_string(), bucket.to_string()),
            ("evt-2".to_string(), bucket.to_string()),
            ("evt-3".to_string(), bucket.to_string()),
        ];

        db.acknowledge_events_tx(&events, bucket, "2026-04-01T12:00:00").unwrap();

        // All events should be marked as processed
        assert!(db.is_event_processed("evt-1", bucket).unwrap());
        assert!(db.is_event_processed("evt-2", bucket).unwrap());
        assert!(db.is_event_processed("evt-3", bucket).unwrap());

        // Cursor should be updated
        assert_eq!(
            db.get_cursor(bucket).unwrap().as_deref(),
            Some("2026-04-01T12:00:00")
        );
    }

    #[test]
    fn test_acknowledge_events_tx_idempotent() {
        let db = Database::open_in_memory().unwrap();
        let bucket = "aw-watcher-window_host";

        // First batch
        let events1 = vec![("evt-1".to_string(), bucket.to_string())];
        db.acknowledge_events_tx(&events1, bucket, "2026-04-01T10:00:00").unwrap();

        // Second batch with overlap
        let events2 = vec![
            ("evt-1".to_string(), bucket.to_string()),
            ("evt-2".to_string(), bucket.to_string()),
        ];
        db.acknowledge_events_tx(&events2, bucket, "2026-04-01T11:00:00").unwrap();

        assert!(db.is_event_processed("evt-1", bucket).unwrap());
        assert!(db.is_event_processed("evt-2", bucket).unwrap());
        assert_eq!(
            db.get_cursor(bucket).unwrap().as_deref(),
            Some("2026-04-01T11:00:00")
        );
    }
}
