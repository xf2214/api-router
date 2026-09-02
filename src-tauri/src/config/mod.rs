pub mod persist;
pub mod types;

#[allow(unused_imports)]
pub use types::{
    AccessPoint, AppConfig, CacheConfig, CacheMode, CircuitBreakerConfig, FallbackConfig,
    GroupMember, ModelDefinition, ModelGroup, ModelInfo, ModelMapping, ModelTarget, ProviderConfig,
    RetryConfig, RoutingStrategy,
};

#[allow(unused_imports)]
pub use persist::{export_yaml, load, save};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn detects_unknown_provider() {
        let config = AppConfig {
            models: vec![ModelMapping {
                local_name: "gpt-4o".to_string(),
                targets: vec![ModelTarget {
                    provider_id: "missing".to_string(),
                    model_name: "gpt-4o".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
