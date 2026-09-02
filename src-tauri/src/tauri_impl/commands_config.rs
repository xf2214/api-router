use std::collections::HashMap;

use serde::Serialize;
use tauri::State;
use tracing::info;

use crate::{
    config::{self, AppConfig},
    keyring,
    state::AppState,
};

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
}

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.inner.config.read().await.clone();
    Ok((*config).clone())
}

/// 导出配置（含 API Key）。
///
/// `include_keys`（前端传 camelCase `includeKeys`）：
/// - `true`：返回真实 Key（用户显式导出备份场景）；
/// - `false`/缺省：所有 Key 以 `"***"` 掩码返回，防止意外序列化到日志或剪贴板。
#[tauri::command]
pub async fn export_config(
    state: State<'_, AppState>,
    include_keys: Option<bool>,
) -> Result<String, String> {
    let include_keys = include_keys.unwrap_or(false);
    let config = state.inner.config.read().await.clone();

    let mut api_keys = HashMap::new();
    for provider in &config.providers {
        if include_keys {
            let key = keyring::get_key_string(&provider.id).unwrap_or_default();
            api_keys.insert(provider.id.clone(), key);
        } else {
            api_keys.insert(provider.id.clone(), "***".to_string());
        }
    }

    #[derive(Serialize)]
    struct ExportedConfig {
        config: AppConfig,
        api_keys: HashMap<String, String>,
    }

    serde_json::to_string_pretty(&ExportedConfig {
        config: (*config).clone(),
        api_keys,
    })
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_config(
    state: State<'_, AppState>,
    config: AppConfig,
    keys: HashMap<String, String>,
) -> Result<(), String> {
    config.validate().map_err(|e| e.to_string())?;

    let path = state.inner.config_path.clone();

    // 持久化前做一次分组脏数据清理
    let mut normalized = config;
    normalized.normalize_all_groups();

    config::save(&normalized, &path).map_err(|e| e.to_string())?;

    for (provider_id, api_key) in keys {
        if !api_key.is_empty() {
            keyring::set_key(&provider_id, &api_key).map_err(|e| e.to_string())?;
        }
    }

    {
        let mut guard = state.inner.config.write().await;
        *guard = std::sync::Arc::new(normalized);
    }

    state.inner.sync_metrics_enabled().await;
    state.inner.sync_cache_config().await;

    info!("Configuration saved to {}", path.display());
    Ok(())
}
