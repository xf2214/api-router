use std::collections::HashSet;

use crate::config::{AppConfig, ModelMapping, ModelTarget};
use crate::state::AppStateInner;

use super::routing_types::ResolvedTarget;

pub fn available_tiers(targets: &[ModelTarget]) -> Vec<u32> {
    let mut tiers: Vec<u32> = targets.iter().map(|t| t.tier).collect();
    tiers.sort_unstable();
    tiers.dedup();
    tiers
}

pub async fn resolve_tier_candidates<'a>(
    config: &'a AppConfig,
    state: &'a AppStateInner,
    mapping: &'a ModelMapping,
    targets: &[ModelTarget],
    tier: u32,
    excluded: &HashSet<(String, String)>,
    allow_degraded: bool,
) -> Vec<ResolvedTarget<'a>> {
    let health_snapshot = state.all_health_statuses().await;
    let health_map: std::collections::HashMap<&str, &crate::state::ProviderHealth> = health_snapshot
        .iter()
        .map(|h| (h.provider_id.as_str(), h))
        .collect();

    let cb_config = mapping.effective_cb_config(&config.fallback);

    let mut enabled: Vec<ResolvedTarget<'a>> = Vec::new();
    let mut filtered: Vec<ResolvedTarget<'a>> = Vec::new();

    for t in targets {
        if t.tier != tier {
            continue;
        }
        if excluded.contains(&(t.provider_id.clone(), t.model_name.clone())) {
            continue;
        }
        let Some(provider) = config.find_provider(&t.provider_id) else {
            continue;
        };
        if !provider.enabled {
            continue;
        }

        let resolved = ResolvedTarget { provider, target: t.clone() };
        enabled.push(resolved);

        let should_skip = match health_map.get(t.provider_id.as_str()) {
            Some(h) => {
                h.should_skip_for_routing(
                    &state.circuit_breaker,
                    &t.provider_id,
                    &cb_config,
                    cb_config.cooldown_interval_ms,
                )
                .await
            }
            None => {
                state
                    .circuit_breaker
                    .is_open(&t.provider_id, &cb_config)
                    .await
            }
        } || state.is_target_on_cooldown(&t.provider_id, &t.model_name).await;

        if !should_skip {
            filtered.push(ResolvedTarget { provider, target: t.clone() });
        }
    }

    if !filtered.is_empty() || !allow_degraded {
        filtered
    } else {
        enabled
    }
}
