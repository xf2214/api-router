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
    std::fs::write(path, content)?;
    Ok(())
}

#[allow(dead_code)]
pub fn export_yaml(config: &AppConfig) -> Result<String, AppError> {
    config.validate()?;
    let content = serde_yaml::to_string(config)?;
    Ok(content)
}
