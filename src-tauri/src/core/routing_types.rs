use std::borrow::Cow;

use crate::config::{AppConfig, ModelMapping, ModelTarget};

#[derive(Debug, Clone)]
pub struct ResolvedTarget<'a> {
    pub provider: &'a crate::config::ProviderConfig,
    pub target: ModelTarget,
}

pub fn resolve_access_points<'a>(
    config: &'a AppConfig,
    mapping: &'a ModelMapping,
) -> Vec<ModelTarget> {
    if let Some(id) = &mapping.model_id {
        if let Some(definition) = config.find_model_definition(id) {
            return definition
                .access_points
                .iter()
                .filter(|ap| ap.enabled)
                .map(|ap| ModelTarget {
                    provider_id: ap.provider_id.clone(),
                    model_name: ap.upstream_model_name.clone(),
                    weight: ap.weight,
                    override_params: None,
                    tier: 1,
                })
                .collect();
        }
    }
    mapping.targets.clone()
}

pub fn effective_targets<'a>(
    config: &'a AppConfig,
    mapping: &'a ModelMapping,
) -> Cow<'a, [ModelTarget]> {
    match &mapping.model_id {
        Some(id) if config.find_model_definition(id).is_some() => {
            Cow::Owned(resolve_access_points(config, mapping))
        }
        _ => Cow::Borrowed(&mapping.targets),
    }
}
