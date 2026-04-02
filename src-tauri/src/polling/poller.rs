use log::{info, warn};
use std::sync::{Arc, Mutex};
use tokio::time::{interval, Duration};

use super::aw_client::{AwClient, AwEvent};
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
    pub skipped_afk: bool,
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
            info!("User is AFK, skipping inference");
            return Some(PollResult {
                window_events: vec![],
                window_bucket,
                is_afk: true,
                skipped_afk: true,
            });
        }

        // Get cursor for window bucket
        let cursor = {
            let db = self.db.lock().unwrap();
            db.get_cursor(&window_bucket).ok().flatten()
        };

        // Fetch ALL events since cursor (paginate until exhausted)
        let mut all_events = Vec::new();
        let page_size = 100;
        let mut offset_cursor = cursor.clone();

        loop {
            let events = self
                .aw_client
                .get_events(&window_bucket, offset_cursor.as_deref(), Some(page_size))
                .await
                .unwrap_or_default();

            let count = events.len();
            if count == 0 {
                break;
            }

            // Update cursor for next page to the last event's timestamp
            if let Some(last) = events.last() {
                offset_cursor = Some(last.timestamp.clone());
            }

            all_events.extend(events);

            // If we got fewer than page_size, we've exhausted the results
            if count < page_size {
                break;
            }
        }

        // Filter out already-processed events (read-only check, no acknowledgement)
        let new_events = {
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

        Some(PollResult {
            window_events: new_events,
            window_bucket,
            is_afk: false,
            skipped_afk: false,
        })
    }

    /// Acknowledge events and advance cursor AFTER successful processing.
    /// This must be called only when inference and persistence succeeded.
    pub fn acknowledge_events(&self, events: &[AwEvent], bucket_id: &str) {
        let db = self.db.lock().unwrap();

        for event in events {
            let event_id = event
                .id
                .map(|id| id.to_string())
                .unwrap_or_else(|| event.timestamp.clone());
            db.mark_event_processed(&event_id, bucket_id).ok();
        }

        // Advance cursor to the latest event timestamp
        if let Some(latest) = events.iter().max_by_key(|e| &e.timestamp) {
            db.update_cursor(bucket_id, &latest.timestamp).ok();
        }
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
