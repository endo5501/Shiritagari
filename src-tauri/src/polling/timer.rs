use tokio::time::Duration;

/// Controls polling loop timing.
/// Ensures sleep always happens (even after `continue`),
/// except on the very first cycle for fast startup.
pub struct PollingTimer {
    interval: Duration,
    first_run: bool,
}

impl PollingTimer {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            first_run: true,
        }
    }

    /// Returns the duration to sleep before the next cycle.
    /// Returns `None` on the first call (no delay for startup).
    /// Returns `Some(interval)` on all subsequent calls.
    pub fn next_delay(&mut self) -> Option<Duration> {
        if self.first_run {
            self.first_run = false;
            None
        } else {
            Some(self.interval)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_call_returns_none() {
        let mut timer = PollingTimer::new(Duration::from_secs(600));
        assert_eq!(timer.next_delay(), None);
    }

    #[test]
    fn test_subsequent_calls_return_interval() {
        let mut timer = PollingTimer::new(Duration::from_secs(600));
        timer.next_delay(); // first: None

        assert_eq!(timer.next_delay(), Some(Duration::from_secs(600)));
        assert_eq!(timer.next_delay(), Some(Duration::from_secs(600)));
        assert_eq!(timer.next_delay(), Some(Duration::from_secs(600)));
    }

    #[test]
    fn test_sleep_not_skipped_after_many_cycles() {
        let mut timer = PollingTimer::new(Duration::from_secs(600));
        timer.next_delay(); // first

        // Simulate many cycles (as if continue was hit repeatedly)
        for _ in 0..100 {
            let delay = timer.next_delay();
            assert_eq!(delay, Some(Duration::from_secs(600)),
                "Sleep must never be skipped after the first cycle");
        }
    }
}
