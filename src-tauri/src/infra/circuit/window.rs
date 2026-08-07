use crate::config::CircuitBreakerConfig;

use super::breaker::CircuitBreaker;
use super::types::BreakerEntry;

impl CircuitBreaker {
    pub(crate) fn evaluate_open(entry: &BreakerEntry, config: &CircuitBreakerConfig) -> bool {
        let window_failures = entry.failure_window.len() as u32;

        if config.failure_threshold > 0 && window_failures >= config.failure_threshold {
            return true;
        }

        if let Some(pct) = config.failure_threshold_percentage {
            let min_reqs = config.minimum_requests.unwrap_or(1).max(1);
            let total = entry.successes + window_failures;
            if total >= min_reqs {
                let failure_rate = (window_failures as f64) / (total as f64) * 100.0;
                if failure_rate >= pct as f64 {
                    return true;
                }
            }
        }

        false
    }
}
