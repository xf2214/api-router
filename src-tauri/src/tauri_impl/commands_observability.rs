use tauri::State;

use crate::{
    metrics::{ProviderStat, RequestLog},
    state::AppState,
};

#[tauri::command]
pub async fn get_request_logs(
    state: State<'_, AppState>,
    limit: Option<usize>,
) -> Result<Vec<RequestLog>, String> {
    let limit = limit.unwrap_or(100).min(500);
    Ok(state.inner.metrics.recent_logs(limit).await)
}

#[tauri::command]
pub async fn get_request_stats(state: State<'_, AppState>) -> Result<Vec<ProviderStat>, String> {
    Ok(state.inner.metrics.all_stats().await)
}

#[tauri::command]
pub async fn clear_request_logs(state: State<'_, AppState>) -> Result<(), String> {
    state.inner.metrics.clear().await;
    Ok(())
}
