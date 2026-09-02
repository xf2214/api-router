use std::path::Path;

use crate::error::AppError;

use super::types::AppConfig;

pub fn load(path: &Path) -> Result<AppConfig, AppError> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = std::fs::read_to_string(path)?;
    let config: AppConfig = serde_yaml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}

pub fn save(config: &AppConfig, path: &Path) -> Result<(), AppError> {
    config.validate()?;
    let content = serde_yaml::to_string(config)?;

    // 原子写入：先写同目录临时文件，再 rename 覆盖目标文件。
    // 直接 write 中途断电/崩溃会留下截断的 YAML，导致下次启动配置丢失。
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."));
    let tmp_path = parent.join(format!(
        "{}.tmp",
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "config.yaml".to_string())
    ));
    std::fs::write(&tmp_path, content)?;
    // std::fs::rename 在 Windows 上使用 MOVEFILE_REPLACE_EXISTING，可覆盖已存在文件。
    if let Err(e) = std::fs::rename(&tmp_path, path) {
        // 清理残留的临时文件
        let _ = std::fs::remove_file(&tmp_path);
        return Err(e.into());
    }
    Ok(())
}

#[allow(dead_code)]
pub fn export_yaml(config: &AppConfig) -> Result<String, AppError> {
    config.validate()?;
    let content = serde_yaml::to_string(config)?;
    Ok(content)
}
