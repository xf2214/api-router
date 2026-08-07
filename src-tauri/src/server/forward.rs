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
) -> Result<(Response, Option<u16>, Option<crate::metrics::TokenUsage>, u32), AppError> {
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
            is_retryable_status(reqwest::StatusCode::from_u16(*status).unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR))
        }
        AppError::Request(e) => is_retryable_error(e),
        _ => false,
    }
}

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

        let metrics_handle = state.inner.metrics.clone();
        let provider_id = resolved.provider.id.to_string();
        let upstream_model = resolved.target.model_name.to_string();
        let runtime_for_tpm = runtime.clone();

        let buffer_for_map = buffer.clone();
        let errored_for_map = errored.clone();
        let stream = upstream.bytes_stream().map(move |result| {
            let mut buf = buffer_for_map.lock().unwrap();
            match result {
                Ok(bytes) => {
                    let out = buf.push(&bytes);
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
                let mut buf = buffer.lock().unwrap();
                (buf.flush(), buf.take_usage())
            };

            let stream_error = errored.load(std::sync::atomic::Ordering::Relaxed);

            if stream_error {
                metrics_handle.convert_to_failure(&provider_id, &upstream_model).await;
            }
            if let Some(u) = usage {
                let total = u.total_tokens;
                metrics_handle.add_usage(&provider_id, &upstream_model, u).await;
                runtime_for_tpm.record_tokens(total).await;
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

pub(crate) async fn check_local_auth(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
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

pub(crate) fn add_trace_header(mut response: Response, trace: &crate::trace::RouteTrace) -> Response {
    if !trace.decisions.is_empty() {
        if let Ok(val) = HeaderValue::from_str(&trace.to_header_value()) {
            response.headers_mut().insert(
                HeaderName::from_static("x-route-trace"),
                val,
            );
        }
    }
    response
}
