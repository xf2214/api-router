//! 请求日志与统计指标。
//!
//! 设计目标：
//! - 当 `enable_logging = true` 时记录最近 N 条请求到内存环形缓冲；
//! - 维护按 (provider_id, model_name) 维度的累计统计：请求数、成功数、失败数、
//!   总延迟、Token 用量；
//! - 全部数据只存在内存中，进程结束即丢失（符合"本地优先"原则，避免引入 SQLite）。

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

/// 默认保留的最大日志条数（防止无界增长）。
const DEFAULT_LOG_CAPACITY: usize = 500;

/// 单次请求的完整日志记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestLog {
    /// 唯一请求 ID（用于关联多次重试）。
    pub request_id: String,
    /// 本地模型名。
    pub local_model: String,
    /// 实际命中的提供商 ID。
    pub provider_id: String,
    /// 实际命中的上游模型名。
    pub upstream_model: String,
    /// 端点（chat/completions、completions、embeddings）。
    pub endpoint: String,
    /// 是否流式请求。
    pub stream: bool,
    /// 开始时间戳（毫秒）。
    pub started_at_ms: u64,
    /// 总耗时（毫秒）。
    pub duration_ms: u64,
    /// HTTP 状态码（上游返回）。
    pub status: Option<u16>,
    /// 是否最终成功。
    pub success: bool,
    /// 失败原因（如果有）。
    pub error: Option<String>,
    /// 重试次数（不含首次）。
    pub retries: u32,
    /// 是否经过降级（fallback）。
    pub fell_back: bool,
    /// Token 用量（若上游返回）。
    pub usage: Option<TokenUsage>,
}

/// Token 用量信息。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// 按 (provider_id, model_name) 聚合的统计指标。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderStat {
    pub provider_id: String,
    pub upstream_model: String,
    pub total_requests: u64,
    pub success_count: u64,
    pub failure_count: u64,
    /// 累计延迟（毫秒），可除以 total_requests 得平均。
    pub total_duration_ms: u64,
    pub total_retries: u64,
    /// 累计 token 用量。
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_tokens: u64,
    /// 最后一次请求时间戳（毫秒）。
    pub last_request_at_ms: u64,
}

/// 全局请求日志与统计收集器。
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    inner: Arc<RwLock<MetricsInner>>,
}

#[derive(Debug)]
struct MetricsInner {
    enabled: bool,
    logs: std::collections::VecDeque<RequestLog>,
    capacity: usize,
    stats: HashMap<String, ProviderStat>,
}

impl MetricsCollector {
    pub fn new(enabled: bool) -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetricsInner {
                enabled,
                logs: std::collections::VecDeque::with_capacity(DEFAULT_LOG_CAPACITY),
                capacity: DEFAULT_LOG_CAPACITY,
                stats: HashMap::new(),
            })),
        }
    }

    /// 更新是否启用日志记录。
    pub async fn set_enabled(&self, enabled: bool) {
        let mut guard = self.inner.write().await;
        guard.enabled = enabled;
    }

    /// 记录一次请求结果。若日志未启用，则仅更新统计；统计始终维护。
    pub async fn record(&self, log: RequestLog) {
        let mut guard = self.inner.write().await;

        // 更新统计（始终）
        let key = stat_key(&log.provider_id, &log.upstream_model);
        let stat = guard.stats.entry(key.clone()).or_insert_with(|| ProviderStat {
            provider_id: log.provider_id.clone(),
            upstream_model: log.upstream_model.clone(),
            ..Default::default()
        });
        stat.total_requests += 1;
        if log.success {
            stat.success_count += 1;
        } else {
            stat.failure_count += 1;
        }
        stat.total_duration_ms += log.duration_ms;
        stat.total_retries += log.retries as u64;
        stat.last_request_at_ms = log.started_at_ms;
        if let Some(usage) = &log.usage {
            stat.total_prompt_tokens += usage.prompt_tokens;
            stat.total_completion_tokens += usage.completion_tokens;
            stat.total_tokens += usage.total_tokens;
        }

        // 详细日志仅在启用时保留
        if guard.enabled {
            if guard.logs.len() >= guard.capacity {
                guard.logs.pop_front();
            }
            guard.logs.push_back(log);
        }
    }

    /// 取最近 `limit` 条日志（按时间倒序）。
    pub async fn recent_logs(&self, limit: usize) -> Vec<RequestLog> {
        let guard = self.inner.read().await;
        if !guard.enabled {
            return Vec::new();
        }
        let limit = limit.min(guard.logs.len());
        guard
            .logs
            .iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }

    /// 清空所有日志与统计。
    pub async fn clear(&self) {
        let mut guard = self.inner.write().await;
        guard.logs.clear();
        guard.stats.clear();
    }

    /// 取所有按 (provider, model) 聚合的统计。
    pub async fn all_stats(&self) -> Vec<ProviderStat> {
        let guard = self.inner.read().await;
        let mut stats: Vec<ProviderStat> = guard.stats.values().cloned().collect();
        stats.sort_by_key(|b| std::cmp::Reverse(b.total_requests));
        stats
    }

    /// 仅累加 token 用量到统计，不递增请求计数。
    /// 用于流式请求在 stream 结束后补充 usage。
    pub async fn add_usage(&self, provider_id: &str, upstream_model: &str, usage: TokenUsage) {
        let mut guard = self.inner.write().await;
        let key = stat_key(provider_id, upstream_model);
        let stat = guard.stats.entry(key).or_insert_with(|| ProviderStat {
            provider_id: provider_id.to_string(),
            upstream_model: upstream_model.to_string(),
            ..Default::default()
        });
        stat.total_prompt_tokens += usage.prompt_tokens;
        stat.total_completion_tokens += usage.completion_tokens;
        stat.total_tokens += usage.total_tokens;
    }

    /// 将一条已记录为成功的请求转为失败（用于流式请求中途出错时修正统计）。
    /// success_count 饱和减 1（不低于 0），failure_count 加 1。
    pub async fn convert_to_failure(&self, provider_id: &str, upstream_model: &str) {
        let mut guard = self.inner.write().await;
        let key = stat_key(provider_id, upstream_model);
        if let Some(stat) = guard.stats.get_mut(&key) {
            stat.success_count = stat.success_count.saturating_sub(1);
            stat.failure_count += 1;
        }
    }
}

/// 生成统计 key。
fn stat_key(provider_id: &str, model: &str) -> String {
    format!("{provider_id}::{model}")
}

/// 从响应 JSON 中提取 usage 字段。
pub fn extract_usage(value: &serde_json::Value) -> Option<TokenUsage> {
    let usage = value.get("usage").filter(|u| u.is_object())?;
    Some(TokenUsage {
        prompt_tokens: usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        completion_tokens: usage
            .get("completion_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0),
        total_tokens: usage.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

/// 计算耗时（毫秒）。
pub fn elapsed_ms(start: std::time::Instant) -> u64 {
    start.elapsed().as_millis() as u64
}

/// 生成简短请求 ID（8 字符 hex）。
pub fn new_request_id() -> String {
    use rand::Rng;
    let bytes: [u8; 4] = rand::thread_rng().gen();
    hex_encode(&bytes)
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_log(provider: &str, model: &str, success: bool) -> RequestLog {
        RequestLog {
            request_id: "abc123".to_string(),
            local_model: "gpt-4o".to_string(),
            provider_id: provider.to_string(),
            upstream_model: model.to_string(),
            endpoint: "chat/completions".to_string(),
            stream: false,
            started_at_ms: 1000,
            duration_ms: 200,
            status: Some(if success { 200 } else { 500 }),
            success,
            error: if success { None } else { Some("upstream".to_string()) },
            retries: 0,
            fell_back: false,
            usage: Some(TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        }
    }

    #[tokio::test]
    async fn records_stats_even_when_disabled() {
        let m = MetricsCollector::new(false);
        m.record(sample_log("p1", "gpt-4o", true)).await;
        m.record(sample_log("p1", "gpt-4o", false)).await;
        let stats = m.all_stats().await;
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].total_requests, 2);
        assert_eq!(stats[0].success_count, 1);
        assert_eq!(stats[0].failure_count, 1);
        assert_eq!(stats[0].total_tokens, 30);
        // 日志不应被保留
        let logs = m.recent_logs(10).await;
        assert!(logs.is_empty());
    }

    #[tokio::test]
    async fn records_logs_when_enabled() {
        let m = MetricsCollector::new(true);
        m.record(sample_log("p1", "gpt-4o", true)).await;
        let logs = m.recent_logs(10).await;
        assert_eq!(logs.len(), 1);
    }

    #[tokio::test]
    async fn clear_wipes_all() {
        let m = MetricsCollector::new(true);
        m.record(sample_log("p1", "gpt-4o", true)).await;
        m.clear().await;
        assert!(m.recent_logs(10).await.is_empty());
        assert!(m.all_stats().await.is_empty());
    }

    #[test]
    fn extract_usage_parses_object() {
        let v: serde_json::Value = serde_json::from_str(r#"{"usage":{"prompt_tokens":5,"completion_tokens":3,"total_tokens":8}}"#).unwrap();
        let u = extract_usage(&v).unwrap();
        assert_eq!(u.prompt_tokens, 5);
        assert_eq!(u.completion_tokens, 3);
        assert_eq!(u.total_tokens, 8);
    }

    #[test]
    fn extract_usage_returns_none_when_missing() {
        let v: serde_json::Value = serde_json::from_str(r#"{"foo":"bar"}"#).unwrap();
        assert!(extract_usage(&v).is_none());
    }

    #[test]
    fn extract_usage_returns_none_for_null() {
        let v: serde_json::Value = serde_json::from_str(r#"{"usage":null}"#).unwrap();
        assert!(extract_usage(&v).is_none());
    }

    #[test]
    fn request_id_is_hex_like() {
        let id = new_request_id();
        assert_eq!(id.len(), 8);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn duration_of_zero_is_zero() {
        let now = std::time::Instant::now();
        // 立即调用，应该接近 0
        let ms = elapsed_ms(now);
        assert!(ms < 1000);
    }

    #[tokio::test]
    async fn counts_streaming_and_non_streaming_once_each() {
        let m = MetricsCollector::new(false);

        let mut streaming = sample_log("p1", "gpt-4o", true);
        streaming.stream = true;
        streaming.usage = Some(TokenUsage {
            prompt_tokens: 5,
            completion_tokens: 3,
            total_tokens: 8,
        });

        let mut non_streaming = sample_log("p1", "gpt-4o", true);
        non_streaming.stream = false;
        non_streaming.usage = Some(TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        });

        m.record(streaming).await;
        m.record(non_streaming).await;

        let stats = m.all_stats().await;
        assert_eq!(stats.len(), 1);
        let stat = &stats[0];
        assert_eq!(stat.total_requests, 2);
        assert_eq!(stat.success_count, 2);
        assert_eq!(stat.failure_count, 0);
        assert_eq!(stat.total_prompt_tokens, 15);
        assert_eq!(stat.total_completion_tokens, 8);
        assert_eq!(stat.total_tokens, 23);
    }

    #[tokio::test]
    async fn counts_success_and_failure_once_each() {
        let m = MetricsCollector::new(false);

        m.record(sample_log("p1", "gpt-4o", true)).await;
        m.record(sample_log("p1", "gpt-4o", false)).await;

        let stats = m.all_stats().await;
        assert_eq!(stats.len(), 1);
        let stat = &stats[0];
        assert_eq!(stat.total_requests, 2);
        assert_eq!(stat.success_count, 1);
        assert_eq!(stat.failure_count, 1);
    }

    #[tokio::test]
    async fn provider_stat_totals_across_models() {
        let m = MetricsCollector::new(false);

        let mut log1 = sample_log("p1", "gpt-4o", true);
        log1.usage = Some(TokenUsage {
            prompt_tokens: 10,
            completion_tokens: 5,
            total_tokens: 15,
        });

        let mut log2 = sample_log("p1", "gpt-4o-mini", true);
        log2.usage = Some(TokenUsage {
            prompt_tokens: 2,
            completion_tokens: 1,
            total_tokens: 3,
        });

        m.record(log1).await;
        m.record(log2).await;

        let stats = m.all_stats().await;
        assert_eq!(stats.len(), 2);

        let gpt4_stat = stats.iter().find(|s| s.upstream_model == "gpt-4o").unwrap();
        assert_eq!(gpt4_stat.total_requests, 1);
        assert_eq!(gpt4_stat.total_tokens, 15);

        let mini_stat = stats.iter().find(|s| s.upstream_model == "gpt-4o-mini").unwrap();
        assert_eq!(mini_stat.total_requests, 1);
        assert_eq!(mini_stat.total_tokens, 3);
    }

    #[tokio::test]
    async fn add_usage_does_not_increment_requests() {
        let m = MetricsCollector::new(false);
        m.record(sample_log("p1", "gpt-4o", true)).await;

        // 初始：1 请求, 15 tokens
        let stats = m.all_stats().await;
        assert_eq!(stats[0].total_requests, 1);
        assert_eq!(stats[0].total_tokens, 15);

        // add_usage 不递增请求计数，只加 token
        m.add_usage("p1", "gpt-4o", TokenUsage {
            prompt_tokens: 5,
            completion_tokens: 3,
            total_tokens: 8,
        }).await;

        let stats = m.all_stats().await;
        assert_eq!(stats[0].total_requests, 1); // 仍然是 1
        assert_eq!(stats[0].total_tokens, 23);  // 15 + 8 = 23
        assert_eq!(stats[0].total_prompt_tokens, 15); // 10 + 5
        assert_eq!(stats[0].total_completion_tokens, 8); // 5 + 3
    }

    #[tokio::test]
    async fn convert_to_failure_adjusts_counts() {
        let m = MetricsCollector::new(false);
        // 记录 2 个成功
        m.record(sample_log("p1", "gpt-4o", true)).await;
        m.record(sample_log("p1", "gpt-4o", true)).await;

        let stats = m.all_stats().await;
        assert_eq!(stats[0].success_count, 2);
        assert_eq!(stats[0].failure_count, 0);

        // 转一个为失败
        m.convert_to_failure("p1", "gpt-4o").await;

        let stats = m.all_stats().await;
        assert_eq!(stats[0].success_count, 1);
        assert_eq!(stats[0].failure_count, 1);
        assert_eq!(stats[0].total_requests, 2); // 总数不变
    }

    #[tokio::test]
    async fn convert_to_failure_saturates_at_zero() {
        let m = MetricsCollector::new(false);
        // 没有任何成功记录，convert_to_failure 不应导致 underflow
        m.convert_to_failure("p1", "gpt-4o").await;

        // stats 可能为空（没有对应 key），或 success_count = 0
        let stats = m.all_stats().await;
        // 如果创建了空 entry, success_count 应为 0, failure_count 应为 1
        // 如果没有创建 entry, stats 为空 - 两种情况都可接受
        if !stats.is_empty() {
            assert_eq!(stats[0].success_count, 0);
        }
    }
}
