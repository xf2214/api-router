use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub port: u16,
    pub local_api_token: Option<String>,
    pub enable_logging: bool,
    pub providers: Vec<ProviderConfig>,
    pub models: Vec<ModelMapping>,
    pub groups: Vec<ModelGroup>,
    pub fallback: FallbackConfig,
    pub enable_auto_health_check: bool,
    pub health_check_interval_seconds: u64,
    pub cache: CacheConfig,
    pub bind_address: String,
    pub enable_cors: bool,
    pub log_level: String,
    pub log_retention_days: u64,
    pub default_stream: bool,
    pub model_definitions: Vec<ModelDefinition>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: 6123,
            local_api_token: None,
            enable_logging: true,
            providers: Vec::new(),
            models: Vec::new(),
            groups: Vec::new(),
            fallback: FallbackConfig::default(),
            enable_auto_health_check: true,
            health_check_interval_seconds: 300,
            cache: CacheConfig::default(),
            bind_address: "127.0.0.1".to_string(),
            enable_cors: true,
            log_level: "info".to_string(),
            log_retention_days: 30,
            default_stream: true,
            model_definitions: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn validate(&self) -> Result<(), AppError> {
        for provider in &self.providers {
            if provider.id.is_empty() {
                return Err(AppError::Config("Provider id cannot be empty".to_string()));
            }
            if provider.base_url.is_empty() {
                return Err(AppError::Config(format!(
                    "Provider {} base_url cannot be empty",
                    provider.id
                )));
            }
        }

        let provider_ids: std::collections::HashSet<_> =
            self.providers.iter().map(|p| p.id.clone()).collect();
        for mapping in &self.models {
            if mapping.local_name.is_empty() {
                return Err(AppError::Config("Model local_name cannot be empty".to_string()));
            }
            for target in &mapping.targets {
                if !provider_ids.contains(&target.provider_id) {
                    return Err(AppError::Config(format!(
                        "Model {} references unknown provider {}",
                        mapping.local_name, target.provider_id
                    )));
                }
            }
        }

        Ok(())
    }

    pub fn find_provider(&self, id: &str) -> Option<&ProviderConfig> {
        self.providers.iter().find(|p| p.id == id)
    }

    pub fn find_model(&self, local_name: &str) -> Option<&ModelMapping> {
        self.models.iter().find(|m| m.local_name == local_name)
    }

    pub fn find_group(&self, name: &str) -> Option<&ModelGroup> {
        self.groups.iter().find(|g| g.name == name)
    }

    pub fn find_model_definition(&self, id: &str) -> Option<&ModelDefinition> {
        self.model_definitions.iter().find(|m| m.id == id)
    }

    /// 对单个 group 做成员数据清理：
    /// - 去除重复（同一个 local_name 出现多次，保留第一个）
    /// - 去除 local_name 找不到对应 ModelMapping，或对应 mapping.group != group.name 的脏项
    pub fn normalize_group_members_in_place(&self, group: &mut ModelGroup) {
        let mut seen = std::collections::HashSet::new();
        let mut cleaned = Vec::new();
        for m in group.members.drain(..) {
            if !seen.insert(m.local_name.clone()) {
                continue;
            }
            let valid = self
                .find_model(&m.local_name)
                .map(|mm| mm.group == group.name)
                .unwrap_or(false);
            if valid {
                cleaned.push(m);
            }
        }
        group.members = cleaned;
    }

    /// 对所有 groups 执行 normalize，并把「指向不存在分组的模型」改回 "默认"。
    pub fn normalize_all_groups(&mut self) {
        use std::collections::HashSet;

        // 先快照 group_names，再分阶段做，避免 &self 和 &mut self 重叠借用
        let group_names: HashSet<String> =
            self.groups.iter().map(|g| g.name.clone()).collect();

        // 阶段 1：对每个 group 清理成员（通过模型快照验证有效性，避免同时可变借用 groups 和调用 self.find_model）
        let models_snapshot: Vec<(String, String)> = self
            .models
            .iter()
            .map(|m| (m.local_name.clone(), m.group.clone()))
            .collect();
        for g in self.groups.iter_mut() {
            let mut seen = HashSet::new();
            let mut cleaned = Vec::new();
            for m in g.members.drain(..) {
                if !seen.insert(m.local_name.clone()) {
                    continue;
                }
                let valid = models_snapshot
                    .iter()
                    .any(|(n, grp)| n == &m.local_name && grp == &g.name);
                if valid {
                    cleaned.push(m);
                }
            }
            g.members = cleaned;
        }

        // 阶段 2：修复 model.group 孤立引用
        for m in self.models.iter_mut() {
            if m.group.is_empty() {
                m.group = "默认".to_string();
                continue;
            }
            if m.group != "默认" && !group_names.contains(&m.group) {
                m.group = "默认".to_string();
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub timeout_seconds: u64,
    pub qps_limit: u32,
    pub concurrency_limit: u32,
    pub tpm_limit: u32,
    pub enabled: bool,
    pub default_models: Vec<String>,
    pub extra_headers: HashMap<String, String>,
    pub disable_proxy: bool,
    pub model_info: HashMap<String, ModelInfo>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            base_url: String::new(),
            timeout_seconds: 60,
            qps_limit: 0,
            concurrency_limit: 0,
            tpm_limit: 0,
            enabled: true,
            default_models: Vec::new(),
            extra_headers: HashMap::new(),
            disable_proxy: false,
            model_info: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelInfo {
    pub context_length: Option<u64>,
    pub max_tokens: Option<u64>,
}

impl Default for ModelInfo {
    fn default() -> Self {
        Self {
            context_length: None,
            max_tokens: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelMapping {
    pub local_name: String,
    pub strategy: RoutingStrategy,
    pub fallback_enabled: bool,
    pub max_retries: u32,
    pub targets: Vec<ModelTarget>,
    pub group: String,
    pub context_length: Option<u64>,
    pub max_tokens: Option<u64>,
    pub cb_config: Option<CircuitBreakerConfig>,
    pub retry: Option<RetryConfig>,
    pub model_id: Option<String>,
}

impl Default for ModelMapping {
    fn default() -> Self {
        Self {
            local_name: String::new(),
            strategy: RoutingStrategy::Priority,
            fallback_enabled: true,
            max_retries: 2,
            targets: Vec::new(),
            group: "默认".to_string(),
            context_length: None,
            max_tokens: None,
            cb_config: None,
            retry: None,
            model_id: None,
        }
    }
}

impl ModelMapping {
    pub fn effective_cb_config(&self, fallback: &FallbackConfig) -> CircuitBreakerConfig {
        self.cb_config.clone().unwrap_or_else(|| fallback.cb_config.clone())
    }

    pub fn effective_retry(&self, fallback: &FallbackConfig) -> RetryConfig {
        self.retry.clone().unwrap_or_else(|| RetryConfig {
            attempts: fallback.default_retries,
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GroupMember {
    pub local_name: String,
    pub weight: u32,
}

impl Default for GroupMember {
    fn default() -> Self {
        Self {
            local_name: String::new(),
            weight: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelGroup {
    pub name: String,
    pub members: Vec<GroupMember>,
    pub strategy: RoutingStrategy,
    pub fallback_enabled: bool,
}

impl Default for ModelGroup {
    fn default() -> Self {
        Self {
            name: String::new(),
            members: Vec::new(),
            strategy: RoutingStrategy::Priority,
            fallback_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelTarget {
    pub provider_id: String,
    pub model_name: String,
    pub weight: u32,
    pub override_params: Option<serde_json::Map<String, serde_json::Value>>,
    pub tier: u32,
}

impl Default for ModelTarget {
    fn default() -> Self {
        Self {
            provider_id: String::new(),
            model_name: String::new(),
            weight: 1,
            override_params: None,
            tier: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AccessPoint {
    pub provider_id: String,
    pub upstream_model_name: String,
    pub enabled: bool,
    pub weight: u32,
}

impl Default for AccessPoint {
    fn default() -> Self {
        Self {
            provider_id: String::new(),
            upstream_model_name: String::new(),
            enabled: true,
            weight: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelDefinition {
    pub id: String,
    pub display_name: String,
    pub context_length: Option<u32>,
    pub max_tokens: Option<u32>,
    pub auto_aggregate: bool,
    pub access_points: Vec<AccessPoint>,
}

impl Default for ModelDefinition {
    fn default() -> Self {
        Self {
            id: String::new(),
            display_name: String::new(),
            context_length: None,
            max_tokens: None,
            auto_aggregate: true,
            access_points: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingStrategy {
    Priority,
    Weighted,
    RoundRobin,
    LeastBusy,
    LatencyBased,
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::Priority
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FallbackConfig {
    pub default_retries: u32,
    pub timeout_seconds: u64,
    pub cb_config: CircuitBreakerConfig,
}

impl Default for FallbackConfig {
    fn default() -> Self {
        Self {
            default_retries: 2,
            timeout_seconds: 60,
            cb_config: CircuitBreakerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub failure_threshold_percentage: Option<u32>,
    pub cooldown_interval_ms: u64,
    pub failure_status_codes: Option<Vec<u16>>,
    pub minimum_requests: Option<u32>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_threshold_percentage: None,
            cooldown_interval_ms: 60_000,
            failure_status_codes: None,
            minimum_requests: Some(10),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RetryConfig {
    pub attempts: u32,
    pub on_status_codes: Option<Vec<u16>>,
    pub use_retry_after_headers: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            attempts: 2,
            on_status_codes: None,
            use_retry_after_headers: true,
        }
    }
}

impl From<u32> for RetryConfig {
    fn from(attempts: u32) -> Self {
        Self {
            attempts,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CacheConfig {
    pub enabled: bool,
    pub mode: CacheMode,
    pub max_age_seconds: u64,
    pub max_entries: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: CacheMode::Simple,
            max_age_seconds: 3600,
            max_entries: 1000,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CacheMode {
    Simple,
}

impl Default for CacheMode {
    fn default() -> Self {
        CacheMode::Simple
    }
}
