pub mod breaker;
pub mod types;
pub mod window;

pub use crate::infra::circuit::breaker::{is_failure_status, CircuitBreaker};
pub use crate::infra::circuit::types::BreakerState;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::config::CircuitBreakerConfig;

    fn config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 3,
            failure_threshold_percentage: None,
            cooldown_interval_ms: 100,
            failure_status_codes: None,
            minimum_requests: None,
        }
    }

    #[tokio::test]
    async fn opens_after_threshold_failures() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for _ in 0..2 {
            cb.record_failure("p1", Some(500), &cfg).await;
            assert!(!cb.is_open("p1", &cfg).await);
        }
        cb.record_failure("p1", Some(500), &cfg).await;
        assert!(cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn ignores_unlisted_status_codes() {
        let cb = CircuitBreaker::new();
        let mut cfg = config();
        cfg.failure_status_codes = Some(vec![429, 503]);
        for _ in 0..5 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        assert!(!cb.is_open("p1", &cfg).await);

        for _ in 0..3 {
            cb.record_failure("p1", Some(429), &cfg).await;
        }
        assert!(cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn closes_after_cooldown() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        assert!(cb.is_open("p1", &cfg).await);
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!cb.is_open("p1", &cfg).await);
        assert!(cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn success_resets_open_state() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        assert!(cb.is_open("p1", &cfg).await);
        cb.record_success("p1").await;
        assert!(!cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn percentage_based_open() {
        let cb = CircuitBreaker::new();
        let cfg = CircuitBreakerConfig {
            failure_threshold: 0,
            failure_threshold_percentage: Some(50),
            cooldown_interval_ms: 1000,
            failure_status_codes: None,
            minimum_requests: Some(4),
        };
        cb.record_failure("p1", Some(500), &cfg).await;
        cb.record_success("p1").await;
        cb.record_failure("p1", Some(500), &cfg).await;
        assert!(!cb.is_open("p1", &cfg).await);
        cb.record_failure("p1", Some(500), &cfg).await;
        assert!(cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn does_not_open_on_client_errors() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for status in [400u16, 401, 403, 404, 413] {
            for _ in 0..10 {
                cb.record_failure("p1", Some(status), &cfg).await;
            }
        }
        assert!(!cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn opens_on_server_errors_and_rate_limit() {
        let cb = CircuitBreaker::new();
        let cfg = config();

        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        assert!(cb.is_open("p1", &cfg).await);

        for _ in 0..3 {
            cb.record_failure("p2", Some(429), &cfg).await;
        }
        assert!(cb.is_open("p2", &cfg).await);
    }

    #[tokio::test]
    async fn sliding_window_failure_rate() {
        let cb = CircuitBreaker::new();
        let cfg = CircuitBreakerConfig {
            failure_threshold: 0,
            failure_threshold_percentage: Some(50),
            cooldown_interval_ms: 100,
            failure_status_codes: None,
            minimum_requests: Some(4),
        };
        cb.record_failure("p1", Some(500), &cfg).await;
        cb.record_success("p1").await;
        cb.record_failure("p1", Some(500), &cfg).await;
        cb.record_success("p1").await;
        cb.record_failure("p1", Some(500), &cfg).await;
        assert!(cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn sliding_window_old_failures_expire() {
        let cb = CircuitBreaker::new();
        let cfg = CircuitBreakerConfig {
            failure_threshold: 3,
            failure_threshold_percentage: None,
            cooldown_interval_ms: 100,
            failure_status_codes: None,
            minimum_requests: None,
        };
        cb.record_failure("p1", Some(500), &cfg).await;
        cb.record_failure("p1", Some(500), &cfg).await;
        cb.record_failure("p1", Some(500), &cfg).await;
        assert!(cb.is_open("p1", &cfg).await);

        tokio::time::sleep(Duration::from_millis(10)).await;

        {
            let mut guard = cb.inner.write().await;
            let entry = guard.get_mut("p1").unwrap();
            entry.window_ms = 1;
            entry.prune_window();
            assert!(entry.failure_window.is_empty());
        }
        cb.reset("p1").await;
        assert!(!cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn half_open_allows_only_one_probe() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        assert!(cb.is_open("p1", &cfg).await);

        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(!cb.is_open("p1", &cfg).await);
        assert!(cb.is_open("p1", &cfg).await);

        cb.release_half_open_permit("p1").await;
        assert!(!cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn half_open_probe_success_closes() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(!cb.is_open("p1", &cfg).await);
        cb.record_success("p1").await;
        assert!(!cb.is_open("p1", &cfg).await);
        assert!(cb.is_available("p1").await);
    }

    #[tokio::test]
    async fn half_open_probe_failure_reopens() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        tokio::time::sleep(Duration::from_millis(150)).await;

        assert!(!cb.is_open("p1", &cfg).await);
        cb.record_failure("p1", Some(500), &cfg).await;
        assert!(cb.is_open("p1", &cfg).await);
    }

    #[tokio::test]
    async fn is_available_returns_true_for_closed() {
        let cb = CircuitBreaker::new();
        assert!(cb.is_available("unknown").await);
        cb.record_success("p1").await;
        assert!(cb.is_available("p1").await);
    }

    #[tokio::test]
    async fn is_available_returns_false_for_open() {
        let cb = CircuitBreaker::new();
        let cfg = config();
        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        assert!(!cb.is_available("p1").await);
    }

    #[tokio::test]
    async fn hot_config_update() {
        let cb = CircuitBreaker::new();
        let mut cfg = config();
        cfg.cooldown_interval_ms = 100;

        for _ in 0..3 {
            cb.record_failure("p1", Some(500), &cfg).await;
        }
        assert!(cb.is_open("p1", &cfg).await);

        let new_cfg = CircuitBreakerConfig {
            failure_threshold: 10,
            cooldown_interval_ms: 100,
            ..Default::default()
        };
        cb.update_config("p1", &new_cfg).await;

        cb.reset("p1").await;
        assert!(!cb.is_open("p1", &cfg).await);

        for _ in 0..5 {
            cb.record_failure("p1", Some(500), &new_cfg).await;
        }
        assert!(!cb.is_open("p1", &new_cfg).await);
    }
}
