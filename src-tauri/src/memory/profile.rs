use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use super::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub occupation: Option<String>,
    pub skills: Vec<String>,
    pub interests: Vec<String>,
    pub notes: String,
    pub updated_at: String,
}

impl Database {
    pub fn get_user_profile(&self) -> Result<Option<UserProfile>> {
        let mut stmt = self.conn.prepare(
            "SELECT occupation, skills, interests, notes, updated_at FROM user_profile WHERE id = 1",
        )?;

        let mut rows = stmt.query_map([], |row| {
            let skills_str: String = row.get(1)?;
            let interests_str: String = row.get(2)?;
            Ok(UserProfile {
                occupation: row.get(0)?,
                skills: serde_json::from_str(&skills_str).unwrap_or_default(),
                interests: serde_json::from_str(&interests_str).unwrap_or_default(),
                notes: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;

        match rows.next() {
            Some(Ok(p)) => Ok(Some(p)),
            _ => Ok(None),
        }
    }

    pub fn upsert_user_profile(
        &self,
        occupation: Option<&str>,
        skills: &[String],
        interests: &[String],
        notes: &str,
    ) -> Result<()> {
        let skills_json = serde_json::to_string(skills).unwrap_or_else(|_| "[]".to_string());
        let interests_json = serde_json::to_string(interests).unwrap_or_else(|_| "[]".to_string());

        self.conn.execute(
            "INSERT INTO user_profile (id, occupation, skills, interests, notes, updated_at)
             VALUES (1, ?1, ?2, ?3, ?4, datetime('now'))
             ON CONFLICT(id) DO UPDATE SET
               occupation = ?1,
               skills = ?2,
               interests = ?3,
               notes = ?4,
               updated_at = datetime('now')",
            params![occupation, skills_json, interests_json, notes],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Database {
        Database::open_in_memory().unwrap()
    }

    #[test]
    fn test_no_profile_initially() {
        let db = setup();
        let profile = db.get_user_profile().unwrap();
        assert!(profile.is_none());
    }

    #[test]
    fn test_create_and_get_profile() {
        let db = setup();
        db.upsert_user_profile(
            Some("ソフトウェアエンジニア"),
            &["Rust".to_string(), "Python".to_string()],
            &["AI".to_string()],
            "Shiritagariの開発者",
        )
        .unwrap();

        let profile = db.get_user_profile().unwrap().unwrap();
        assert_eq!(profile.occupation.as_deref(), Some("ソフトウェアエンジニア"));
        assert_eq!(profile.skills, vec!["Rust", "Python"]);
        assert_eq!(profile.interests, vec!["AI"]);
    }

    #[test]
    fn test_update_profile() {
        let db = setup();
        db.upsert_user_profile(Some("学生"), &[], &[], "").unwrap();
        db.upsert_user_profile(Some("エンジニア"), &["Go".to_string()], &[], "転職した")
            .unwrap();

        let profile = db.get_user_profile().unwrap().unwrap();
        assert_eq!(profile.occupation.as_deref(), Some("エンジニア"));
        assert_eq!(profile.skills, vec!["Go"]);
        assert_eq!(profile.notes, "転職した");
    }
}
