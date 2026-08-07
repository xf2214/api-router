use tauri::State;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::{
    config::ModelMapping,
    server,
    state::AppState,
};

use super::commands_config::ServerStatus;
use super::commands_provider::{test_target_chat_completion, ModelTestResult, ModelTestTargetResult};

#[tauri::command]
pub async fn start_server(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    if state.inner.is_server_running().await {
        let port = state.inner.config.read().await.port;
        return Ok(ServerStatus { running: true, port });
    }

    let port = state.inner.config.read().await.port;
    let cancel = CancellationToken::new();
    state.inner.set_server_token(cancel.clone()).await;

    match server::start_server(AppState { inner: state.inner.clone() }, port, cancel).await {
        Ok(addr) => {
            info!("Local server started on {}", addr);
            Ok(ServerStatus { running: true, port: addr.port() })
        }
        Err(e) => {
            error!("Failed to start server: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn stop_server(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    state.inner.stop_server().await;
    let port = state.inner.config.read().await.port;
    Ok(ServerStatus { running: false, port })
}

#[tauri::command]
pub async fn get_server_status(state: State<'_, AppState>) -> Result<ServerStatus, String> {
    let running = state.inner.is_server_running().await;
    let port = state.inner.config.read().await.port;
    Ok(ServerStatus { running, port })
}

#[tauri::command]
pub async fn test_model_connection(
    state: State<'_, AppState>,
    local_name: String,
) -> Result<ModelTestResult, String> {
    let config = state.inner.config.read().await.clone();
    let mapping = config
        .find_model(&local_name)
        .ok_or_else(|| format!("Model {local_name} not found"))?
        .clone();
    test_model_mapping(&state, mapping).await
}

#[tauri::command]
pub async fn test_model_config(
    state: State<'_, AppState>,
    model: crate::config::ModelMapping,
) -> Result<ModelTestResult, String> {
    test_model_mapping(&state, model).await
}

async fn test_model_mapping(
    state: &AppState,
    model: ModelMapping,
) -> Result<ModelTestResult, String> {
    let config = state.inner.config.read().await.clone();
    let mut results = Vec::new();

    for target in &model.targets {
        let provider = match config.find_provider(&target.provider_id) {
            Some(p) => p.clone(),
            None => {
                results.push(ModelTestTargetResult {
                    provider_id: target.provider_id.clone(),
                    model_name: target.model_name.clone(),
                    online: false,
                    latency_ms: None,
                    error: Some("提供商不存在".to_string()),
                });
                continue;
            }
        };

        let result = test_target_chat_completion(&provider, &target.model_name).await;
        results.push(result);
    }

    Ok(ModelTestResult {
        local_name: model.local_name,
        targets: results,
    })
}
