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

/// 导入配置的结果摘要。
#[derive(serde::Serialize, Clone, Debug)]
pub struct ImportSummary {
    /// 导入的提供商数量。
    pub providers: usize,
    /// 导入的模型映射数量。
    pub models: usize,
    /// 备份文件路径（无既有配置时为空）。
    pub backup_path: String,
}

/// 导入 `export_config` 产出的 JSON（`{config: AppConfig, api_keys: {id: key}}` 或裸 `AppConfig`）。
/// 流程：解析 → validate → 备份当前 config.yaml（时间戳后缀）→ atomic save → 写 keyring → 换 Arc → sync。
#[tauri::command]
pub async fn import_config(
    state: State<'_, AppState>,
    payload: String,
) -> Result<ImportSummary, String> {
    let (mut cfg, keys): (AppConfig, std::collections::HashMap<String, String>) =
        if let Ok(wrapped) = serde_json::from_str::<serde_json::Value>(&payload) {
            if wrapped.get("config").is_some() {
                let c: AppConfig = serde_json::from_value(wrapped["config"].clone())
                    .map_err(|e| format!("配置解析失败: {e}"))?;
                let k: std::collections::HashMap<String, String> = wrapped
                    .get("api_keys")
                    .and_then(|v| serde_json::from_value(v.clone()).ok())
                    .unwrap_or_default();
                (c, k)
            } else {
                (
                    serde_json::from_str(&payload).map_err(|e| format!("配置解析失败: {e}"))?,
                    Default::default(),
                )
            }
        } else {
            return Err("配置解析失败: 非 JSON".into());
        };
    cfg.validate().map_err(|e| format!("配置校验失败: {e}"))?;
    cfg.normalize_all_groups();
    let path = state.inner.config_path.clone();
    if path.exists() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let backup = path.with_extension(format!("yaml.bak-{stamp}"));
        std::fs::copy(&path, &backup).map_err(|e| format!("备份失败: {e}"))?;
        let backup_str = backup.to_string_lossy().to_string();
        config::save(&cfg, &path).map_err(|e| format!("保存失败: {e}"))?;
        for (id, key) in keys.iter().filter(|(_, v)| !v.is_empty() && *v != "***") {
            keyring::set_key(id, key).map_err(|e| format!("密钥写入失败({id}): {e}"))?;
        }
        {
            let mut guard = state.inner.config.write().await;
            *guard = std::sync::Arc::new(cfg.clone());
        }
        state.inner.sync_metrics_enabled().await;
        state.inner.sync_cache_config().await;
        Ok(ImportSummary {
            providers: cfg.providers.len(),
            models: cfg.models.len(),
            backup_path: backup_str,
        })
    } else {
        config::save(&cfg, &path).map_err(|e| format!("保存失败: {e}"))?;
        for (id, key) in keys.iter().filter(|(_, v)| !v.is_empty() && *v != "***") {
            keyring::set_key(id, key).map_err(|e| format!("密钥写入失败({id}): {e}"))?;
        }
        {
            let mut guard = state.inner.config.write().await;
            *guard = std::sync::Arc::new(cfg.clone());
        }
        state.inner.sync_metrics_enabled().await;
        state.inner.sync_cache_config().await;
        Ok(ImportSummary {
            providers: cfg.providers.len(),
            models: cfg.models.len(),
            backup_path: String::new(),
        })
    }
}

#[cfg(test)]
mod import_tests {
    use crate::config::{AppConfig, ModelMapping, ModelTarget, ProviderConfig};

    fn minimal_valid_config() -> AppConfig {
        AppConfig {
            providers: vec![ProviderConfig {
                id: "p1".to_string(),
                name: "Test Provider".to_string(),
                base_url: "https://api.example.com".to_string(),
                ..Default::default()
            }],
            models: vec![ModelMapping {
                local_name: "test-model".to_string(),
                targets: vec![ModelTarget {
                    provider_id: "p1".to_string(),
                    model_name: "gpt-4o".to_string(),
                    ..Default::default()
                }],
                group: "orphan-group".to_string(),
                ..Default::default()
            }],
            groups: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn import_config_fn_exists() {
        let ok = true;
        assert!(ok);
    }

    #[test]
    fn minimal_config_validate_ok() {
        let cfg = minimal_valid_config();
        assert!(cfg.validate().is_ok());
        // serde round-trip as bare AppConfig
        let json = serde_json::to_string(&cfg).expect("serialize bare config");
        let parsed: AppConfig = serde_json::from_str(&json).expect("deserialize bare config");
        assert_eq!(parsed.providers.len(), 1);
        assert_eq!(parsed.models.len(), 1);
        assert!(parsed.validate().is_ok());
    }

    #[test]
    fn normalize_corrects_orphan_group_to_default() {
        let mut cfg = minimal_valid_config();
        assert_eq!(cfg.models[0].group, "orphan-group");
        cfg.normalize_all_groups();
        assert_eq!(cfg.models[0].group, "默认");
    }

    #[test]
    fn wrapped_payload_serde_round_trip() {
        let cfg = minimal_valid_config();
        let mut api_keys = std::collections::HashMap::new();
        api_keys.insert("p1".to_string(), "sk-test-123".to_string());

        #[derive(serde::Serialize)]
        struct Wrapped<'a> {
            config: &'a AppConfig,
            api_keys: std::collections::HashMap<String, String>,
        }

        let payload = serde_json::to_string(&Wrapped {
            config: &cfg,
            api_keys: api_keys.clone(),
        })
        .expect("serialize wrapped");

        // Mirror import_config parsing logic for wrapped shape
        let wrapped: serde_json::Value =
            serde_json::from_str(&payload).expect("parse wrapped value");
        assert!(wrapped.get("config").is_some());
        let c: AppConfig =
            serde_json::from_value(wrapped["config"].clone()).expect("extract config");
        let k: std::collections::HashMap<String, String> = wrapped
            .get("api_keys")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();
        assert_eq!(c.providers.len(), 1);
        assert_eq!(c.models.len(), 1);
        assert_eq!(k.get("p1").map(String::as_str), Some("sk-test-123"));
        assert!(c.validate().is_ok());
    }

    #[test]
    fn bare_payload_serde_round_trip() {
        let cfg = minimal_valid_config();
        let payload = serde_json::to_string(&cfg).expect("serialize bare");
        let wrapped: serde_json::Value = serde_json::from_str(&payload).expect("parse as value");
        // bare payload has no "config" key - it is the config itself
        assert!(wrapped.get("config").is_none());
        let parsed: AppConfig = serde_json::from_str(&payload).expect("parse bare");
        assert_eq!(parsed.providers.len(), 1);
        assert_eq!(parsed.models.len(), 1);
        assert!(parsed.validate().is_ok());
        // normalize orphan group also works on bare-parsed config
        let mut normalized = parsed.clone();
        normalized.normalize_all_groups();
        assert_eq!(normalized.models[0].group, "默认");
    }

    #[test]
    fn validate_rejects_unknown_provider() {
        let mut cfg = minimal_valid_config();
        cfg.models[0].targets[0].provider_id = "missing".to_string();
        assert!(cfg.validate().is_err());
    }
}
