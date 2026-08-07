use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use tokio::sync::RwLock;

use crate::config::CircuitBreakerConfig;

use super::types::{BreakerEntry, BreakerState};

#[derive(Debug, Default, Clone)]
pub struct CircuitBreaker {
    pub(crate) inner: Arc<RwLock<HashMap<String, BreakerEntry>>>,
}

impl CircuitBreaker {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn is_available(&self, provider_id: &str) -> bool {
        let guard = self.inner.read().await;
        match guard.get(provider_id) {
            Some(entry) => entry.state == BreakerState::Closed,
            None => true,
        }
    }

    pub async fn is_open(&self, provider_id: &str, config: &CircuitBreakerConfig) -> bool {
        if config.failure_threshold == 0 && config.failure_threshold_percentage.is_none() {
            return false;
        }

        {
            let guard = self.inner.read().await;
            let Some(entry) = guard.get(provider_id) else {
                return false;
            };

            match entry.state {
                BreakerState::Closed => return false,
                BreakerState::HalfOpen => {
                    let acquired = entry.half_open_permits.fetch_update(
                        Ordering::AcqRel,
                        Ordering::Acquire,
                        |v| if v > 0 { Some(v - 1) } else { None },
                    );
                    return acquired.is_err();
                }
                BreakerState::Open => {
                    if let Some(opened_at) = entry.opened_at {
                        let elapsed = opened_at.elapsed().as_millis() as u64;
                        if elapsed < config.cooldown_interval_ms {
                            return true;
                        }
                    } else {
                        return false;
                    }
                }
            }
        }

        let mut guard = self.inner.write().await;
        let Some(entry) = guard.get_mut(provider_id) else {
            return false;
        };

        if entry.state == BreakerState::Open {
            entry.state = BreakerState::HalfOpen;
            entry.half_open_permits.store(1, Ordering::Release);
            entry.reset_counts();
            entry.half_open_permits.fetch_sub(1, Ordering::AcqRel);
            return false;
        }

        match entry.state {
            BreakerState::HalfOpen => {
                let acquired = entry.half_open_permits.fetch_update(
                    Ordering::AcqRel,
                    Ordering::Acquire,
                    |v| if v > 0 { Some(v - 1) } else { None },
                );
                acquired.is_err()
            }
            _ => false,
        }
    }

    pub async fn release_half_open_permit(&self, provider_id: &str) {
        let guard = self.inner.read().await;
        if let Some(entry) = guard.get(provider_id) {
            if entry.state == BreakerState::HalfOpen {
                entry.half_open_permits.fetch_add(1, Ordering::Release);
            }
        }
    }

    pub async fn record_success(&self, provider_id: &str) {
        let mut guard = self.inner.write().await;
        let entry = guard.entry(provider_id.to_string()).or_default();

        match entry.state {
            BreakerState::HalfOpen => {
                entry.state = BreakerState::Closed;
                entry.reset_counts();
                entry.opened_at = None;
                entry.half_open_permits.store(0, Ordering::Release);
            }
            BreakerState::Open => {
                entry.state = BreakerState::Closed;
                entry.reset_counts();
                entry.opened_at = None;
                entry.half_open_permits.store(0, Ordering::Release);
            }
            BreakerState::Closed => {
                entry.successes += 1;
            }
        }
    }

    pub async fn record_failure(
        &self,
        provider_id: &str,
        status: Option<u16>,
        config: &CircuitBreakerConfig,
    ) {
        if let Some(codes) = &config.failure_status_codes {
            if let Some(s) = status {
                if !codes.contains(&s) {
                    return;
                }
            } else {
                return;
            }
        }

        if config.failure_status_codes.is_none() {
            if let Some(s) = status {
                if !is_failure_status(s, config) {
                    return;
                }
            }
        }

        let mut guard = self.inner.write().await;
        let entry = guard.entry(provider_id.to_string()).or_default();

        match entry.state {
            BreakerState::HalfOpen => {
                entry.state = BreakerState::Open;
                entry.opened_at = Some(Instant::now());
                entry.half_open_permits.store(0, Ordering::Release);
                entry.failure_window.push_back(Instant::now());
                entry.prune_window();
                entry.failures += 1;
                return;
            }
            BreakerState::Open => {
                return;
            }
            BreakerState::Closed => {}
        }

        entry.failures += 1;
        entry.failure_window.push_back(Instant::now());
        entry.prune_window();
        if entry.first_failure.is_none() {
            entry.first_failure = Some(Instant::now());
        }

        let should_open = Self::evaluate_open(entry, config);
        if should_open {
            entry.state = BreakerState::Open;
            entry.opened_at = Some(Instant::now());
        }
    }

    pub async fn update_config(&self, provider_id: &str, config: &CircuitBreakerConfig) {
        let mut guard = self.inner.write().await;
        if let Some(entry) = guard.get_mut(provider_id) {
            entry.window_ms = config.cooldown_interval_ms.max(1000);
            entry.prune_window();
        }
    }

    pub async fn reset(&self, provider_id: &str) {
        let mut guard = self.inner.write().await;
        if let Some(entry) = guard.get_mut(provider_id) {
            entry.state = BreakerState::Closed;
            entry.reset_counts();
            entry.opened_at = None;
            entry.half_open_permits.store(0, Ordering::Release);
        }
    }

    pub async fn all_states(&self) -> Vec<(String, BreakerState, u32, u32)> {
        let guard = self.inner.read().await;
        guard
            .iter()
            .map(|(id, e)| (id.clone(), e.state, e.successes, e.failures))
            .collect()
    }
}

pub fn is_failure_status(status: u16, config: &CircuitBreakerConfig) -> bool {
    match &config.failure_status_codes {
        Some(codes) => codes.contains(&status),
        None => status >= 500 || status == 429,
    }
}
