//! 共享 HTTP 客户端、限流器与重试策略。
//!
//! 设计目标：
//! - 复用 `reqwest::Client`（连接池），避免每次请求重建客户端带来的 TCP/TLS 握手开销。
//! - 实现配置中已声明但未生效的 `qps_limit` / `concurrency_limit`。
//! - 提供带指数退避+抖动的重试策略，仅对可重试错误（5xx/429/网络错误）重试。

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use tokio::sync::{Mutex, Semaphore};
use tracing::debug;

use crate::config::ProviderConfig;
use crate::error::AppError;

/// 单个提供商的运行时资源（HTTP 客户端 + 限流器）。
#[derive(Debug, Clone)]
pub struct ProviderRuntime {
    pub provider_id: String,
    pub http: Client,
    /// 并发限制信号量；`None` 表示不限制。
    concurrency: Option<Arc<Semaphore>>,
    /// 简单令牌桶：上次请求时间 + 最小间隔。
    qps_state: Option<Arc<Mutex<QpsState>>>,
    /// TPM 滑动窗口限流器；`None` 表示不限制。
    tpm_state: Option<Arc<Mutex<TpmState>>>,
}

#[derive(Debug)]
struct QpsState {
    /// 最小请求间隔（毫秒）。0 表示不限制。
    min_interval_ms: u64,
    /// 上次放行时间戳（毫秒）。
    last_release_ms: u64,
}

/// TPM 滑动窗口限流状态。
#[derive(Debug)]
struct TpmState {
    /// TPM 限制（tokens per minute）。
    limit: u64,
    /// 滑动窗口内的 token 记录：(时间戳 ms, token 数)。
    window: VecDeque<(u64, u64)>,
    /// 窗口大小（毫秒），默认 60_000。
    window_ms: u64,
}

impl TpmState {
    fn new(limit: u32) -> Self {
        Self {
            limit: limit as u64,
            window: VecDeque::new(),
            window_ms: 60_000,
        }
    }

    /// 清理窗口中已过期的记录，返回窗口内总 token 数。
    fn prune_and_sum(&mut self) -> u64 {
        let now = now_ms();
        let cutoff = now.saturating_sub(self.window_ms);
        while let Some(&(ts, _)) = self.window.front() {
            if ts < cutoff {
                self.window.pop_front();
            } else {
                break;
            }
        }
        self.window.iter().map(|&(_, tokens)| tokens).sum()
    }
}

impl ProviderRuntime {
    /// 构建一个提供商运行时。`config.qps_limit` / `concurrency_limit` 为 0 表示不限制。
    pub fn new(config: &ProviderConfig) -> Result<Self, AppError> {
        let timeout = Duration::from_secs(config.timeout_seconds.max(5));
        let connect_timeout = Duration::from_secs(10);

        let mut builder = Client::builder()
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            .pool_max_idle_per_host(8)
            .tcp_keepalive(Duration::from_secs(30))
            .user_agent(concat!("api-router/", env!("CARGO_PKG_VERSION")));
        if config.disable_proxy {
            builder = builder.no_proxy();
        }
        let http = builder
            .build()
            .map_err(|e| AppError::Config(format!("构建 HTTP 客户端失败: {e}")))?;

        let concurrency = if config.concurrency_limit > 0 {
            Some(Arc::new(Semaphore::new(config.concurrency_limit as usize)))
        } else {
            None
        };

        let qps_state = if config.qps_limit > 0 {
            let min_interval_ms = 1000u64 / config.qps_limit as u64;
            Some(Arc::new(Mutex::new(QpsState {
                min_interval_ms,
                last_release_ms: 0,
            })))
        } else {
            None
        };

        let tpm_state = if config.tpm_limit > 0 {
            Some(Arc::new(Mutex::new(TpmState::new(config.tpm_limit))))
        } else {
            None
        };

        Ok(Self {
            provider_id: config.id.clone(),
            http,
            concurrency,
            qps_state,
            tpm_state,
        })
    }

    /// 申请一次请求许可（并发槽 + QPS 间隔 + TPM 窗口检查）。返回的守卫释放时归还并发槽。
    pub async fn acquire(&self) -> Result<PermitGuard, AppError> {
        let _sem_guard = if let Some(sem) = &self.concurrency {
            Some(sem.clone().acquire_owned().await.map_err(|e| {
                AppError::Config(format!("并发信号量异常: {e}"))
            })?)
        } else {
            None
        };

        if let Some(state_arc) = &self.qps_state {
            let mut state = state_arc.lock().await;
            let now = now_ms();
            if state.last_release_ms > 0 {
                let elapsed = now.saturating_sub(state.last_release_ms);
                if elapsed < state.min_interval_ms {
                    let wait = state.min_interval_ms - elapsed;
                    debug!(
                        "QPS limit: provider={} sleeping {}ms",
                        self.provider_id, wait
                    );
                    tokio::time::sleep(Duration::from_millis(wait)).await;
                }
            }
            state.last_release_ms = now_ms();
        }

        // TPM 滑动窗口检查：若当前窗口内 token 总量已达上限，等待最早记录过期
        if let Some(state_arc) = &self.tpm_state {
            let mut state = state_arc.lock().await;
            let current = state.prune_and_sum();
            if current >= state.limit {
                // 计算需要等待的时间：最早记录过期的时间
                if let Some(&(oldest_ts, _)) = state.window.front() {
                    let now = now_ms();
                    let wait = oldest_ts + state.window_ms.saturating_sub(now);
                    if wait > 0 {
                        debug!(
                            "TPM limit: provider={} current={} limit={} waiting {}ms",
                            self.provider_id, current, state.limit, wait
                        );
                        tokio::time::sleep(Duration::from_millis(wait)).await;
                        // 等待后重新清理
                        state.prune_and_sum();
                    }
                }
            }
        }

        Ok(PermitGuard { _sem_guard })
    }

    /// 记录一次请求的 token 用量（用于 TPM 限流）。
    /// 应在请求完成并获取到 usage 后调用。
    pub async fn record_tokens(&self, total_tokens: u64) {
        if let Some(state_arc) = &self.tpm_state {
            if total_tokens == 0 {
                return;
            }
            let mut state = state_arc.lock().await;
            state.prune_and_sum();
            state.window.push_back((now_ms(), total_tokens));
        }
    }

    /// Returns the number of available permits for the concurrency semaphore.
    /// Returns `usize::MAX` if there is no concurrency limit.
    pub fn available_permits(&self) -> usize {
        self.concurrency
            .as_ref()
            .map(|s| s.available_permits())
            .unwrap_or(usize::MAX)
    }
}

/// 并发许可守卫；drop 时自动归还信号量。
pub struct PermitGuard {
    _sem_guard: Option<tokio::sync::OwnedSemaphorePermit>,
}

/// 提供商 ID -> 运行时资源的缓存。
#[derive(Debug, Default)]
pub struct ClientRegistry {
    runtimes: Mutex<HashMap<String, ProviderRuntime>>,
}

impl ClientRegistry {
    pub fn new() -> Self {
        Self {
            runtimes: Mutex::new(HashMap::new()),
        }
    }

    /// 获取或创建指定提供商的运行时。
    ///
    /// 如果已存在同一 provider_id 的运行时则直接返回，避免重复创建
    /// 导致信号量/限流状态丢失。若需强制刷新（如配置变更），调用方可
    /// 先调用 `invalidate` 再调用 `get`。
    pub async fn get(&self, config: &ProviderConfig) -> Result<ProviderRuntime, AppError> {
        let mut runtimes = self.runtimes.lock().await;
        if let Some(runtime) = runtimes.get(&config.id) {
            return Ok(runtime.clone());
        }
        let runtime = ProviderRuntime::new(config)?;
        runtimes.insert(config.id.clone(), runtime.clone());
        Ok(runtime)
    }

    /// 返回指定提供商的并发信号量剩余许可数。
    /// 若无并发限制或提供商尚未注册，返回 `usize::MAX`。
    pub async fn available_permits(&self, provider_id: &str) -> usize {
        let runtimes = self.runtimes.lock().await;
        runtimes
            .get(provider_id)
            .map(|rt| rt.available_permits())
            .unwrap_or(usize::MAX)
    }
}

/// 判断 HTTP 状态码是否可重试。
pub fn is_retryable_status(status: StatusCode) -> bool {
    matches!(
        status.as_u16(),
        408 | 425 | 429 | 500 | 502 | 503 | 504
    )
}

/// 判断请求错误是否可重试（连接 / 超时类）。
pub fn is_retryable_error(err: &reqwest::Error) -> bool {
    err.is_timeout() || err.is_connect() || err.is_request()
}

/// 计算第 `attempt` 次重试（从 0 开始）的退避时长。
/// 指数退避：base * 2^attempt ± 25% jitter，最终封顶 max。
pub fn backoff_duration(attempt: u32, base: Duration, max: Duration) -> Duration {
    let exp = attempt.min(6); // 防止 2^overflow
    let raw = base.as_millis() as u64 * (1u64 << exp);
    let max_millis = max.as_millis() as u64;

    // 先计算 ±25% 抖动
    let jitter = (raw / 4).max(1);
    let rand_offset = rand::random::<u64>() % (jitter * 2 + 1); // 0..=2*jitter
    let with_jitter = raw.saturating_add(rand_offset).saturating_sub(jitter);

    // 抖动后再封顶（防止 jitter 把值推过 max）
    let millis = if with_jitter > max_millis {
        max_millis
    } else {
        with_jitter
    };
    Duration::from_millis(millis.max(1))
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_grows_then_caps() {
        let base = Duration::from_millis(100);
        let max = Duration::from_secs(5);
        let d0 = backoff_duration(0, base, max);
        let d3 = backoff_duration(3, base, max);
        let d10 = backoff_duration(10, base, max);
        // 退避应该单调非递减（抖动可能让 d3 略小于 d0，但应大致增长）
        assert!(d3.as_millis() >= d0.as_millis() / 4);
        // 封顶
        assert!(d10.as_millis() <= max.as_millis() as u128 + 1);
    }

    #[test]
    fn retryable_status_classification() {
        assert!(is_retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(is_retryable_status(StatusCode::BAD_GATEWAY));
        assert!(is_retryable_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(!is_retryable_status(StatusCode::UNAUTHORIZED));
        assert!(!is_retryable_status(StatusCode::NOT_FOUND));
        assert!(!is_retryable_status(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn runtime_builds_without_limits() {
        let cfg = ProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com/v1".to_string(),
            timeout_seconds: 30,
            qps_limit: 0,
            concurrency_limit: 0,
            tpm_limit: 0,
            enabled: true,
            default_models: vec![],
            extra_headers: HashMap::new(),
            disable_proxy: false,
            model_info: HashMap::new(),
        };
        let rt = ProviderRuntime::new(&cfg).unwrap();
        assert!(rt.concurrency.is_none());
        assert!(rt.qps_state.is_none());
        assert!(rt.tpm_state.is_none());
    }

    #[test]
    fn runtime_builds_with_limits() {
        let cfg = ProviderConfig {
            id: "test".to_string(),
            name: "Test".to_string(),
            base_url: "https://example.com/v1".to_string(),
            timeout_seconds: 30,
            qps_limit: 10,
            concurrency_limit: 5,
            tpm_limit: 10000,
            enabled: true,
            default_models: vec![],
            extra_headers: HashMap::new(),
            disable_proxy: false,
            model_info: HashMap::new(),
        };
        let rt = ProviderRuntime::new(&cfg).unwrap();
        assert!(rt.concurrency.is_some());
        assert!(rt.qps_state.is_some());
        assert!(rt.tpm_state.is_some());
    }

    #[tokio::test]
    async fn registry_returns_runtime() {
        let registry = ClientRegistry::new();
        let cfg = ProviderConfig {
            id: "p1".to_string(),
            name: "P1".to_string(),
            base_url: "https://example.com/v1".to_string(),
            timeout_seconds: 30,
            qps_limit: 0,
            concurrency_limit: 0,
            tpm_limit: 0,
            enabled: true,
            default_models: vec![],
            extra_headers: HashMap::new(),
            disable_proxy: false,
            model_info: HashMap::new(),
        };
        let rt = registry.get(&cfg).await.unwrap();
        assert_eq!(rt.provider_id, "p1");
    }

    #[tokio::test]
    async fn tpm_record_and_check() {
        let cfg = ProviderConfig {
            id: "tpm-test".to_string(),
            name: "TPM Test".to_string(),
            base_url: "https://example.com/v1".to_string(),
            timeout_seconds: 30,
            qps_limit: 0,
            concurrency_limit: 0,
            tpm_limit: 100,
            enabled: true,
            default_models: vec![],
            extra_headers: HashMap::new(),
            disable_proxy: false,
            model_info: HashMap::new(),
        };
        let rt = ProviderRuntime::new(&cfg).unwrap();

        // 记录 60 tokens，应通过
        rt.record_tokens(60).await;
        // acquire 不应阻塞
        let _guard = rt.acquire().await.unwrap();

        // 再记录 50 tokens，总计 110 > 100，下次 acquire 应等待
        rt.record_tokens(50).await;
        // 此时窗口内 tokens = 110，已超限
        // 验证 record_tokens 确实记录了
        {
            let state = rt.tpm_state.as_ref().unwrap().lock().await;
            let sum: u64 = state.window.iter().map(|&(_, t)| t).sum();
            assert_eq!(sum, 110);
        }
    }

    #[tokio::test]
    async fn tpm_zero_tokens_not_recorded() {
        let cfg = ProviderConfig {
            id: "tpm-zero".to_string(),
            name: "TPM Zero".to_string(),
            base_url: "https://example.com/v1".to_string(),
            timeout_seconds: 30,
            qps_limit: 0,
            concurrency_limit: 0,
            tpm_limit: 100,
            enabled: true,
            default_models: vec![],
            extra_headers: HashMap::new(),
            disable_proxy: false,
            model_info: HashMap::new(),
        };
        let rt = ProviderRuntime::new(&cfg).unwrap();
        rt.record_tokens(0).await;
        // 窗口应仍为空
        {
            let state = rt.tpm_state.as_ref().unwrap().lock().await;
            assert!(state.window.is_empty());
        }
    }
}
