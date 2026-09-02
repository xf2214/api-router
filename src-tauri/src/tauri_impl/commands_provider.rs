use std::collections::HashMap;
use std::sync::Arc;

use reqwest::header::AUTHORIZATION;
use serde::Serialize;
use serde_json::json;
use tauri::State;
use tokio::sync::Semaphore;
use tracing::info;

use crate::{
    client::provider_timeout,
    config::{self, ModelInfo, ProviderConfig},
    keyring,
    state::{AppState, ProviderHealth},
};

/// 并发健康检查探活的最大在途请求数。
const MAX_CONCURRENT_PROBES: usize = 8;

#[derive(Debug, Clone, Serialize)]
pub struct ModelTestTargetResult {
    pub provider_id: String,
    pub model_name: String,
    pub online: bool,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelTestResult {
    pub local_name: String,
    pub targets: Vec<ModelTestTargetResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderModelsResult {
    pub models: Vec<String>,
    pub raw_json: String,
    pub info: HashMap<String, ModelInfo>,
}

#[tauri::command]
pub async fn delete_provider(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<(), String> {
    let path = state.inner.config_path.clone();

    // 克隆-修改-替换：磁盘 IO 期间不持有写锁。
    let mut config_snapshot = state.inner.config.read().await.as_ref().clone();
    config_snapshot.providers.retain(|p| p.id != provider_id);
    config_snapshot.models.iter_mut().for_each(|m| {
        m.targets.retain(|t| t.provider_id != provider_id);
    });
    config_snapshot.models.retain(|m| !m.targets.is_empty());
    config::save(&config_snapshot, &path).map_err(|e| e.to_string())?;
    *state.inner.config.write().await = Arc::new(config_snapshot);

    let _ = keyring::clear_key(&provider_id);
    Ok(())
}

#[tauri::command]
pub async fn check_provider_health(
    state: State<'_, AppState>,
    provider_id: String,
) -> Result<ProviderHealth, String> {
    let config = state.inner.config.read().await.clone();
    let provider = config
        .find_provider(&provider_id)
        .ok_or_else(|| format!("Provider {provider_id} not found"))?
        .clone();

    let health = probe_provider(&state, &provider).await;
    state.inner.set_provider_health(health.clone()).await;
    Ok(health)
}

#[tauri::command]
pub async fn check_all_providers_health(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderHealth>, String> {
    Ok(check_all_providers_health_internal(&state).await)
}

pub async fn check_all_providers_health_internal(state: &AppState) -> Vec<ProviderHealth> {
    let config = state.inner.config.read().await.clone();
    let providers: Vec<ProviderConfig> = config
        .providers
        .iter()
        .filter(|p| p.enabled)
        .cloned()
        .collect();

    // 并发探活：Semaphore 限制同时在途的探测数，避免大量提供商时打满本机出口。
    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_PROBES));
    let mut tasks = Vec::with_capacity(providers.len());
    for (idx, provider) in providers.into_iter().enumerate() {
        let st = state.clone();
        let sem = semaphore.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire_owned().await;
            let health = probe_provider(&st, &provider).await;
            st.inner.set_provider_health(health.clone()).await;
            (idx, health)
        }));
    }

    let mut indexed = Vec::with_capacity(tasks.len());
    for task in tasks {
        if let Ok(pair) = task.await {
            indexed.push(pair);
        }
    }
    indexed.sort_by_key(|(idx, _)| *idx);
    indexed.into_iter().map(|(_, h)| h).collect()
}

#[tauri::command]
pub async fn get_health_status(state: State<'_, AppState>) -> Result<Vec<ProviderHealth>, String> {
    Ok(state.inner.all_health_statuses().await)
}

fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

async fn probe_provider(state: &AppState, provider: &ProviderConfig) -> ProviderHealth {
    let api_key = match keyring::get_key_string(&provider.id) {
        Ok(k) => k,
        Err(e) => {
            return ProviderHealth {
                provider_id: provider.id.clone(),
                online: false,
                last_checked: Some(now_timestamp()),
                latency_ms: None,
                error: Some(format!("API Key 错误: {e}")),
            }
        }
    };

    let upstream_url = build_models_url(&provider.base_url);

    // 复用 ClientRegistry 的连接池（已含代理/keepalive 配置），
    // 超时通过单请求覆盖为统一的 provider_timeout 策略。
    let client = match state.inner.clients.get(provider).await {
        Ok(rt) => rt.http,
        Err(e) => {
            return ProviderHealth {
                provider_id: provider.id.clone(),
                online: false,
                last_checked: Some(now_timestamp()),
                latency_ms: None,
                error: Some(format!("构建 HTTP 客户端失败: {e}")),
            }
        }
    };

    let start = std::time::Instant::now();
    let response = client
        .get(&upstream_url)
        .timeout(provider_timeout(provider))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(
            "User-Agent",
            concat!("api-router/", env!("CARGO_PKG_VERSION")),
        )
        .header("Accept", "application/json")
        .send()
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if status.is_success() {
                ProviderHealth {
                    provider_id: provider.id.clone(),
                    online: true,
                    last_checked: Some(now_timestamp()),
                    latency_ms: Some(latency_ms),
                    error: None,
                }
            } else {
                let err = classify_http_error(status, &body);
                ProviderHealth {
                    provider_id: provider.id.clone(),
                    online: false,
                    last_checked: Some(now_timestamp()),
                    latency_ms: Some(latency_ms),
                    error: Some(err),
                }
            }
        }
        Err(e) => ProviderHealth {
            provider_id: provider.id.clone(),
            online: false,
            last_checked: Some(now_timestamp()),
            latency_ms: Some(latency_ms),
            error: Some(format!("连接失败: {e}")),
        },
    }
}

pub(crate) async fn test_target_chat_completion(
    state: &AppState,
    provider: &ProviderConfig,
    model_name: &str,
) -> ModelTestTargetResult {
    let api_key = match keyring::get_key_string(&provider.id) {
        Ok(k) => k,
        Err(e) => {
            return ModelTestTargetResult {
                provider_id: provider.id.clone(),
                model_name: model_name.to_string(),
                online: false,
                latency_ms: None,
                error: Some(format!("API Key 错误: {e}")),
            }
        }
    };

    let upstream_url = build_upstream_url(&provider.base_url, "chat/completions");

    // 复用连接池客户端；超时走统一策略。
    let client = match state.inner.clients.get(provider).await {
        Ok(rt) => rt.http,
        Err(e) => {
            return ModelTestTargetResult {
                provider_id: provider.id.clone(),
                model_name: model_name.to_string(),
                online: false,
                latency_ms: None,
                error: Some(format!("构建 HTTP 客户端失败: {e}")),
            }
        }
    };

    let body = json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "ping"}],
        "max_tokens": 10
    });

    let start = std::time::Instant::now();
    let response = client
        .post(&upstream_url)
        .timeout(provider_timeout(provider))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if status.is_success() {
                ModelTestTargetResult {
                    provider_id: provider.id.clone(),
                    model_name: model_name.to_string(),
                    online: true,
                    latency_ms: Some(latency_ms),
                    error: None,
                }
            } else {
                let err = classify_http_error(status, &text);
                ModelTestTargetResult {
                    provider_id: provider.id.clone(),
                    model_name: model_name.to_string(),
                    online: false,
                    latency_ms: Some(latency_ms),
                    error: Some(err),
                }
            }
        }
        Err(e) => ModelTestTargetResult {
            provider_id: provider.id.clone(),
            model_name: model_name.to_string(),
            online: false,
            latency_ms: Some(latency_ms),
            error: Some(format!("连接失败: {e}")),
        },
    }
}

pub(crate) fn classify_http_error(status: reqwest::StatusCode, body: &str) -> String {
    let lowered = body.to_lowercase();
    if status.as_u16() == 401 || status.as_u16() == 403 {
        "API Key 无效或权限不足".to_string()
    } else if status.as_u16() == 404
        || lowered.contains("model_not_found")
        || lowered.contains("model does not exist")
    {
        "模型不存在".to_string()
    } else if status.as_u16() == 429 {
        "请求过于频繁".to_string()
    } else {
        format!("HTTP {}: {}", status.as_u16(), truncate(body, 120))
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(1)])
    }
}

fn normalize_base_url(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

fn build_models_url(base_url: &str) -> String {
    let trimmed = base_url.trim();
    let models_suffix = "/models";
    if trimmed.len() >= models_suffix.len()
        && trimmed[trimmed.len() - models_suffix.len()..].eq_ignore_ascii_case(models_suffix)
    {
        return trimmed.to_string();
    }

    let chat_suffix = "/chat/completions";
    if trimmed.len() >= chat_suffix.len()
        && trimmed[trimmed.len() - chat_suffix.len()..].eq_ignore_ascii_case(chat_suffix)
    {
        return format!("{}models", &trimmed[..trimmed.len() - chat_suffix.len()]);
    }

    format!("{}/models", trimmed.trim_end_matches('/'))
}

fn build_upstream_url(base_url: &str, endpoint: &str) -> String {
    let trimmed = base_url.trim();
    let suffix = format!("/{}", endpoint);
    if trimmed.len() >= suffix.len()
        && trimmed[trimmed.len() - suffix.len()..].eq_ignore_ascii_case(&suffix)
    {
        return trimmed.to_string();
    }
    format!("{}/{}", trimmed.trim_end_matches('/'), endpoint)
}

#[tauri::command]
pub async fn fetch_provider_models(
    mut provider: ProviderConfig,
    api_key: String,
) -> Result<ProviderModelsResult, String> {
    let api_key = if api_key.is_empty() {
        keyring::get_key_string(&provider.id)
            .map_err(|e| format!("读取已保存的 API Key 失败：{e}，请重新填写 API Key"))?
    } else {
        api_key
    };

    provider.base_url = normalize_base_url(&provider.base_url);
    let upstream_url = build_models_url(&provider.base_url);
    // 注意：provider 可能是尚未保存的草稿配置，不能写入 ClientRegistry 缓存，
    // 因此这里保留一次性客户端，但超时走统一的 provider_timeout 策略。
    info!("Fetching models from {}", upstream_url);
    let mut builder = reqwest::Client::builder().timeout(provider_timeout(&provider));
    if provider.disable_proxy {
        builder = builder.no_proxy();
    }
    let client = builder
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(&upstream_url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(
            "User-Agent",
            concat!("api-router/", env!("CARGO_PKG_VERSION")),
        )
        .header("Accept", "application/json")
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if !status.is_success() {
                let summary = body.chars().take(200).collect::<String>();
                return Err(format!("上游返回错误 [{}]: {}", status.as_u16(), summary));
            }
            let (models, info) = parse_models_response(&body)?;
            Ok(ProviderModelsResult {
                models,
                raw_json: body,
                info,
            })
        }
        Err(e) => Err(format!("连接失败: {e}")),
    }
}

fn parse_models_response(body: &str) -> Result<(Vec<String>, HashMap<String, ModelInfo>), String> {
    let value: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("解析响应失败: {e}"))?;

    let data = value
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| {
            format!(
                "响应中未找到 data 数组: {}",
                body.chars().take(300).collect::<String>()
            )
        })?;

    let mut models = Vec::new();
    let mut info = HashMap::new();
    for item in data {
        let name = item
            .get("id")
            .and_then(|v| v.as_str())
            .or_else(|| item.get("name").and_then(|v| v.as_str()));
        if let Some(name) = name {
            models.push(name.to_string());
            if let Some(mi) = extract_model_info(item) {
                info.insert(name.to_string(), mi);
            }
        }
    }

    if models.is_empty() {
        return Err(format!(
            "上游返回的模型列表为空: {}",
            body.chars().take(300).collect::<String>()
        ));
    }

    models.sort();
    models.dedup();
    Ok((models, info))
}

fn extract_model_info(item: &serde_json::Value) -> Option<ModelInfo> {
    let context_length = find_u64_key(
        item,
        &[
            "context_window",
            "context_length",
            "max_context_length",
            "max_input_tokens",
            "input_token_limit",
        ],
    );
    let max_tokens = find_u64_key(
        item,
        &[
            "max_output_tokens",
            "max_tokens",
            "max_output",
            "output_token_limit",
        ],
    );

    if context_length.is_some() || max_tokens.is_some() {
        Some(ModelInfo {
            context_length,
            max_tokens,
        })
    } else {
        None
    }
}

fn find_u64_key(value: &serde_json::Value, keys: &[&str]) -> Option<u64> {
    for key in keys {
        if let Some(v) = value.get(key) {
            if let Some(n) = v.as_u64() {
                if n > 0 {
                    return Some(n);
                }
            }
        }
    }
    None
}

#[tauri::command]
pub async fn test_provider_target(
    state: State<'_, AppState>,
    provider: ProviderConfig,
    api_key: String,
    model_name: String,
) -> Result<ModelTestTargetResult, String> {
    let _ = state;
    let effective_key = if api_key.is_empty() {
        let saved = keyring::get_key_string(&provider.id).unwrap_or_default();
        if saved.is_empty() {
            return Err("请先填写 API Key".to_string());
        }
        saved
    } else {
        api_key
    };
    test_target_with_key(&provider, &effective_key, &model_name).await
}

async fn test_target_with_key(
    provider: &ProviderConfig,
    api_key: &str,
    model_name: &str,
) -> Result<ModelTestTargetResult, String> {
    let upstream_url = format!(
        "{}/chat/completions",
        provider.base_url.trim_end_matches('/')
    );
    // 草稿配置：一次性客户端 + 统一超时策略。
    let mut builder = reqwest::Client::builder().timeout(provider_timeout(provider));
    if provider.disable_proxy {
        builder = builder.no_proxy();
    }
    let client = builder
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {e}"))?;

    let body = json!({
        "model": model_name,
        "messages": [{"role": "user", "content": "ping"}],
        "max_tokens": 10
    });

    let start = std::time::Instant::now();
    let response = client
        .post(&upstream_url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await;
    let latency_ms = start.elapsed().as_millis() as u64;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if status.is_success() {
                Ok(ModelTestTargetResult {
                    provider_id: provider.id.clone(),
                    model_name: model_name.to_string(),
                    online: true,
                    latency_ms: Some(latency_ms),
                    error: None,
                })
            } else {
                let err = classify_http_error(status, &text);
                Ok(ModelTestTargetResult {
                    provider_id: provider.id.clone(),
                    model_name: model_name.to_string(),
                    online: false,
                    latency_ms: Some(latency_ms),
                    error: Some(err),
                })
            }
        }
        Err(e) => Ok(ModelTestTargetResult {
            provider_id: provider.id.clone(),
            model_name: model_name.to_string(),
            online: false,
            latency_ms: Some(latency_ms),
            error: Some(format!("连接失败: {e}")),
        }),
    }
}
