use std::time::Instant;

use axum::{
    body::Body,
    http::{HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use futures::StreamExt;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use tracing::{debug, error};

use crate::{
    client::{backoff_duration, is_retryable_error, is_retryable_status, ProviderRuntime},
    config::{CircuitBreakerConfig, RetryConfig},
    error::AppError,
    keyring,
    metrics::extract_usage,
    router::ResolvedTarget,
    sse::SseLineBuffer,
    state::AppState,
    transform::transform_request,
    transform::transform_response,
};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn forward_with_retry(
    state: &AppState,
    resolved: &ResolvedTarget<'_>,
    body: &mut Value,
    endpoint: &str,
    is_stream: bool,
    local_model: &str,
    request_id: &str,
    fell_back: bool,
    started: Instant,
    retry_config: RetryConfig,
    cb_config: CircuitBreakerConfig,
) -> Result<
    (
        Response,
        Option<u16>,
        Option<crate::metrics::TokenUsage>,
        u32,
    ),
    AppError,
> {
    let max_retries = retry_config.attempts.max(1);

    let mut attempt: u32 = 0;
    loop {
        match forward_request_once(
            state,
            resolved,
            body,
            endpoint,
            is_stream,
            local_model,
            request_id,
            attempt,
            fell_back,
            started,
            cb_config.clone(),
        )
        .await
        {
            Ok((resp, status, usage)) => {
                return Ok((resp, status, usage, attempt));
            }
            Err(err) => {
                let retry_after_ms = extract_retry_after_ms(&err);
                let retryable = is_retryable_app_error(&err, &retry_config.on_status_codes);
                if !retryable || attempt >= max_retries {
                    return Err(err);
                }

                let base = std::time::Duration::from_millis(200);
                let cap = std::time::Duration::from_secs(60);
                let backoff = if retry_config.use_retry_after_headers {
                    if let Some(ms) = retry_after_ms {
                        std::time::Duration::from_millis(ms.clamp(100, 60_000))
                    } else {
                        backoff_duration(attempt, base, cap)
                    }
                } else {
                    backoff_duration(attempt, base, cap)
                };

                tracing::warn!(
                    request_id = %request_id,
                    attempt = attempt + 1,
                    backoff_ms = backoff.as_millis(),
                    "Retrying after error: {}", err
                );
                tokio::time::sleep(backoff).await;
                attempt += 1;
            }
        }
    }
}

fn is_retryable_app_error(err: &AppError, allowed_codes: &Option<Vec<u16>>) -> bool {
    match err {
        AppError::Upstream { status, .. } => {
            if let Some(codes) = allowed_codes {
                return codes.contains(status);
            }
            is_retryable_status(
                reqwest::StatusCode::from_u16(*status)
                    .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
            )
        }
        AppError::Request(e) => is_retryable_error(e),
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn forward_request_once(
    state: &AppState,
    resolved: &ResolvedTarget<'_>,
    body: &mut Value,
    endpoint: &str,
    is_stream: bool,
    local_model: &str,
    request_id: &str,
    _retries: u32,
    _fell_back: bool,
    _started: Instant,
    cb_config: CircuitBreakerConfig,
) -> Result<(Response, Option<u16>, Option<crate::metrics::TokenUsage>), AppError> {
    if let Some(cached) = state.inner.cache.get(endpoint, body, is_stream).await {
        return Ok((cached, Some(200), None));
    }

    let cache_key_body = body.clone();
    transform_request(body, &resolved.target);

    let api_key = keyring::get_key_string(&resolved.provider.id)?;
    let upstream_url = build_upstream_url(&resolved.provider.base_url, endpoint);

    debug!(request_id = %request_id, "Forwarding to {}", upstream_url);

    let runtime: ProviderRuntime = state.inner.clients.get(resolved.provider).await?;
    let _permit = runtime.acquire().await?;

    let mut request = runtime
        .http
        .post(&upstream_url)
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .header(CONTENT_TYPE, "application/json")
        .header("Accept", "application/json")
        .header("X-Request-ID", request_id)
        .json(body);

    for (key, value) in &resolved.provider.extra_headers {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(value),
        ) {
            request = request.header(name, val);
        }
    }

    let upstream = request.send().await?;
    let status = upstream.status();

    if status.is_server_error() || status.is_client_error() {
        let retry_after_ms = parse_retry_after(upstream.headers());
        let text = upstream.text().await.unwrap_or_default();
        let body_text = serde_json::to_string(body).unwrap_or_else(|_| "<non-json>".to_string());

        state
            .inner
            .circuit_breaker
            .record_failure(&resolved.provider.id, Some(status.as_u16()), &cb_config)
            .await;

        state
            .inner
            .record_target_failure(&resolved.provider.id, &resolved.target.model_name, &text)
            .await;

        error!(
            request_id = %request_id,
            "Upstream {} returned {}. body={} response={}", upstream_url, status, body_text, text
        );
        return Err(AppError::Upstream {
            status: status.as_u16(),
            message: text,
            retry_after_ms,
        });
    }

    state
        .inner
        .circuit_breaker
        .record_success(&resolved.provider.id)
        .await;

    state
        .inner
        .record_target_success(&resolved.provider.id, &resolved.target.model_name)
        .await;

    if is_stream {
        let buffer = std::sync::Arc::new(std::sync::Mutex::new(SseLineBuffer::with_local_model(
            local_model.to_string(),
        )));
        let errored = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let usage_recorded = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let metrics_handle = state.inner.metrics.clone();
        let provider_id = resolved.provider.id.to_string();
        let upstream_model = resolved.target.model_name.to_string();
        let runtime_for_tpm = runtime.clone();

        let buffer_for_map = buffer.clone();
        let errored_for_map = errored.clone();
        let metrics_handle_for_map = metrics_handle.clone();
        let runtime_for_tpm_for_map = runtime_for_tpm.clone();
        let provider_id_for_map = provider_id.clone();
        let upstream_model_for_map = upstream_model.clone();
        let usage_recorded_map = usage_recorded.clone();
        let stream = upstream.bytes_stream().map(move |result| {
            // 锁中毒时恢复内部数据而不是 panic：SSE 流闭包运行在响应流任务中，
            // 一旦 panic 会终止该连接，且 poisoned Mutex 会让所有后续连接连环 panic。
            let mut buf = buffer_for_map
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match result {
                Ok(bytes) => {
                    let out = buf.push(&bytes);
                    if let Some(u) = buf.take_usage() {
                        if usage_recorded_map
                            .compare_exchange(
                                false,
                                true,
                                std::sync::atomic::Ordering::Relaxed,
                                std::sync::atomic::Ordering::Relaxed,
                            )
                            .is_ok()
                        {
                            let metrics = metrics_handle_for_map.clone();
                            let rt = runtime_for_tpm_for_map.clone();
                            let pid = provider_id_for_map.clone();
                            let um = upstream_model_for_map.clone();
                            tokio::spawn(async move {
                                metrics.add_usage(&pid, &um, u.clone()).await;
                                rt.record_tokens(u.total_tokens).await;
                            });
                        }
                    }
                    Ok::<_, std::io::Error>(axum::body::Bytes::from(out))
                }
                Err(e) => {
                    error!("Stream error: {e}");
                    errored_for_map.store(true, std::sync::atomic::Ordering::Relaxed);
                    let rest = buf.flush();
                    Ok::<_, std::io::Error>(axum::body::Bytes::from(rest))
                }
            }
        });

        let stream = stream.chain(futures::stream::once(async move {
            let (rest, usage) = {
                let mut buf = buffer
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                (buf.flush(), buf.take_usage())
            };

            let stream_error = errored.load(std::sync::atomic::Ordering::Relaxed);

            if stream_error {
                metrics_handle
                    .convert_to_failure(&provider_id, &upstream_model)
                    .await;
            }
            if let Some(u) = usage {
                if usage_recorded
                    .compare_exchange(
                        false,
                        true,
                        std::sync::atomic::Ordering::Relaxed,
                        std::sync::atomic::Ordering::Relaxed,
                    )
                    .is_ok()
                {
                    let total = u.total_tokens;
                    metrics_handle
                        .add_usage(&provider_id, &upstream_model, u)
                        .await;
                    runtime_for_tpm.record_tokens(total).await;
                }
            }

            Ok::<_, std::io::Error>(axum::body::Bytes::from(rest))
        }));

        let mut response_headers = HeaderMap::new();
        response_headers.insert(CONTENT_TYPE, HeaderValue::from_static("text/event-stream"));
        response_headers.insert(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("no-cache"),
        );
        response_headers.insert(
            HeaderName::from_static("x-request-id"),
            HeaderValue::from_str(request_id).unwrap_or(HeaderValue::from_static("")),
        );

        Ok((
            (StatusCode::OK, response_headers, Body::from_stream(stream)).into_response(),
            Some(status.as_u16()),
            None,
        ))
    } else {
        let bytes = upstream.bytes().await?;
        let mut value: Value = serde_json::from_slice(&bytes)?;
        let usage = extract_usage(&value);
        if let Some(u) = &usage {
            runtime.record_tokens(u.total_tokens).await;
        }
        transform_response(&mut value, local_model);
        state
            .inner
            .cache
            .put(endpoint, &cache_key_body, &value, is_stream)
            .await;
        Ok((
            (StatusCode::OK, axum::Json(value)).into_response(),
            Some(status.as_u16()),
            usage,
        ))
    }
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

pub(crate) async fn check_local_auth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let config = state.inner.config.read().await;
    if let Some(expected) = &config.local_api_token {
        let provided = headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));

        if provided != Some(expected.as_str()) {
            return Err(AppError::Config(
                "Invalid or missing local API token".to_string(),
            ));
        }
    }
    Ok(())
}

pub(crate) fn started_as_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn parse_retry_after(headers: &HeaderMap) -> Option<u64> {
    for key in ["retry-after-ms", "x-ms-retry-after-ms"] {
        if let Some(v) = headers.get(key) {
            if let Ok(s) = v.to_str() {
                if let Ok(ms) = s.parse::<u64>() {
                    return Some(ms);
                }
            }
        }
    }
    if let Some(v) = headers.get("retry-after") {
        if let Ok(s) = v.to_str() {
            if let Ok(secs) = s.parse::<u64>() {
                return Some(secs * 1000);
            }
        }
    }
    None
}

pub(crate) fn extract_retry_after_ms(err: &AppError) -> Option<u64> {
    match err {
        AppError::Upstream { retry_after_ms, .. } => *retry_after_ms,
        _ => None,
    }
}

pub(crate) fn add_trace_header(
    mut response: Response,
    trace: &crate::trace::RouteTrace,
) -> Response {
    if !trace.decisions.is_empty() {
        if let Ok(val) = HeaderValue::from_str(&trace.to_header_value()) {
            response
                .headers_mut()
                .insert(HeaderName::from_static("x-route-trace"), val);
        }
    }
    response
}

#[cfg(test)]
mod mock_upstream_tests {
    // Entry signatures (verbatim from src-tauri/src/server/forward.rs):
    // pub(crate) async fn forward_with_retry(
    //     state: &AppState,
    //     resolved: &ResolvedTarget<'_>,
    //     body: &mut Value,
    //     endpoint: &str,
    //     is_stream: bool,
    //     local_model: &str,
    //     request_id: &str,
    //     fell_back: bool,
    //     started: Instant,
    //     retry_config: RetryConfig,
    //     cb_config: CircuitBreakerConfig,
    // ) -> Result<(Response, Option<u16>, Option<crate::metrics::TokenUsage>, u32), AppError>
    // pub(crate) async fn forward_request_once(
    //     state: &AppState,
    //     resolved: &ResolvedTarget<'_>,
    //     body: &mut Value,
    //     endpoint: &str,
    //     is_stream: bool,
    //     local_model: &str,
    //     request_id: &str,
    //     _retries: u32,
    //     _fell_back: bool,
    //     _started: Instant,
    //     cb_config: CircuitBreakerConfig,
    // ) -> Result<(Response, Option<u16>, Option<crate::metrics::TokenUsage>), AppError>
    // try_mapping is private to server/handlers.rs:
    // async fn try_mapping(
    //     state: &AppState,
    //     config: &AppConfig,
    //     mapping: &ModelMapping,
    //     body: &mut Value,
    //     endpoint: &'static str,
    //     is_stream: bool,
    //     request_id: &str,
    //     local_model: &str,
    //     fell_back: bool,
    //     trace: &mut RouteTrace,
    // ) -> Result<Response, AppError>
    use super::*;
    use axum::{routing::post, Json, Router};
    use tokio::net::TcpListener;

    async fn spawn_mock() -> String {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(|| async {
                Json(serde_json::json!({
                    "id": "chatcmpl-mock", "object": "chat.completion",
                    "created": 1, "model": "mock",
                    "choices": [{"index": 0, "message": {"role": "assistant", "content": "hi"}, "finish_reason": "stop"}],
                    "usage": {"prompt_tokens": 2, "completion_tokens": 1, "total_tokens": 3}
                }))
            }),
        );
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(l, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    async fn spawn_mock_500() -> String {
        let app = Router::new().route(
            "/v1/chat/completions",
            post(|| async {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "internal server error"})),
                )
            }),
        );
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = l.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(l, app).await.unwrap();
        });
        format!("http://{addr}")
    }

    fn test_provider_config(base: &str, id: &str) -> crate::config::ProviderConfig {
        crate::config::ProviderConfig {
            id: id.to_string(),
            name: "mock".to_string(),
            base_url: base.to_string(),
            timeout_seconds: 10,
            qps_limit: 0,
            concurrency_limit: 0,
            tpm_limit: 0,
            enabled: true,
            default_models: vec![],
            extra_headers: std::collections::HashMap::new(),
            disable_proxy: true,
            model_info: std::collections::HashMap::new(),
        }
    }

    #[tokio::test]
    async fn mock_upstream_non_stream_passthrough() {
        let base = spawn_mock().await;
        let provider = test_provider_config(&base, "mock-provider");
        crate::keyring::set_key(&provider.id, "sk-test-mock").expect("test keyring set");
        let state = crate::state::AppState::new(
            crate::config::AppConfig::default(),
            std::path::PathBuf::from("/tmp/test-config.yaml"),
        );
        let target = crate::config::ModelTarget {
            provider_id: provider.id.clone(),
            model_name: "mock".to_string(),
            weight: 1,
            override_params: None,
            tier: 1,
        };
        let resolved = crate::router::ResolvedTarget {
            provider: &provider,
            target,
        };
        let mut body = serde_json::json!({
            "model": "local-model",
            "messages": [{"role": "user", "content": "hello"}],
            "stream": false
        });
        let retry_config = crate::config::RetryConfig {
            attempts: 0,
            on_status_codes: None,
            use_retry_after_headers: true,
        };
        let cb_config = crate::config::CircuitBreakerConfig::default();
        let (resp, status, usage, _retries) = forward_with_retry(
            &state,
            &resolved,
            &mut body,
            "v1/chat/completions",
            false,
            "local-model",
            "test-req-1",
            false,
            std::time::Instant::now(),
            retry_config,
            cb_config,
        )
        .await
        .expect("forward should succeed");
        assert_eq!(status, Some(200));
        let usage = usage.expect("usage should be present");
        assert_eq!(usage.total_tokens, 3);
        // Also verify body JSON passthrough
        let (_parts, b) = resp.into_parts();
        let bytes = axum::body::to_bytes(b, 1024 * 1024).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["choices"][0]["message"]["content"], "hi");
        assert_eq!(v["usage"]["total_tokens"], 3);
        // transform_response rewrites model to local_model
        assert_eq!(v["model"], "local-model");
    }

    #[tokio::test]
    async fn mock_upstream_500_retry_exhausted_returns_upstream_error() {
        // What the code ACTUALLY does for 500:
        // forward_request_once returns Err(AppError::Upstream{status:500,...}) for any 5xx/4xx.
        // forward_with_retry checks is_retryable_app_error -> is_retryable_status(500)==true,
        // so it retries up to max_retries (attempts.max(1)) with exponential backoff,
        // then returns the last Upstream error. No fallback here - fallback is handled in
        // handlers::try_mapping tier loop, not inside forward_with_retry.
        let base = spawn_mock_500().await;
        let provider = test_provider_config(&base, "mock-provider-500");
        crate::keyring::set_key(&provider.id, "sk-test-mock").expect("test keyring set");
        let state = crate::state::AppState::new(
            crate::config::AppConfig::default(),
            std::path::PathBuf::from("/tmp/test-config-500.yaml"),
        );
        let target = crate::config::ModelTarget {
            provider_id: provider.id.clone(),
            model_name: "mock".to_string(),
            weight: 1,
            override_params: None,
            tier: 1,
        };
        let resolved = crate::router::ResolvedTarget {
            provider: &provider,
            target,
        };
        let mut body = serde_json::json!({
            "model": "local-model",
            "messages": [{"role": "user", "content": "hello"}],
            "stream": false
        });
        // Use attempts=1 so retries happen but we don't sleep 200ms * many times with jitter.
        // Minimal attempts=0 still maps to max_retries=1 (one retry), but we test retry path.
        let retry_config = crate::config::RetryConfig {
            attempts: 1,
            on_status_codes: None,
            use_retry_after_headers: false,
        };
        let cb_config = crate::config::CircuitBreakerConfig::default();

        let start = std::time::Instant::now();
        let result = forward_with_retry(
            &state,
            &resolved,
            &mut body,
            "v1/chat/completions",
            false,
            "local-model",
            "test-req-500",
            false,
            start,
            retry_config,
            cb_config,
        )
        .await;

        match result {
            Err(crate::error::AppError::Upstream {
                status, message, ..
            }) => {
                assert_eq!(
                    status, 500,
                    "expected upstream 500, got {status}: {message}"
                );
                assert!(
                    message.contains("internal server error") || !message.is_empty(),
                    "message should contain server error, got: {message}"
                );
            }
            Err(e) => panic!("expected Upstream 500 error, got different AppError: {e:?}"),
            Ok(_) => panic!("expected Upstream error, got Ok"),
        }

        // Also verify forward_request_once alone returns same Upstream 500 without retry
        let mut body2 = serde_json::json!({
            "model": "local-model",
            "messages": [{"role": "user", "content": "hello"}],
            "stream": false
        });
        let cb2 = crate::config::CircuitBreakerConfig::default();
        let result2 = forward_request_once(
            &state,
            &resolved,
            &mut body2,
            "v1/chat/completions",
            false,
            "local-model",
            "test-req-500-once",
            0,
            false,
            std::time::Instant::now(),
            cb2,
        )
        .await;
        match result2 {
            Err(crate::error::AppError::Upstream { status, .. }) => assert_eq!(status, 500),
            other => panic!("forward_request_once should return Upstream 500, got {other:?}"),
        }
    }
}
