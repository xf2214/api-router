use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::cache::ResponseCache;
use crate::circuit::CircuitBreaker;
use crate::client::ClientRegistry;
use crate::config::{AppConfig, CircuitBreakerConfig};
use crate::metrics::MetricsCollector;

/// Health status for a single provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub provider_id: String,
    pub online: bool,
    pub last_checked: Option<i64>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

impl ProviderHealth {
    /// 路由视角下：是否应跳过此提供商。
    ///
    /// 返回 true 的条件：
    /// - 离线且 last_checked 在冷却期内；
    /// - 或熔断器已打开（`is_open` 返回 true）。
    ///
    /// 若没有最近检查记录，不跳过（让请求自行测试）。
    pub async fn should_skip_for_routing(
        &self,
        circuit_breaker: &CircuitBreaker,
        provider_id: &str,
        cb_config: &CircuitBreakerConfig,
        cooldown_interval_ms: u64,
    ) -> bool {
        // 熔断器检查
        if circuit_breaker.is_open(provider_id, cb_config).await {
            return true;
        }

        // 健康检查冷却期检查
        if self.online {
            return false;
        }
        let Some(checked) = self.last_checked else {
            return false;
        };
        let now = now_secs();
        let age_secs = now.saturating_sub(checked);
        let cooldown_secs = (cooldown_interval_ms / 1000) as i64;
        age_secs < cooldown_secs
    }
}

/// 目标级冷却状态：记录 (provider, model) 的连续失败次数和冷却到期时间。
///
/// 与熔断器（CircuitBreaker）不同，此冷却机制针对非熔断类失败
/// （如 400 context too long、413 payload too large 等），
/// 在连续失败达到阈值后暂时跳过该目标。
#[derive(Debug, Clone)]
pub struct TargetCooldownState {
    /// 连续失败次数。
    pub consecutive_failures: u32,
    /// 冷却到期时间（如果处于冷却期）。
    pub cooldown_until: Option<Instant>,
    /// 最近一次失败的错误信息。
    pub last_error: Option<String>,
}

/// Shared application state.
#[derive(Debug, Clone)]
pub struct AppState {
    pub inner: Arc<AppStateInner>,
}

impl AppState {
    pub fn new(config: AppConfig, config_path: PathBuf) -> Self {
        Self {
            inner: Arc::new(AppStateInner::new(config, config_path)),
        }
    }
}

#[derive(Debug)]
pub struct AppStateInner {
    /// Current configuration.
    ///
    /// 以 `Arc<AppConfig>` 存储：请求路径只需克隆 Arc（廉价指针拷贝）即可获得
    /// 一致的配置快照，避免每个请求对整个 AppConfig（providers/models/groups）
    /// 做深拷贝；写方在保存时构造新的 Arc 整体替换。
    pub config: RwLock<Arc<AppConfig>>,
    /// Path to the configuration file.
    pub config_path: PathBuf,
    /// Round-robin counter.
    pub round_robin: AtomicUsize,
    /// Token used to shut down the local server.
    pub server_token: Mutex<Option<CancellationToken>>,
    /// Latest provider health status.
    pub health_status: RwLock<HashMap<String, ProviderHealth>>,
    /// 共享 HTTP 客户端注册表（含 QPS / 并发限流）。
    pub clients: ClientRegistry,
    /// 请求日志与统计收集器。
    pub metrics: MetricsCollector,
    /// 熔断器状态。
    pub circuit_breaker: CircuitBreaker,
    /// 响应缓存。
    pub cache: ResponseCache,
    /// 目标级冷却状态：按 (provider_id, model_name) 索引。
    pub target_cooldowns: RwLock<HashMap<(String, String), TargetCooldownState>>,
}

impl AppStateInner {
    pub fn new(config: AppConfig, config_path: PathBuf) -> Self {
        let enable_logging = config.enable_logging;
        let cache_config = config.cache.clone();
        Self {
            config: RwLock::new(Arc::new(config)),
            config_path,
            round_robin: AtomicUsize::new(0),
            server_token: Mutex::new(None),
            health_status: RwLock::new(HashMap::new()),
            clients: ClientRegistry::new(),
            metrics: MetricsCollector::new(enable_logging),
            circuit_breaker: CircuitBreaker::new(),
            cache: ResponseCache::new(cache_config),
            target_cooldowns: RwLock::new(HashMap::new()),
        }
    }

    /// Store health status for a provider.
    pub async fn set_provider_health(&self, health: ProviderHealth) {
        let mut guard = self.health_status.write().await;
        guard.insert(health.provider_id.clone(), health);
    }

    /// Get a snapshot of all provider health statuses.
    pub async fn all_health_statuses(&self) -> Vec<ProviderHealth> {
        let guard = self.health_status.read().await;
        guard.values().cloned().collect()
    }

    /// Atomically increment and return the round-robin index.
    pub fn next_round_robin(&self) -> usize {
        self.round_robin.fetch_add(1, Ordering::Relaxed)
    }

    /// Store a new cancellation token for the running server.
    pub async fn set_server_token(&self, token: CancellationToken) {
        let mut guard = self.server_token.lock().await;
        *guard = Some(token);
    }

    /// Cancel the running server if any.
    pub async fn stop_server(&self) {
        let mut guard = self.server_token.lock().await;
        if let Some(token) = guard.take() {
            token.cancel();
        }
    }

    /// Returns true if the server is believed to be running.
    pub async fn is_server_running(&self) -> bool {
        let guard = self.server_token.lock().await;
        guard.as_ref().map(|t| !t.is_cancelled()).unwrap_or(false)
    }

    /// 同步 metrics 的启用状态到当前配置。
    pub async fn sync_metrics_enabled(&self) {
        let config = self.config.read().await;
        self.metrics.set_enabled(config.enable_logging).await;
    }

    /// 同步缓存配置到当前配置。
    pub async fn sync_cache_config(&self) {
        let config = self.config.read().await;
        self.cache.set_config(config.cache.clone()).await;
    }

    /// 默认目标级冷却阈值：连续失败次数。
    const TARGET_COOLDOWN_THRESHOLD: u32 = 3;

    /// 默认目标级冷却时长（秒）。
    const TARGET_COOLDOWN_DURATION_SECS: u64 = 30;

    /// 记录一次目标级失败。
    ///
    /// 递增 (provider_id, model_name) 的连续失败计数。
    /// 当连续失败达到阈值时，进入冷却期（默认 30 秒）。
    pub async fn record_target_failure(&self, provider_id: &str, model_name: &str, error: &str) {
        let mut guard = self.target_cooldowns.write().await;
        let key = (provider_id.to_string(), model_name.to_string());
        let entry = guard.entry(key).or_insert(TargetCooldownState {
            consecutive_failures: 0,
            cooldown_until: None,
            last_error: None,
        });
        entry.consecutive_failures += 1;
        entry.last_error = Some(error.to_string());

        if entry.consecutive_failures >= Self::TARGET_COOLDOWN_THRESHOLD {
            entry.cooldown_until = Some(
                Instant::now()
                    + std::time::Duration::from_secs(Self::TARGET_COOLDOWN_DURATION_SECS),
            );
        }
    }

    /// 检查指定目标是否处于冷却期。
    ///
    /// 返回 `true` 表示应跳过该目标。
    /// 对于未知目标（无可冷却记录）返回 `false`。
    pub async fn is_target_on_cooldown(&self, provider_id: &str, model_name: &str) -> bool {
        let guard = self.target_cooldowns.read().await;
        let key = (provider_id.to_string(), model_name.to_string());
        match guard.get(&key) {
            Some(entry) => match entry.cooldown_until {
                Some(until) => Instant::now() < until,
                None => false,
            },
            None => false,
        }
    }

    /// 记录一次目标级成功。
    ///
    /// 重置连续失败计数并清除冷却状态。
    pub async fn record_target_success(&self, provider_id: &str, model_name: &str) {
        let mut guard = self.target_cooldowns.write().await;
        let key = (provider_id.to_string(), model_name.to_string());
        if let Some(entry) = guard.get_mut(&key) {
            entry.consecutive_failures = 0;
            entry.cooldown_until = None;
            entry.last_error = None;
        }
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::CircuitBreaker;
    use crate::config::CircuitBreakerConfig;

    fn test_cb_config() -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            failure_threshold: 0, // 禁用熔断器，只测试健康检查
            failure_threshold_percentage: None,
            cooldown_interval_ms: 60_000,
            failure_status_codes: None,
            minimum_requests: None,
        }
    }

    #[tokio::test]
    async fn should_skip_offline_within_cooldown() {
        let cb = CircuitBreaker::new();
        let cfg = test_cb_config();
        let now = now_secs();
        let h = ProviderHealth {
            provider_id: "p".to_string(),
            online: false,
            last_checked: Some(now),
            latency_ms: None,
            error: Some("err".to_string()),
        };
        assert!(h.should_skip_for_routing(&cb, "p", &cfg, 60_000).await);
    }

    #[tokio::test]
    async fn should_not_skip_offline_after_cooldown() {
        let cb = CircuitBreaker::new();
        let cfg = test_cb_config();
        let now = now_secs();
        let h = ProviderHealth {
            provider_id: "p".to_string(),
            online: false,
            last_checked: Some(now - 120), // 2 分钟前，超过 60s 冷却
            latency_ms: None,
            error: Some("err".to_string()),
        };
        assert!(!h.should_skip_for_routing(&cb, "p", &cfg, 60_000).await);
    }

    #[tokio::test]
    async fn should_not_skip_online() {
        let cb = CircuitBreaker::new();
        let cfg = test_cb_config();
        let h = ProviderHealth {
            provider_id: "p".to_string(),
            online: true,
            last_checked: Some(now_secs()),
            latency_ms: Some(50),
            error: None,
        };
        assert!(!h.should_skip_for_routing(&cb, "p", &cfg, 60_000).await);
    }

    #[tokio::test]
    async fn should_not_skip_when_never_checked() {
        let cb = CircuitBreaker::new();
        let cfg = test_cb_config();
        let h = ProviderHealth {
            provider_id: "p".to_string(),
            online: false,
            last_checked: None,
            latency_ms: None,
            error: None,
        };
        assert!(!h.should_skip_for_routing(&cb, "p", &cfg, 60_000).await);
    }

    #[tokio::test]
    async fn should_skip_when_circuit_breaker_open() {
        let cb = CircuitBreaker::new();
        let mut cfg = test_cb_config();
        cfg.failure_threshold = 2; // 启用熔断器
        let now = now_secs();
        let h = ProviderHealth {
            provider_id: "p".to_string(),
            online: true,
            last_checked: Some(now),
            latency_ms: Some(50),
            error: None,
        };
        // 触发熔断
        cb.record_failure("p", Some(500), &cfg).await;
        cb.record_failure("p", Some(500), &cfg).await;
        // 即使在线，熔断器打开也应跳过
        assert!(h.should_skip_for_routing(&cb, "p", &cfg, 60_000).await);
    }

    // ── 目标级冷却测试 ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn target_cooldown_unknown_target_returns_false() {
        let state = AppStateInner::new(
            crate::config::AppConfig::default(),
            std::path::PathBuf::new(),
        );
        // 从未记录过的目标应返回 false
        assert!(!state.is_target_on_cooldown("unknown", "unknown").await);
    }

    #[tokio::test]
    async fn target_cooldown_triggers_after_three_failures() {
        let state = AppStateInner::new(
            crate::config::AppConfig::default(),
            std::path::PathBuf::new(),
        );
        // 2 次失败，不应冷却
        state.record_target_failure("p1", "m1", "error 1").await;
        assert!(!state.is_target_on_cooldown("p1", "m1").await);
        state.record_target_failure("p1", "m1", "error 2").await;
        assert!(!state.is_target_on_cooldown("p1", "m1").await);
        // 第 3 次失败，触发冷却
        state.record_target_failure("p1", "m1", "error 3").await;
        assert!(state.is_target_on_cooldown("p1", "m1").await);
    }

    #[tokio::test]
    async fn target_cooldown_success_resets_state() {
        let state = AppStateInner::new(
            crate::config::AppConfig::default(),
            std::path::PathBuf::new(),
        );
        // 3 次失败触发冷却
        for i in 0..3 {
            state
                .record_target_failure("p1", "m1", &format!("error {i}"))
                .await;
        }
        assert!(state.is_target_on_cooldown("p1", "m1").await);
        // 成功后重置
        state.record_target_success("p1", "m1").await;
        assert!(!state.is_target_on_cooldown("p1", "m1").await);
    }

    #[tokio::test]
    async fn target_cooldown_expires_after_duration() {
        let state = AppStateInner::new(
            crate::config::AppConfig::default(),
            std::path::PathBuf::new(),
        );
        // 直接设置一个已过期的冷却状态
        {
            let mut guard = state.target_cooldowns.write().await;
            guard.insert(
                ("p1".to_string(), "m1".to_string()),
                TargetCooldownState {
                    consecutive_failures: 3,
                    cooldown_until: Some(Instant::now() - std::time::Duration::from_secs(1)),
                    last_error: Some("old error".to_string()),
                },
            );
        }
        // 冷却已过期，应返回 false
        assert!(!state.is_target_on_cooldown("p1", "m1").await);
    }
}
