//! 简单响应缓存（Simple Cache）。
//!
//! 参考 Portkey AI Gateway 的 Cache 能力：
//! - 对非流式请求按 (endpoint, body) 做精确匹配缓存
//! - 支持 TTL 与最大条目数限制
//! - 本地优先：仅内存缓存，进程结束即清空

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::response::{IntoResponse, Response};
use serde_json::Value;
use tokio::sync::RwLock;

use crate::config::CacheConfig;

/// 缓存键。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CacheKey {
    endpoint: String,
    body: String,
}

/// 缓存条目。
#[derive(Debug, Clone)]
struct CacheEntry {
    response_body: Value,
    created_at: Instant,
}

/// 内存响应缓存。
#[derive(Debug, Clone, Default)]
pub struct ResponseCache {
    inner: Arc<RwLock<CacheInner>>,
}

#[derive(Debug, Default)]
struct CacheInner {
    config: CacheConfig,
    entries: HashMap<CacheKey, CacheEntry>,
    order: VecDeque<CacheKey>,
}

impl ResponseCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner {
                config,
                entries: HashMap::new(),
                order: VecDeque::new(),
            })),
        }
    }

    /// 同步配置（通常在保存配置后调用）。
    pub async fn set_config(&self, config: CacheConfig) {
        let mut guard = self.inner.write().await;
        guard.config = config;
        if !guard.config.enabled {
            guard.entries.clear();
            guard.order.clear();
        }
    }

    /// 尝试从缓存获取响应。仅当缓存启用、非流式请求时生效。
    pub async fn get(&self, endpoint: &str, body: &Value, stream: bool) -> Option<Response> {
        if stream {
            return None;
        }
        let guard = self.inner.read().await;
        if !guard.config.enabled || guard.config.mode != crate::config::CacheMode::Simple {
            return None;
        }

        let key = build_key(endpoint, body);
        let entry = guard.entries.get(&key)?;
        let ttl = Duration::from_secs(guard.config.max_age_seconds.max(1));
        if entry.created_at.elapsed() > ttl {
            return None;
        }

        Some(
            (
                axum::http::StatusCode::OK,
                axum::Json(entry.response_body.clone()),
            )
                .into_response(),
        )
    }

    /// 将成功响应存入缓存。
    pub async fn put(&self, endpoint: &str, body: &Value, response: &Value, stream: bool) {
        if stream {
            return;
        }
        let mut guard = self.inner.write().await;
        if !guard.config.enabled || guard.config.mode != crate::config::CacheMode::Simple {
            return;
        }

        let key = build_key(endpoint, body);
        if guard.entries.contains_key(&key) {
            // 更新已有条目
            if let Some(pos) = guard.order.iter().position(|k| k == &key) {
                guard.order.remove(pos);
            }
        } else if guard.entries.len() >= guard.config.max_entries.max(1) {
            // LRU 淘汰
            if let Some(oldest) = guard.order.pop_front() {
                guard.entries.remove(&oldest);
            }
        }

        guard.order.push_back(key.clone());
        guard.entries.insert(
            key,
            CacheEntry {
                response_body: response.clone(),
                created_at: Instant::now(),
            },
        );
    }

    /// 清空缓存。
    pub async fn clear(&self) {
        let mut guard = self.inner.write().await;
        guard.entries.clear();
        guard.order.clear();
    }
}

fn build_key(endpoint: &str, body: &Value) -> CacheKey {
    CacheKey {
        endpoint: endpoint.to_string(),
        body: body.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache_cfg() -> CacheConfig {
        CacheConfig {
            enabled: true,
            max_age_seconds: 1,
            max_entries: 2,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn caches_and_returns_non_stream() {
        let cache = ResponseCache::new(cache_cfg());
        let body = serde_json::json!({"model":"gpt-4","messages":[{"role":"user","content":"hi"}]});
        let resp = serde_json::json!({"choices":[],"model":"gpt-4"});

        assert!(cache.get("chat/completions", &body, false).await.is_none());
        cache.put("chat/completions", &body, &resp, false).await;
        let cached = cache.get("chat/completions", &body, false).await;
        assert!(cached.is_some());
    }

    #[tokio::test]
    async fn does_not_cache_stream() {
        let cache = ResponseCache::new(cache_cfg());
        let body = serde_json::json!({"model":"gpt-4"});
        let resp = serde_json::json!({"choices":[]});
        cache.put("chat/completions", &body, &resp, true).await;
        assert!(cache.get("chat/completions", &body, true).await.is_none());
    }

    #[tokio::test]
    async fn respects_ttl() {
        let cache = ResponseCache::new(cache_cfg());
        let body = serde_json::json!({"model":"gpt-4"});
        let resp = serde_json::json!({"choices":[]});
        cache.put("chat/completions", &body, &resp, false).await;
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(cache.get("chat/completions", &body, false).await.is_none());
    }

    #[tokio::test]
    async fn lru_eviction() {
        let cache = ResponseCache::new(cache_cfg());
        for i in 0..3 {
            let body = serde_json::json!({"model": format!("gpt-{}", i)});
            let resp = serde_json::json!({"i": i});
            cache.put("chat/completions", &body, &resp, false).await;
        }
        let first = serde_json::json!({"model":"gpt-0"});
        assert!(cache.get("chat/completions", &first, false).await.is_none());
        let second = serde_json::json!({"model":"gpt-1"});
        assert!(cache
            .get("chat/completions", &second, false)
            .await
            .is_some());
    }
}
