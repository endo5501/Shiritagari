use chrono::{DateTime, Duration, Utc};

/// Determine the effective start timestamp for event fetching.
///
/// Returns the later of `cursor` and `now - lookback_minutes`.
/// If cursor is missing or unparsable, uses the time-based limit.
pub fn effective_start(cursor: Option<&str>, now: DateTime<Utc>, lookback_minutes: u64) -> String {
    let time_limit = now - Duration::minutes(lookback_minutes as i64);
    let time_limit_str = time_limit.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string();

    match cursor {
        Some(c) => match DateTime::parse_from_rfc3339(c) {
            Ok(cursor_dt) if cursor_dt > time_limit => c.to_string(),
            _ => time_limit_str,
        },
        None => time_limit_str,
    }
}

/// Advance an ISO 8601 timestamp by 1 millisecond.
///
/// Used to make pagination cursors exclusive when the API's `start` parameter is inclusive.
/// Returns the original timestamp unchanged if parsing fails.
pub fn advance_timestamp_1ms(timestamp: &str) -> String {
    match DateTime::parse_from_rfc3339(timestamp) {
        Ok(dt) => {
            let advanced = dt + Duration::milliseconds(1);
            advanced.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
        }
        Err(_) => timestamp.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advance_with_milliseconds_and_offset() {
        let result = advance_timestamp_1ms("2026-04-04T01:58:46.123+00:00");
        assert_eq!(result, "2026-04-04T01:58:46.124+00:00");
    }

    #[test]
    fn test_advance_without_milliseconds_offset() {
        let result = advance_timestamp_1ms("2026-04-04T01:58:46+00:00");
        assert_eq!(result, "2026-04-04T01:58:46.001+00:00");
    }

    #[test]
    fn test_advance_with_milliseconds_z() {
        let result = advance_timestamp_1ms("2026-04-04T01:58:46.123Z");
        assert_eq!(result, "2026-04-04T01:58:46.124+00:00");
    }

    #[test]
    fn test_advance_without_milliseconds_z() {
        let result = advance_timestamp_1ms("2026-04-04T01:58:46Z");
        assert_eq!(result, "2026-04-04T01:58:46.001+00:00");
    }

    #[test]
    fn test_advance_with_positive_offset() {
        let result = advance_timestamp_1ms("2026-04-04T10:58:46.500+09:00");
        assert_eq!(result, "2026-04-04T10:58:46.501+09:00");
    }

    #[test]
    fn test_advance_millisecond_rollover() {
        let result = advance_timestamp_1ms("2026-04-04T01:58:46.999+00:00");
        assert_eq!(result, "2026-04-04T01:58:47.000+00:00");
    }

    #[test]
    fn test_advance_second_rollover() {
        let result = advance_timestamp_1ms("2026-04-04T01:58:59.999+00:00");
        assert_eq!(result, "2026-04-04T01:59:00.000+00:00");
    }

    #[test]
    fn test_advance_preserves_timezone_offset() {
        let result = advance_timestamp_1ms("2026-04-04T10:00:00.000+09:00");
        assert_eq!(result, "2026-04-04T10:00:00.001+09:00");
    }

    #[test]
    fn test_advance_malformed_timestamp_returns_original() {
        let result = advance_timestamp_1ms("not-a-timestamp");
        assert_eq!(result, "not-a-timestamp");
    }

    #[test]
    fn test_advance_missing_offset_returns_original() {
        let result = advance_timestamp_1ms("2026-04-04T01:58:46");
        assert_eq!(result, "2026-04-04T01:58:46");
    }

    #[test]
    fn test_effective_start_no_cursor_returns_time_limit() {
        let now = DateTime::parse_from_rfc3339("2026-04-04T04:00:00.000+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let result = effective_start(None, now, 30);
        // 30 minutes before now
        assert_eq!(result, "2026-04-04T03:30:00.000+00:00");
    }

    #[test]
    fn test_effective_start_recent_cursor_returns_cursor() {
        let now = DateTime::parse_from_rfc3339("2026-04-04T04:00:00.000+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let cursor = Some("2026-04-04T03:50:00.000+00:00".to_string());
        let result = effective_start(cursor.as_deref(), now, 30);
        // Cursor is within 30 minutes, so use cursor
        assert_eq!(result, "2026-04-04T03:50:00.000+00:00");
    }

    #[test]
    fn test_effective_start_old_cursor_returns_time_limit() {
        let now = DateTime::parse_from_rfc3339("2026-04-04T04:00:00.000+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let cursor = Some("2026-04-04T01:00:00.000+00:00".to_string());
        let result = effective_start(cursor.as_deref(), now, 30);
        // Cursor is 3 hours old, so use time limit
        assert_eq!(result, "2026-04-04T03:30:00.000+00:00");
    }

    #[test]
    fn test_effective_start_malformed_cursor_returns_time_limit() {
        let now = DateTime::parse_from_rfc3339("2026-04-04T04:00:00.000+00:00")
            .unwrap()
            .with_timezone(&Utc);
        let cursor = Some("not-a-timestamp".to_string());
        let result = effective_start(cursor.as_deref(), now, 30);
        assert_eq!(result, "2026-04-04T03:30:00.000+00:00");
    }
}
