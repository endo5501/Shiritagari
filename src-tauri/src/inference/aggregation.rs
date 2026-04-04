use std::collections::HashMap;

use crate::providers::types::AggregatedEvent;

/// Aggregate events by (app, title), summing durations and tracking the latest timestamp.
/// Returns results sorted by last_active descending, limited to `max_entries`.
pub fn aggregate_events(
    events: &[(String, String, f64, String)], // (app, title, duration, timestamp)
    max_entries: usize,
) -> Vec<AggregatedEvent> {
    let mut map: HashMap<(String, String), (f64, String)> = HashMap::new();

    for (app, title, duration, timestamp) in events {
        let key = (app.clone(), title.clone());
        let entry = map.entry(key).or_insert((0.0, String::new()));
        entry.0 += duration;
        if timestamp > &entry.1 {
            entry.1 = timestamp.clone();
        }
    }

    let mut aggregated: Vec<AggregatedEvent> = map
        .into_iter()
        .map(|((app, title), (total_duration, last_active))| AggregatedEvent {
            app,
            title,
            total_duration_seconds: total_duration,
            last_active,
        })
        .collect();

    aggregated.sort_by(|a, b| b.last_active.cmp(&a.last_active));
    aggregated.truncate(max_entries);
    aggregated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_groups_by_app_title() {
        let events = vec![
            ("Code".to_string(), "main.rs".to_string(), 3.0, "2026-04-04T03:50:00+00:00".to_string()),
            ("Code".to_string(), "main.rs".to_string(), 5.0, "2026-04-04T03:51:00+00:00".to_string()),
            ("Code".to_string(), "lib.rs".to_string(), 2.0, "2026-04-04T03:52:00+00:00".to_string()),
        ];

        let result = aggregate_events(&events, 30);

        assert_eq!(result.len(), 2);
        let main_rs = result.iter().find(|e| e.title == "main.rs").unwrap();
        assert_eq!(main_rs.total_duration_seconds, 8.0);
        assert_eq!(main_rs.last_active, "2026-04-04T03:51:00+00:00");
    }

    #[test]
    fn test_aggregate_sorted_by_last_active_descending() {
        let events = vec![
            ("Firefox".to_string(), "GitHub".to_string(), 10.0, "2026-04-04T03:40:00+00:00".to_string()),
            ("Code".to_string(), "main.rs".to_string(), 3.0, "2026-04-04T03:55:00+00:00".to_string()),
            ("Terminal".to_string(), "cargo test".to_string(), 5.0, "2026-04-04T03:50:00+00:00".to_string()),
        ];

        let result = aggregate_events(&events, 30);

        assert_eq!(result[0].app, "Code");
        assert_eq!(result[1].app, "Terminal");
        assert_eq!(result[2].app, "Firefox");
    }

    #[test]
    fn test_aggregate_limits_to_max_entries() {
        let events: Vec<_> = (0..50)
            .map(|i| (
                format!("App{}", i),
                format!("Title{}", i),
                1.0,
                format!("2026-04-04T03:{:02}:00+00:00", i),
            ))
            .collect();

        let result = aggregate_events(&events, 30);

        assert_eq!(result.len(), 30);
        // Most recent should be first
        assert_eq!(result[0].app, "App49");
    }

    #[test]
    fn test_aggregate_empty_input() {
        let result = aggregate_events(&[], 30);
        assert!(result.is_empty());
    }

    #[test]
    fn test_aggregate_sums_duration_across_same_app_title() {
        let events = vec![
            ("Code".to_string(), "main.rs".to_string(), 10.0, "2026-04-04T03:50:00+00:00".to_string()),
            ("Code".to_string(), "main.rs".to_string(), 20.0, "2026-04-04T03:51:00+00:00".to_string()),
            ("Code".to_string(), "main.rs".to_string(), 30.0, "2026-04-04T03:52:00+00:00".to_string()),
        ];

        let result = aggregate_events(&events, 30);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].total_duration_seconds, 60.0);
        assert_eq!(result[0].last_active, "2026-04-04T03:52:00+00:00");
    }

    #[test]
    fn test_aggregate_different_apps_same_title_not_merged() {
        let events = vec![
            ("Code".to_string(), "README.md".to_string(), 5.0, "2026-04-04T03:50:00+00:00".to_string()),
            ("Vim".to_string(), "README.md".to_string(), 3.0, "2026-04-04T03:51:00+00:00".to_string()),
        ];

        let result = aggregate_events(&events, 30);

        assert_eq!(result.len(), 2);
    }
}
