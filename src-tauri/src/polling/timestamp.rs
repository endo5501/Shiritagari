use chrono::{DateTime, Duration, FixedOffset};

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
}
