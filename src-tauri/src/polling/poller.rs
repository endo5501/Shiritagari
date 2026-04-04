use log::{debug, info, warn};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

use super::aw_client::{AwClient, AwEvent};
use super::timestamp::advance_timestamp_1ms;
use crate::memory::Database;

pub struct Poller {
    aw_client: AwClient,
    db: Arc<Mutex<Database>>,
    interval_minutes: u64,
}

#[derive(Debug)]
pub struct PollResult {
    pub window_events: Vec<AwEvent>,
    pub window_bucket: String,
    pub is_afk: bool,
}

impl Poller {
    pub fn new(aw_client: AwClient, db: Arc<Mutex<Database>>, interval_minutes: u64) -> Self {
        Self {
            aw_client,
            db,
            interval_minutes,
        }
    }

    pub async fn check_availability(&self) -> bool {
        self.aw_client.is_available().await
    }

    /// Perform a single poll cycle: fetch events but do NOT acknowledge them.
    /// Caller must call `acknowledge_events` after successful processing.
    pub async fn poll_once(&self) -> Option<PollResult> {
        if !self.aw_client.is_available().await {
            warn!("ActivityWatch is not available, skipping poll");
            return None;
        }

        // Find buckets
        let window_bucket = match self.aw_client.find_window_bucket().await {
            Ok(Some(b)) => b,
            _ => {
                warn!("No window watcher bucket found");
                return None;
            }
        };

        let afk_bucket = self.aw_client.find_afk_bucket().await.ok().flatten();

        // Check AFK status
        let is_afk = if let Some(ref afk_id) = afk_bucket {
            self.check_afk(afk_id).await
        } else {
            false
        };

        if is_afk {
            debug!("User is AFK, continuing with event fetch");
        } else {
            debug!("User is active (not AFK)");
        }

        // Get cursor for window bucket
        let cursor = {
            let db = self.db.lock().unwrap();
            db.get_cursor(&window_bucket).ok().flatten()
        };

        // Fetch ALL events since cursor (paginate until exhausted)
        // Collect events synchronously via a buffer filled by async calls
        let page_size = 100;
        let max_pages = 50;
        let mut pages: Vec<Vec<AwEvent>> = Vec::new();

        let mut offset_cursor = cursor.clone();
        for page in 0..max_pages {
            let events = self
                .aw_client
                .get_events(&window_bucket, offset_cursor.as_deref(), Some(page_size))
                .await
                .unwrap_or_default();

            let count = events.len();
            if count == 0 {
                break;
            }

            if let Some(last) = events.last() {
                offset_cursor = Some(advance_timestamp_1ms(&last.timestamp));
            }

            pages.push(events);

            if count < page_size {
                break;
            }

            if page == max_pages - 1 {
                let total: usize = pages.iter().map(|p| p.len()).sum();
                warn!(
                    "Pagination reached maximum of {} pages ({} events). Continuing with fetched events.",
                    max_pages, total
                );
            }
        }

        let all_events: Vec<AwEvent> = pages.into_iter().flatten().collect();

        debug!(
            "Fetched {} events from ActivityWatch (cursor: {})",
            all_events.len(),
            cursor.as_deref().unwrap_or("none")
        );

        // Filter out already-processed events (read-only check, no acknowledgement)
        let new_events: Vec<_> = {
            let db = self.db.lock().unwrap();
            all_events
                .into_iter()
                .filter(|event| {
                    let event_id = event
                        .id
                        .map(|id| id.to_string())
                        .unwrap_or_else(|| event.timestamp.clone());
                    !db.is_event_processed(&event_id, &window_bucket).unwrap_or(true)
                })
                .collect()
        };

        debug!("{} new events after deduplication", new_events.len());

        Some(PollResult {
            window_events: new_events,
            window_bucket,
            is_afk,
        })
    }

    /// Acknowledge events and advance cursor AFTER successful processing.
    /// Uses a single SQLite transaction for atomicity.
    pub fn acknowledge_events(&self, events: &[AwEvent], bucket_id: &str) {
        if events.is_empty() {
            return;
        }

        let db = self.db.lock().unwrap();

        let event_pairs: Vec<(String, String)> = events
            .iter()
            .map(|e| {
                let event_id = e
                    .id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| e.timestamp.clone());
                (event_id, bucket_id.to_string())
            })
            .collect();

        let latest_timestamp = events
            .iter()
            .max_by_key(|e| &e.timestamp)
            .map(|e| e.timestamp.as_str())
            .unwrap();

        db.acknowledge_events_tx(&event_pairs, bucket_id, latest_timestamp).ok();
    }

    async fn check_afk(&self, afk_bucket_id: &str) -> bool {
        let events = self
            .aw_client
            .get_events(afk_bucket_id, None, Some(1))
            .await
            .unwrap_or_default();

        events
            .first()
            .and_then(|e| e.status())
            .map(|s| s == "afk")
            .unwrap_or(false)
    }

    pub fn interval_duration(&self) -> Duration {
        Duration::from_secs(self.interval_minutes * 60)
    }

    /// Run the polling loop. Calls the callback for each successful poll.
    pub async fn run<F>(&self, mut on_poll: F)
    where
        F: FnMut(PollResult) + Send,
    {
        let mut ticker = interval(self.interval_duration());

        loop {
            ticker.tick().await;

            if let Some(result) = self.poll_once().await {
                on_poll(result);
            } else {
                info!("ActivityWatch unavailable, will retry next interval");
            }
        }
    }
}

/// Paginate through events using the given fetch function.
/// Advances cursor by +1ms between pages to avoid re-fetching inclusive boundaries.
/// Stops after `max_pages` iterations as a safety limit.
pub(crate) fn paginate_events<F>(
    initial_cursor: Option<String>,
    page_size: usize,
    max_pages: usize,
    mut fetch: F,
) -> Vec<AwEvent>
where
    F: FnMut(Option<&str>, usize) -> Vec<AwEvent>,
{
    let mut all_events = Vec::new();
    let mut offset_cursor = initial_cursor;

    for page in 0..max_pages {
        let events = fetch(offset_cursor.as_deref(), page_size);

        let count = events.len();
        if count == 0 {
            break;
        }

        if let Some(last) = events.last() {
            offset_cursor = Some(advance_timestamp_1ms(&last.timestamp));
        }

        all_events.extend(events);

        if count < page_size {
            break;
        }

        if page == max_pages - 1 {
            warn!(
                "Pagination reached maximum of {} pages ({} events). Continuing with fetched events.",
                max_pages,
                all_events.len()
            );
        }
    }

    all_events
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_event(id: i64, timestamp: &str) -> AwEvent {
        AwEvent {
            id: Some(id),
            timestamp: timestamp.to_string(),
            duration: 1.0,
            data: HashMap::new(),
        }
    }

    #[test]
    fn test_paginate_cursor_advances_between_pages() {
        let mut cursors_received: Vec<Option<String>> = Vec::new();

        let events = paginate_events(
            Some("2026-04-04T01:00:00.000+00:00".to_string()),
            2, // page_size
            50,
            |cursor, _page_size| {
                cursors_received.push(cursor.map(|s| s.to_string()));
                match cursors_received.len() {
                    1 => vec![
                        make_event(1, "2026-04-04T01:00:00.100+00:00"),
                        make_event(2, "2026-04-04T01:00:00.200+00:00"),
                    ],
                    2 => vec![
                        make_event(3, "2026-04-04T01:00:00.300+00:00"),
                    ],
                    _ => vec![],
                }
            },
        );

        assert_eq!(events.len(), 3);
        // First call uses initial cursor
        assert_eq!(cursors_received[0], Some("2026-04-04T01:00:00.000+00:00".to_string()));
        // Second call uses last event's timestamp + 1ms
        assert_eq!(cursors_received[1], Some("2026-04-04T01:00:00.201+00:00".to_string()));
    }

    #[test]
    fn test_paginate_no_duplicate_events_between_pages() {
        let events = paginate_events(
            Some("2026-04-04T01:00:00.000+00:00".to_string()),
            2,
            50,
            |cursor, _page_size| {
                match cursor {
                    Some(c) if c == "2026-04-04T01:00:00.000+00:00" => vec![
                        make_event(1, "2026-04-04T01:00:00.100+00:00"),
                        make_event(2, "2026-04-04T01:00:00.200+00:00"),
                    ],
                    Some(c) if c == "2026-04-04T01:00:00.201+00:00" => vec![
                        // Event 2 should NOT appear here because cursor advanced past it
                        make_event(3, "2026-04-04T01:00:00.300+00:00"),
                    ],
                    _ => vec![],
                }
            },
        );

        let ids: Vec<i64> = events.iter().filter_map(|e| e.id).collect();
        assert_eq!(ids, vec![1, 2, 3]);
    }

    #[test]
    fn test_paginate_stops_at_max_pages() {
        let mut call_count = 0;

        let events = paginate_events(
            Some("2026-04-04T01:00:00.000+00:00".to_string()),
            2,
            3, // max 3 pages
            |_cursor, _page_size| {
                call_count += 1;
                // Always return full page to simulate endless data
                vec![
                    make_event(call_count * 2 - 1, &format!("2026-04-04T01:00:{:02}.000+00:00", call_count)),
                    make_event(call_count * 2, &format!("2026-04-04T01:00:{:02}.500+00:00", call_count)),
                ]
            },
        );

        assert_eq!(call_count, 3, "Should stop after max_pages iterations");
        assert_eq!(events.len(), 6, "Should have collected events from all 3 pages");
    }

    #[test]
    fn test_paginate_stops_on_empty_response() {
        let mut call_count = 0;

        let events = paginate_events(
            None,
            100,
            50,
            |_cursor, _page_size| {
                call_count += 1;
                vec![]
            },
        );

        assert_eq!(call_count, 1);
        assert_eq!(events.len(), 0);
    }

    #[test]
    fn test_paginate_stops_on_partial_page() {
        let mut call_count = 0;

        let events = paginate_events(
            None,
            100,
            50,
            |_cursor, _page_size| {
                call_count += 1;
                vec![make_event(1, "2026-04-04T01:00:00.000+00:00")]
            },
        );

        assert_eq!(call_count, 1, "Should stop after partial page");
        assert_eq!(events.len(), 1);
    }
}
