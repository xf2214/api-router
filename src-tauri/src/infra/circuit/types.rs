use std::collections::VecDeque;
use std::sync::atomic::AtomicU32;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BreakerState {
    #[default]
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
pub(crate) struct BreakerEntry {
    pub(crate) state: BreakerState,
    pub(crate) successes: u32,
    pub(crate) failures: u32,
    pub(crate) first_failure: Option<Instant>,
    pub(crate) opened_at: Option<Instant>,
    pub(crate) half_open_permits: AtomicU32,
    pub(crate) failure_window: VecDeque<Instant>,
    pub(crate) window_ms: u64,
}

impl Default for BreakerEntry {
    fn default() -> Self {
        Self {
            state: BreakerState::Closed,
            successes: 0,
            failures: 0,
            first_failure: None,
            opened_at: None,
            half_open_permits: AtomicU32::new(0),
            failure_window: VecDeque::new(),
            window_ms: 60_000,
        }
    }
}

impl BreakerEntry {
    pub(crate) fn reset_counts(&mut self) {
        self.successes = 0;
        self.failures = 0;
        self.first_failure = None;
        self.failure_window.clear();
    }

    pub(crate) fn prune_window(&mut self) {
        let cutoff = Instant::now()
            .checked_sub(std::time::Duration::from_millis(self.window_ms))
            .unwrap_or_else(|| Instant::now());
        while let Some(&t) = self.failure_window.front() {
            if t < cutoff {
                self.failure_window.pop_front();
            } else {
                break;
            }
        }
    }
}
