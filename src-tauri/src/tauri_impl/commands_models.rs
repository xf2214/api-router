use tauri::State;
use tracing::info;

use crate::{
    config::{self, ModelDefinition, ModelGroup},
    state::AppState,
};

#[tauri::command]
pub async fn get_model_definitions(state: State<'_, AppState>) -> Result<Vec<ModelDefinition>, String> {
    let config = state.inner.config.read().await.clone();
    Ok(config.model_definitions)
}

#[tauri::command]
pub async fn save_model_definition(
    state: State<'_, AppState>,
    def: ModelDefinition,
) -> Result<(), String> {
    if def.id.trim().is_empty() {
        return Err("Model definition id cannot be empty".to_string());
    }

    let path = state.inner.config_path.clone();
    let mut config = state.inner.config.write().await;

    for ap in &def.access_points {
        if ap.provider_id.trim().is_empty() || config.find_provider(&ap.provider_id).is_none() {
            return Err(format!("Access point references unknown provider {}", ap.provider_id));
        }
        if ap.upstream_model_name.trim().is_empty() {
            return Err("upstream_model_name cannot be empty".to_string());
        }
    }

    if let Some(idx) = config.model_definitions.iter().position(|d| d.id == def.id) {
        config.model_definitions[idx] = def;
    } else {
        config.model_definitions.push(def);
    }

    config::save(&config, &path).map_err(|e| e.to_string())?;
    info!("Model definitions saved");
    Ok(())
}

#[tauri::command]
pub async fn delete_model_definition(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let path = state.inner.config_path.clone();
    let mut config = state.inner.config.write().await;
    config.model_definitions.retain(|d| d.id != id);
    config::save(&config, &path).map_err(|e| e.to_string())?;
    info!("Model definition {} deleted", id);
    Ok(())
}

#[tauri::command]
pub async fn get_groups(state: State<'_, AppState>) -> Result<Vec<ModelGroup>, String> {
    let config = state.inner.config.read().await.clone();
    Ok(config.groups)
}

#[tauri::command]
pub async fn save_group(state: State<'_, AppState>, group: ModelGroup) -> Result<(), String> {
    if group.name.trim().is_empty() {
        return Err("Group name cannot be empty".to_string());
    }
    if group.name.trim() == "默认" {
        return Err("'默认' is a reserved group and cannot be created".to_string());
    }

    let path = state.inner.config_path.clone();
    let mut config = state.inner.config.write().await;

    // 注意：不再校验 members 里每个 model 是否存在 —— 因为 members 现在仅作为
    // 顺序/权重覆盖，成员归属完全由 ModelMapping.group 字段决定。
    // 在写入前做一次 normalize 清理脏数据。
    let mut normalized = group;
    config.normalize_group_members_in_place(&mut normalized);

    if let Some(idx) = config.groups.iter().position(|g| g.name == normalized.name) {
        config.groups[idx] = normalized;
    } else {
        config.groups.push(normalized);
    }

    // 对所有 groups 执行一次 normalize（移除无效成员引用）
    config.normalize_all_groups();

    config::save(&config, &path).map_err(|e| e.to_string())?;
    info!("Group configuration saved");
    Ok(())
}

#[tauri::command]
pub async fn delete_group(state: State<'_, AppState>, name: String) -> Result<(), String> {
    let path = state.inner.config_path.clone();
    let mut config = state.inner.config.write().await;
    config.groups.retain(|g| g.name != name);
    // 删除后对所有分组和孤立引用做一次清理
    config.normalize_all_groups();
    config::save(&config, &path).map_err(|e| e.to_string())?;
    info!("Group {} deleted", name);
    Ok(())
}
