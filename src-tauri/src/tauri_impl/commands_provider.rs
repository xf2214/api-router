use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::AUTHORIZATION;
use serde::Serialize;
use serde_json::json;
use tauri::State;
use tracing::info;

use crate::{
    config::{self, ModelInfo, ProviderConfig},
    keyring,
    state::{AppState, ProviderHealth},
};

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
pub async fn delete_provider(state: State<'_, AppState>, provider_id: String) -> Result<(), String> {
    let path = state.inner.config_path.clone();

    {
        let mut config = state.inner.config.write().await;
        config.providers.retain(|p| p.id != provider_id);
        config.models.iter_mut().for_each(|m| {
            m.targets.retain(|t| t.provider_id != provider_id);
        });
        config.models.retain(|m| !m.targets.is_empty());
        config::save(&config, &path).map_err(|e| e.to_string())?;
    }

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

    let health = probe_provider(&provider).await;
    state.inner.set_provider_health(health.clone()).await;
    Ok(health)
}

#[tauri::command]
pub async fn check_all_providers_health(state: State<'_, AppState>) -> Result<Vec<ProviderHealth>, String> {
    Ok(check_all_providers_health_internal(&state).await)
}

pub async fn check_all_providers_health_internal(state: &AppState) -> Vec<ProviderHealth> {
    let config = state.inner.config.read().await.clone();
    let mut results = Vec::new();

    for provider in &config.providers {
        if !provider.enabled {
            continue;
        }
        let health = probe_provider(provider).await;
        state.inner.set_provider_health(health.clone()).await;
        results.push(health);
    }

    results
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

async fn probe_provider(provider: &ProviderConfig) -> ProviderHealth {
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
    let timeout = Duration::from_secs(provider.timeout_seconds.clamp(5, 30));
    let mut builder = reqwest::Client::builder().timeout(timeout);
    if provider.disable_proxy {
        builder = builder.no_proxy();
    }
    let client = match builder.build() {
        Ok(c) => c,
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
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header("User-Agent", "api-router/0.1.0")
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
    let timeout = Duration::from_secs(provider.timeout_seconds.clamp(5, 30));
    let mut builder = reqwest::Client::builder().timeout(timeout);
    if provider.disable_proxy {
        builder = builder.no_proxy();
    }
    let client = match builder.build() {
        Ok(c) => c,
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
    } else if status.as_u16() == 404 || lowered.contains("model_not_found") || lowered.contains("model does not exist") {
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
    let timeout = Duration::from_secs(provider.timeout_seconds.clamp(5, 60));
    info!("Fetching models from {}", upstream_url);
    let mut builder = reqwest::Client::builder().timeout(timeout);
    if provider.disable_proxy {
        builder = builder.no_proxy();
    }
    let client = builder
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(&upstream_url)
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header("User-Agent", "api-router/0.1.0")
        .header("Accept", "application/json")
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            if !status.is_success() {
                let summary = body.chars().take(200).collect::<String>();
                return Err(format!(
                    "上游返回错误 [{}]: {}",
                    status.as_u16(),
                    summary
                ));
            }
            let (models, info) = parse_models_response(&body)?;
            Ok(ProviderModelsResult { models, raw_json: body, info })
        }
        Err(e) => Err(format!("连接失败: {e}")),
    }
}

fn parse_models_response(body: &str) -> Result<(Vec<String>, HashMap<String, ModelInfo>), String> {
    let value: serde_json::Value = serde_json::from_str(body).map_err(|e| format!("解析响应失败: {e}"))?;

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
    let timeout = Duration::from_secs(provider.timeout_seconds.clamp(5, 30));
    let client = reqwest::Client::builder()
        .timeout(timeout)
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
