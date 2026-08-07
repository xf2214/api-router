use std::collections::HashSet;
use std::time::Instant;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use tracing::{debug, info, warn};

use crate::{
    config::AppConfig,
    error::AppError,
    metrics::{elapsed_ms, new_request_id, RequestLog},
    router::{
        available_tiers, effective_targets, resolve_group_member, resolve_tier_candidates,
        select_target,
    },
    state::AppState,
    trace::RouteTrace,
    transform::is_compact_request,
};

use super::forward::{
    add_trace_header, check_local_auth, forward_with_retry,
    started_as_ms,
};

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

pub async fn list_models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    check_local_auth(&state, &headers).await?;

    let config = state.inner.config.read().await.clone();
    let mut data = Vec::new();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    for mapping in &config.models {
        let owner = mapping
            .targets
            .first()
            .and_then(|t| config.find_provider(&t.provider_id))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "api-router".to_string());

        data.push(json!({
            "id": mapping.local_name,
            "object": "model",
            "created": now as i64,
            "owned_by": owner
        }));
    }

    for group in &config.groups {
        data.push(json!({
            "id": group.name,
            "object": "model",
            "created": now as i64,
            "owned_by": "api-router"
        }));
    }

    Ok(Json(json!({
        "object": "list",
        "data": data
    })))
}

pub async fn chat_completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    handle_completion(State(state), headers, Json(body), "chat/completions").await
}

pub async fn completions(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    handle_completion(State(state), headers, Json(body), "completions").await
}

pub async fn embeddings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Result<Response, AppError> {
    handle_completion(State(state), headers, Json(body), "embeddings").await
}

pub fn resolve_stream(body: &Value, endpoint: &str, default_stream: bool) -> bool {
    match body.get("stream").and_then(|v| v.as_bool()) {
        Some(stream) => stream,
        None => {
            if endpoint == "chat/completions" || endpoint == "completions" {
                default_stream
            } else {
                false
            }
        }
    }
}

pub(crate) fn upstream_status_from_error(err: &AppError) -> Option<u16> {
    match err {
        AppError::Upstream { status, .. } => Some(*status),
        _ => None,
    }
}

async fn handle_completion(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(mut body): Json<Value>,
    endpoint: &'static str,
) -> Result<Response, AppError> {
    check_local_auth(&state, &headers).await?;

    let local_model = body
        .get("model")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Config("Missing 'model' field".to_string()))?
        .to_string();

    let config = state.inner.config.read().await.clone();

    let is_stream = resolve_stream(&body, endpoint, config.default_stream);
    body["stream"] = Value::Bool(is_stream);
    if is_stream && body.get("stream_options").is_none() {
        body["stream_options"] = json!({ "include_usage": true });
    }

    let mut trace = RouteTrace::new();

    let is_compact = is_compact_request(&body);
    trace.set_compact(is_compact);
    if is_compact {
        trace.record("compact_detected", None, None, None, None);
    }

    if let Some(mapping) = config.find_model(&local_model) {
        let request_id = new_request_id();
        let resp = try_mapping(
            &state,
            &config,
            mapping,
            &mut body,
            endpoint,
            is_stream,
            &request_id,
            &local_model,
            false,
            &mut trace,
        )
        .await?;
        return Ok(add_trace_header(resp, &trace));
    }

    if let Some(group) = config.find_group(&local_model) {
        let request_id = new_request_id();
        let mut excluded_members: HashSet<String> = HashSet::new();
        let last_error;

        loop {
            let resolved = match resolve_group_member(&config, &state.inner, group, &excluded_members).await {
                Ok(r) => r,
                Err(e) => {
                    last_error = Some(e);
                    trace.record("failure", None, None, None, Some(last_error.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "all members exhausted".to_string())));
                    return Err(last_error.unwrap_or_else(|| AppError::NoAvailableBackend(local_model.clone())));
                }
            };

            trace.record("select_target", None, Some(resolved.provider.id.clone()), Some(resolved.target.model_name.clone()), None);

            let member_name = group
                .members
                .iter()
                .filter_map(|m| {
                    config.find_model(&m.local_name).and_then(|mp| {
                        let eff = crate::router::effective_targets(&config, mp);
                        if eff.iter().any(|t| {
                            t.provider_id == resolved.provider.id
                                && t.model_name == resolved.target.model_name
                        }) {
                            Some(m.local_name.clone())
                        } else {
                            None
                        }
                    })
                })
                .next();

            let member_name = match member_name {
                Some(name) => name,
                None => {
                    if group.fallback_enabled {
                        trace.record("fallback", None, None, None, Some("member not found in config".to_string()));
                        continue;
                    } else {
                        trace.record("failure", None, None, None, Some("member not found in config".to_string()));
                        return Err(AppError::NoAvailableBackend(local_model.clone()));
                    }
                }
            };

            let Some(member_mapping) = config.find_model(&member_name) else {
                if group.fallback_enabled {
                    trace.record("fallback", None, None, None, Some(format!("member mapping not found: {}", member_name)));
                    excluded_members.insert(member_name);
                    continue;
                } else {
                    trace.record("failure", None, None, None, Some(format!("member mapping not found: {}", member_name)));
                    return Err(AppError::NoAvailableBackend(local_model.clone()));
                }
            };

            let mut member_body = body.clone();
            match try_mapping(
                &state,
                &config,
                member_mapping,
                &mut member_body,
                endpoint,
                is_stream,
                &request_id,
                &local_model,
                true,
                &mut trace,
            )
            .await
            {
                Ok(resp) => return Ok(add_trace_header(resp, &trace)),
                Err(e) => {
                    if group.fallback_enabled {
                        trace.record("fallback", None, None, None, Some(e.to_string()));
                        excluded_members.insert(member_name);
                        continue;
                    } else {
                        trace.record("failure", None, None, None, Some(e.to_string()));
                        return Err(e);
                    }
                }
            }
        }
    }

    trace.record("failure", None, None, None, Some(format!("model not found: {}", local_model)));
    Err(AppError::ModelNotFound(local_model))
}

async fn try_mapping(
    state: &AppState,
    config: &AppConfig,
    mapping: &crate::config::ModelMapping,
    body: &mut Value,
    endpoint: &'static str,
    is_stream: bool,
    request_id: &str,
    local_model: &str,
    fell_back: bool,
    trace: &mut RouteTrace,
) -> Result<Response, AppError> {
    let retry_config = mapping.effective_retry(&config.fallback);
    let cb_config = mapping.effective_cb_config(&config.fallback);
    let targets = effective_targets(config, mapping);
    let tiers = available_tiers(&targets);
    if tiers.is_empty() {
        trace.record("failure", None, None, None, Some("no available tiers".to_string()));
        return Err(AppError::NoAvailableBackend(local_model.to_string()));
    }

    let started = Instant::now();
    debug!(request_id = %request_id, local_model = %local_model, "try_mapping start");

    let mut attempted: HashSet<(String, String)> = HashSet::new();
    let mut last_error: Option<AppError> = None;

    for tier in &tiers {
        let tier_fell_back = fell_back || *tier > tiers[0];
        trace.record("select_tier", Some(*tier), None, None, None);
        loop {
            let candidates = resolve_tier_candidates(config, &state.inner, mapping, &targets, *tier, &attempted, true).await;
            if candidates.is_empty() {
                break;
            }

            let resolved = select_target(mapping.strategy, &candidates, &state.inner).await?;
            attempted.insert((resolved.provider.id.to_string(), resolved.target.model_name.to_string()));

            trace.record("select_target", Some(*tier), Some(resolved.provider.id.to_string()), Some(resolved.target.model_name.to_string()), None);

            let attempt_started = Instant::now();
            match forward_with_retry(
                state,
                &resolved,
                body,
                endpoint,
                is_stream,
                local_model,
                request_id,
                tier_fell_back,
                started,
                retry_config.clone(),
                cb_config.clone(),
            )
            .await
            {
                Ok((resp, status, usage, retries)) => {
                    info!(
                        request_id = %request_id,
                        tier = tier,
                        provider = %resolved.provider.id,
                        "Request succeeded for {}", local_model
                    );
                    trace.record("success", Some(*tier), Some(resolved.provider.id.to_string()), Some(resolved.target.model_name.to_string()), None);
                    state
                        .inner
                        .metrics
                        .record(RequestLog {
                            request_id: request_id.to_string(),
                            local_model: local_model.to_string(),
                            provider_id: resolved.provider.id.to_string(),
                            upstream_model: resolved.target.model_name.to_string(),
                            endpoint: endpoint.to_string(),
                            stream: is_stream,
                            started_at_ms: started_as_ms(),
                            duration_ms: elapsed_ms(started),
                            status,
                            success: true,
                            error: None,
                            retries,
                            fell_back: tier_fell_back,
                            usage,
                        })
                        .await;
                    return Ok(resp);
                }
                Err(e) => {
                    warn!(
                        request_id = %request_id,
                        tier = tier,
                        provider = %resolved.provider.id,
                        "Target failed for {}: {}", local_model, e
                    );
                    trace.record("failure", Some(*tier), Some(resolved.provider.id.to_string()), Some(resolved.target.model_name.to_string()), Some(e.to_string()));
                    state
                        .inner
                        .metrics
                        .record(RequestLog {
                            request_id: request_id.to_string(),
                            local_model: local_model.to_string(),
                            provider_id: resolved.provider.id.to_string(),
                            upstream_model: resolved.target.model_name.to_string(),
                            endpoint: endpoint.to_string(),
                            stream: is_stream,
                            started_at_ms: started_as_ms(),
                            duration_ms: elapsed_ms(attempt_started),
                            status: upstream_status_from_error(&e),
                            success: false,
                            error: Some(e.to_string()),
                            retries: 0,
                            fell_back: tier_fell_back,
                            usage: None,
                        })
                        .await;
                    last_error = Some(e);
                }
            }
        }

        if !mapping.fallback_enabled && *tier == tiers[0] {
            let err_msg = last_error.as_ref().map(|e| e.to_string()).unwrap_or_else(|| "no available backend".to_string());
            trace.record("failure", Some(*tier), None, None, Some(err_msg));
            return Err(last_error.unwrap_or_else(|| AppError::NoAvailableBackend(local_model.to_string())));
        }
    }

    if let Some(e) = last_error {
        return Err(e);
    }
    Err(AppError::NoAvailableBackend(local_model.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_stream_uses_client_value() {
        let body = json!({ "stream": false });
        assert!(!resolve_stream(&body, "chat/completions", true));
        assert!(!resolve_stream(&body, "completions", true));
        assert!(!resolve_stream(&body, "embeddings", true));

        let body = json!({ "stream": true });
        assert!(resolve_stream(&body, "chat/completions", false));
        assert!(resolve_stream(&body, "completions", false));
        assert!(resolve_stream(&body, "embeddings", false));
    }

    #[test]
    fn resolve_stream_fallback_for_completion_endpoints() {
        let body = json!({ "model": "gpt-4o" });
        assert!(resolve_stream(&body, "chat/completions", true));
        assert!(resolve_stream(&body, "completions", true));
        assert!(!resolve_stream(&body, "chat/completions", false));
        assert!(!resolve_stream(&body, "completions", false));
    }

    #[test]
    fn resolve_stream_embeddings_always_non_streaming() {
        let body = json!({ "model": "text-embedding-3" });
        assert!(!resolve_stream(&body, "embeddings", true));
        assert!(!resolve_stream(&body, "embeddings", false));
    }

    #[test]
    fn resolve_stream_rejects_non_boolean_stream() {
        let body = json!({ "stream": "yes" });
        assert!(resolve_stream(&body, "chat/completions", true));
        assert!(!resolve_stream(&body, "embeddings", true));
    }

    #[test]
    fn injects_stream_and_stream_options() {
        let mut body = json!({ "model": "gpt-4o" });
        let is_stream = resolve_stream(&body, "chat/completions", true);
        body["stream"] = Value::Bool(is_stream);
        if is_stream && body.get("stream_options").is_none() {
            body["stream_options"] = json!({ "include_usage": true });
        }
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], true);
    }

    #[test]
    fn preserves_existing_stream_options() {
        let mut body = json!({
            "model": "gpt-4o",
            "stream_options": { "include_usage": false }
        });
        let is_stream = resolve_stream(&body, "chat/completions", true);
        body["stream"] = Value::Bool(is_stream);
        if is_stream && body.get("stream_options").is_none() {
            body["stream_options"] = json!({ "include_usage": true });
        }
        assert_eq!(body["stream"], true);
        assert_eq!(body["stream_options"]["include_usage"], false);
    }

    #[test]
    fn no_stream_options_for_non_streaming() {
        let mut body = json!({ "model": "gpt-4o" });
        let is_stream = resolve_stream(&body, "chat/completions", false);
        body["stream"] = Value::Bool(is_stream);
        if is_stream && body.get("stream_options").is_none() {
            body["stream_options"] = json!({ "include_usage": true });
        }
        assert_eq!(body["stream"], false);
        assert!(body.get("stream_options").is_none());
    }
}
