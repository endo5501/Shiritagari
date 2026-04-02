use log::{info, warn};
use std::sync::Arc;
use tokio::sync::Mutex;
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

    /// Perform a single poll cycle. Returns None if AW is unavailable.
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
                is_afk: true,
                skipped_afk: true,
            });
        }

        // Get cursor for window bucket
        let cursor = {
            let db = self.db.lock().await;
            db.get_cursor(&window_bucket).ok().flatten()
        };

        // Fetch events since cursor
        let events = self
            .aw_client
            .get_events(&window_bucket, cursor.as_deref(), Some(100))
            .await
            .unwrap_or_default();

        // Deduplicate and update cursor within a transaction
        let new_events = {
            let db = self.db.lock().await;
            let mut new = Vec::new();

            for event in &events {
                let event_id = event
                    .id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| event.timestamp.clone());

                if !db.is_event_processed(&event_id, &window_bucket).unwrap_or(true) {
                    db.mark_event_processed(&event_id, &window_bucket).ok();
                    new.push(event.clone());
                }
            }

            // Update cursor to latest event timestamp
            if let Some(latest) = events.first() {
                db.update_cursor(&window_bucket, &latest.timestamp).ok();
            }

            new
        };

        Some(PollResult {
            window_events: new_events,
            is_afk: false,
            skipped_afk: false,
        })
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
