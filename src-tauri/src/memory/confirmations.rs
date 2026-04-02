use rusqlite::{params, Result};

use super::db::Database;

impl Database {
    pub fn is_confirmed(&self, key: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM user_confirmations WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn set_confirmed(&self, key: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO user_confirmations (key) VALUES (?1)",
            params![key],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::memory::Database;

    #[test]
    fn test_confirmation_flow() {
        let db = Database::open_in_memory().unwrap();

        assert!(!db.is_confirmed("external_api_claude").unwrap());

        db.set_confirmed("external_api_claude").unwrap();
        assert!(db.is_confirmed("external_api_claude").unwrap());

        // Different key is not confirmed
        assert!(!db.is_confirmed("external_api_openai").unwrap());
    }

    #[test]
    fn test_set_confirmed_idempotent() {
        let db = Database::open_in_memory().unwrap();

        db.set_confirmed("external_api_claude").unwrap();
        db.set_confirmed("external_api_claude").unwrap(); // Should not fail
        assert!(db.is_confirmed("external_api_claude").unwrap());
    }
}
