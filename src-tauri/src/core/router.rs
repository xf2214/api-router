use std::collections::HashSet;

use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::config::{AppConfig, ModelGroup};
use crate::error::AppError;
use crate::state::AppStateInner;

use super::routing_types::{effective_targets, ResolvedTarget};
use super::router_strategies::select_target;
use super::router_tiers::{available_tiers, resolve_tier_candidates};

pub async fn resolve_primary<'a>(
    config: &'a AppConfig,
    state: &'a AppStateInner,
    local_model: &str,
) -> Result<ResolvedTarget<'a>, AppError> {
    let mapping = config
        .find_model(local_model)
        .ok_or_else(|| AppError::ModelNotFound(local_model.to_string()))?;

    let targets = effective_targets(config, mapping);
    let tiers = available_tiers(&targets);
    if tiers.is_empty() {
        return Err(AppError::NoAvailableBackend(local_model.to_string()));
    }

    let excluded = HashSet::new();
    let mut tier_errors: Vec<String> = Vec::new();
    for tier in tiers {
        let candidates = resolve_tier_candidates(config, state, mapping, &targets, tier, &excluded, false).await;
        if candidates.is_empty() {
            tier_errors.push(format!("tier {}: 无可用候选", tier));
            continue;
        }
        let target = select_target(mapping.strategy, &candidates, state).await?;
        return Ok(target);
    }

    Err(AppError::NoAvailableBackend(format!(
        "{} (所有层级均无可用目标): {}",
        local_model,
        tier_errors.join("; ")
    )))
}

pub async fn resolve_group_member<'a>(
    config: &'a AppConfig,
    state: &'a AppStateInner,
    group: &ModelGroup,
    excluded_members: &HashSet<String>,
) -> Result<ResolvedTarget<'a>, AppError> {
    let (candidates, last_error) = build_group_candidates(config, state, group, excluded_members).await;
    if candidates.is_empty() {
        return Err(last_error.unwrap_or_else(|| AppError::NoAvailableBackend(group.name.clone())));
    }

    match group.strategy {
        crate::config::RoutingStrategy::Priority => Ok(candidates[0].0.clone()),
        crate::config::RoutingStrategy::RoundRobin => {
            let idx = state.next_round_robin() % candidates.len();
            Ok(candidates[idx].0.clone())
        }
        crate::config::RoutingStrategy::Weighted => {
            let weights: Vec<u32> = candidates.iter().map(|(_, w)| (*w).max(1)).collect();
            let dist = WeightedIndex::new(&weights)
                .map_err(|e| AppError::Config(format!("Invalid weights: {e}")))?;
            let mut rng = thread_rng();
            Ok(candidates[dist.sample(&mut rng)].0.clone())
        }
        crate::config::RoutingStrategy::LeastBusy => {
            let mut best_idx = 0;
            let mut best_permits = state
                .clients
                .available_permits(&candidates[0].0.provider.id)
                .await;
            for (i, (candidate, _)) in candidates.iter().enumerate().skip(1) {
                let permits = state
                    .clients
                    .available_permits(&candidate.provider.id)
                    .await;
                if permits > best_permits {
                    best_permits = permits;
                    best_idx = i;
                }
            }
            Ok(candidates[best_idx].0.clone())
        }
        crate::config::RoutingStrategy::LatencyBased => {
            let stats = state.metrics.all_stats().await;
            let mut best_idx = 0;
            let mut best_latency = f64::MAX;

            for (i, (candidate, _)) in candidates.iter().enumerate() {
                let provider_id = &candidate.provider.id;
                let provider_stats: Vec<_> = stats
                    .iter()
                    .filter(|s| s.provider_id == *provider_id)
                    .collect();

                let avg_latency = if provider_stats.is_empty() {
                    f64::MAX
                } else {
                    let total_duration: u64 = provider_stats.iter().map(|s| s.total_duration_ms).sum();
                    let total_requests: u64 = provider_stats.iter().map(|s| s.total_requests).sum();
                    if total_requests == 0 {
                        f64::MAX
                    } else {
                        total_duration as f64 / total_requests as f64
                    }
                };

                if avg_latency < best_latency {
                    best_latency = avg_latency;
                    best_idx = i;
                }
            }

            Ok(candidates[best_idx].0.clone())
        }
    }
}

async fn build_group_candidates<'a>(
    config: &'a AppConfig,
    state: &'a AppStateInner,
    group: &ModelGroup,
    excluded_members: &HashSet<String>,
) -> (Vec<(ResolvedTarget<'a>, u32)>, Option<AppError>) {
    let mut candidates = Vec::new();
    let mut last_error: Option<AppError> = None;
    for member in &group.members {
        if excluded_members.contains(&member.local_name) {
            continue;
        }
        if config.find_model(&member.local_name).is_none() {
            continue;
        }
        match resolve_primary(config, state, &member.local_name).await {
            Ok(resolved) => candidates.push((resolved, member.weight)),
            Err(e) => last_error = Some(e),
        }
    }
    (candidates, last_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        AccessPoint, AppConfig, GroupMember, ModelDefinition,
        ModelGroup, ModelMapping, ModelTarget, ProviderConfig, RoutingStrategy,
    };
    use crate::state::ProviderHealth;

    fn test_config() -> AppConfig {
        AppConfig {
            providers: vec![
                ProviderConfig {
                    id: "agg1".to_string(),
                    name: "聚合 1".to_string(),
                    base_url: "https://agg1.example.com/v1".to_string(),
                    enabled: true,
                    ..Default::default()
                },
                ProviderConfig {
                    id: "agg2".to_string(),
                    name: "聚合 2".to_string(),
                    base_url: "https://agg2.example.com/v1".to_string(),
                    enabled: true,
                    ..Default::default()
                },
                ProviderConfig {
                    id: "agg3".to_string(),
                    name: "聚合 3".to_string(),
                    base_url: "https://agg3.example.com/v1".to_string(),
                    enabled: true,
                    ..Default::default()
                },
            ],
            models: vec![ModelMapping {
                local_name: "glm5.2".to_string(),
                strategy: RoutingStrategy::Priority,
                targets: vec![
                    ModelTarget {
                        provider_id: "agg1".to_string(),
                        model_name: "glm-5.2".to_string(),
                        tier: 1,
                        ..Default::default()
                    },
                    ModelTarget {
                        provider_id: "agg2".to_string(),
                        model_name: "glm-5.2".to_string(),
                        tier: 1,
                        ..Default::default()
                    },
                    ModelTarget {
                        provider_id: "agg1".to_string(),
                        model_name: "mimo-2.5".to_string(),
                        tier: 2,
                        ..Default::default()
                    },
                    ModelTarget {
                        provider_id: "agg3".to_string(),
                        model_name: "mimo-2.5".to_string(),
                        tier: 2,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn available_tiers_sorted_unique() {
        let config = test_config();
        let mapping = &config.models[0];
        assert_eq!(available_tiers(&mapping.targets), vec![1, 2]);
    }

    #[tokio::test]
    async fn resolve_primary_uses_tier_1() {
        let config = test_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let resolved = resolve_primary(&config, &state, "glm5.2").await.unwrap();
        assert_eq!(resolved.provider.id, "agg1");
        assert_eq!(resolved.target.model_name, "glm-5.2");
        assert_eq!(resolved.target.tier, 1);
    }

    #[tokio::test]
    async fn resolve_tier_candidates_excludes_attempted() {
        let config = test_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let mapping = &config.models[0];
        let targets = effective_targets(&config, mapping);

        let mut excluded = HashSet::new();
        excluded.insert(("agg1".to_string(), "glm-5.2".to_string()));

        let candidates = resolve_tier_candidates(&config, &state, mapping, &targets, 1, &excluded, false).await;
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider.id, "agg2");
    }

    #[tokio::test]
    async fn prefers_healthy_over_offline() {
        let config = test_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        state
            .set_provider_health(ProviderHealth {
                provider_id: "agg1".to_string(),
                online: false,
                last_checked: Some(now),
                latency_ms: None,
                error: Some("down".to_string()),
            })
            .await;

        let resolved = resolve_primary(&config, &state, "glm5.2").await.unwrap();
        assert_eq!(resolved.provider.id, "agg2");
        assert_eq!(resolved.target.model_name, "glm-5.2");
    }

    #[tokio::test]
    async fn falls_back_to_next_tier_when_tier_1_all_offline() {
        let config = test_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        state
            .set_provider_health(ProviderHealth {
                provider_id: "agg1".to_string(),
                online: false,
                last_checked: Some(now),
                latency_ms: None,
                error: Some("down".to_string()),
            })
            .await;
        state
            .set_provider_health(ProviderHealth {
                provider_id: "agg2".to_string(),
                online: false,
                last_checked: Some(now),
                latency_ms: None,
                error: Some("down".to_string()),
            })
            .await;

        let resolved = resolve_primary(&config, &state, "glm5.2").await.unwrap();
        assert_eq!(resolved.target.model_name, "mimo-2.5");
        assert_eq!(resolved.target.tier, 2);
    }

    fn access_point_config() -> AppConfig {
        AppConfig {
            providers: vec![
                ProviderConfig {
                    id: "agg1".to_string(),
                    name: "聚合 1".to_string(),
                    base_url: "https://agg1.example.com/v1".to_string(),
                    enabled: true,
                    ..Default::default()
                },
                ProviderConfig {
                    id: "agg2".to_string(),
                    name: "聚合 2".to_string(),
                    base_url: "https://agg2.example.com/v1".to_string(),
                    enabled: true,
                    ..Default::default()
                },
            ],
            models: vec![ModelMapping {
                local_name: "glm5.2".to_string(),
                strategy: RoutingStrategy::Priority,
                model_id: Some("glm-def".to_string()),
                ..Default::default()
            }],
            model_definitions: vec![ModelDefinition {
                id: "glm-def".to_string(),
                display_name: "GLM".to_string(),
                access_points: vec![
                    AccessPoint {
                        provider_id: "agg1".to_string(),
                        upstream_model_name: "glm-5.2-pro".to_string(),
                        enabled: true,
                        weight: 3,
                    },
                    AccessPoint {
                        provider_id: "agg2".to_string(),
                        upstream_model_name: "glm-5.2-lite".to_string(),
                        enabled: true,
                        weight: 1,
                    },
                    AccessPoint {
                        provider_id: "agg1".to_string(),
                        upstream_model_name: "glm-5.2-disabled".to_string(),
                        enabled: false,
                        weight: 1,
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn access_points_resolve_from_model_definition() {
        let config = access_point_config();
        let mapping = &config.models[0];
        let targets = effective_targets(&config, mapping);

        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].provider_id, "agg1");
        assert_eq!(targets[0].model_name, "glm-5.2-pro");
        assert_eq!(targets[0].weight, 3);
        assert_eq!(targets[0].tier, 1);
        assert_eq!(targets[0].override_params, None);
        assert_eq!(targets[1].provider_id, "agg2");
        assert_eq!(targets[1].model_name, "glm-5.2-lite");
    }

    #[tokio::test]
    async fn access_points_exclude_disabled() {
        let config = access_point_config();
        let mapping = &config.models[0];
        let targets = effective_targets(&config, mapping);

        assert!(!targets.iter().any(|t| t.model_name == "glm-5.2-disabled"));
    }

    #[tokio::test]
    async fn resolve_primary_uses_model_definition_access_points() {
        let config = access_point_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let resolved = resolve_primary(&config, &state, "glm5.2").await.unwrap();

        assert_eq!(resolved.provider.id, "agg1");
        assert_eq!(resolved.target.model_name, "glm-5.2-pro");
        assert_eq!(resolved.target.tier, 1);
    }

    #[tokio::test]
    async fn effective_targets_fall_back_to_mapping_targets_without_model_id() {
        let mut config = access_point_config();
        config.models[0].model_id = None;
        config.models[0].targets = vec![ModelTarget {
            provider_id: "agg2".to_string(),
            model_name: "fallback-model".to_string(),
            tier: 1,
            ..Default::default()
        }];

        let mapping = &config.models[0];
        let targets = effective_targets(&config, mapping).into_owned();

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].provider_id, "agg2");
        assert_eq!(targets[0].model_name, "fallback-model");
    }

    #[tokio::test]
    async fn effective_targets_fall_back_to_mapping_targets_when_definition_missing() {
        let mut config = access_point_config();
        config.models[0].model_id = Some("missing-def".to_string());
        config.models[0].targets = vec![ModelTarget {
            provider_id: "agg2".to_string(),
            model_name: "fallback-model".to_string(),
            tier: 1,
            ..Default::default()
        }];

        let mapping = &config.models[0];
        let targets = effective_targets(&config, mapping).into_owned();

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].provider_id, "agg2");
        assert_eq!(targets[0].model_name, "fallback-model");
    }

    fn group_test_config() -> AppConfig {
        AppConfig {
            providers: vec![
                ProviderConfig {
                    id: "agg1".to_string(),
                    name: "聚合 1".to_string(),
                    base_url: "https://agg1.example.com/v1".to_string(),
                    enabled: true,
                    ..Default::default()
                },
                ProviderConfig {
                    id: "agg2".to_string(),
                    name: "聚合 2".to_string(),
                    base_url: "https://agg2.example.com/v1".to_string(),
                    enabled: true,
                    ..Default::default()
                },
            ],
            models: vec![
                ModelMapping {
                    local_name: "model-a".to_string(),
                    strategy: RoutingStrategy::Priority,
                    targets: vec![ModelTarget {
                        provider_id: "agg1".to_string(),
                        model_name: "upstream-a".to_string(),
                        tier: 1,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                ModelMapping {
                    local_name: "model-b".to_string(),
                    strategy: RoutingStrategy::Priority,
                    targets: vec![ModelTarget {
                        provider_id: "agg2".to_string(),
                        model_name: "upstream-b".to_string(),
                        tier: 1,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            ],
            groups: vec![ModelGroup {
                name: "g1".to_string(),
                members: vec![
                    GroupMember {
                        local_name: "model-a".to_string(),
                        weight: 1,
                    },
                    GroupMember {
                        local_name: "model-b".to_string(),
                        weight: 1,
                    },
                ],
                strategy: RoutingStrategy::Priority,
                fallback_enabled: true,
            }],
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn resolve_group_member_priority_returns_first_member() {
        let config = group_test_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let group = &config.groups[0];
        let excluded = HashSet::new();
        let resolved = resolve_group_member(&config, &state, group, &excluded)
            .await
            .unwrap();
        assert_eq!(resolved.provider.id, "agg1");
        assert_eq!(resolved.target.model_name, "upstream-a");
    }

    #[tokio::test]
    async fn resolve_group_member_weighted_prefers_heavier_member() {
        let mut config = group_test_config();
        config.groups[0].strategy = RoutingStrategy::Weighted;
        config.groups[0].members[0].weight = 1;
        config.groups[0].members[1].weight = 99;
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let group = &config.groups[0];
        let excluded = HashSet::new();

        let mut heavier_count = 0;
        for _ in 0..1000 {
            let resolved = resolve_group_member(&config, &state, group, &excluded)
                .await
                .unwrap();
            if resolved.provider.id == "agg2" {
                heavier_count += 1;
            }
        }
        assert!(heavier_count > 900);
    }

    #[tokio::test]
    async fn resolve_group_member_round_robin_rotates() {
        let mut config = group_test_config();
        config.groups[0].strategy = RoutingStrategy::RoundRobin;
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let group = &config.groups[0];
        let excluded = HashSet::new();

        let r1 = resolve_group_member(&config, &state, group, &excluded)
            .await
            .unwrap();
        let r2 = resolve_group_member(&config, &state, group, &excluded)
            .await
            .unwrap();
        let r3 = resolve_group_member(&config, &state, group, &excluded)
            .await
            .unwrap();
        let r4 = resolve_group_member(&config, &state, group, &excluded)
            .await
            .unwrap();

        assert_eq!(r1.provider.id, "agg1");
        assert_eq!(r2.provider.id, "agg2");
        assert_eq!(r3.provider.id, "agg1");
        assert_eq!(r4.provider.id, "agg2");
    }

    #[tokio::test]
    async fn resolve_group_member_skips_excluded_members() {
        let config = group_test_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());
        let group = &config.groups[0];
        let mut excluded = HashSet::new();
        excluded.insert("model-a".to_string());

        let resolved = resolve_group_member(&config, &state, group, &excluded)
            .await
            .unwrap();
        assert_eq!(resolved.provider.id, "agg2");
        assert_eq!(resolved.target.model_name, "upstream-b");
    }

    #[tokio::test]
    async fn latency_based_selects_lowest_latency() {
        let mut config = test_config();
        config.models[0].strategy = RoutingStrategy::LatencyBased;
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        state
            .metrics
            .record(crate::metrics::RequestLog {
                request_id: "r1".to_string(),
                local_model: "glm5.2".to_string(),
                provider_id: "agg1".to_string(),
                upstream_model: "glm-5.2".to_string(),
                endpoint: "chat/completions".to_string(),
                stream: false,
                started_at_ms: now,
                duration_ms: 200,
                status: Some(200),
                success: true,
                error: None,
                retries: 0,
                fell_back: false,
                usage: None,
            })
            .await;

        state
            .metrics
            .record(crate::metrics::RequestLog {
                request_id: "r2".to_string(),
                local_model: "glm5.2".to_string(),
                provider_id: "agg2".to_string(),
                upstream_model: "glm-5.2".to_string(),
                endpoint: "chat/completions".to_string(),
                stream: false,
                started_at_ms: now,
                duration_ms: 50,
                status: Some(200),
                success: true,
                error: None,
                retries: 0,
                fell_back: false,
                usage: None,
            })
            .await;

        let resolved = resolve_primary(&config, &state, "glm5.2").await.unwrap();
        assert_eq!(resolved.provider.id, "agg2");
        assert_eq!(resolved.target.model_name, "glm-5.2");
    }

    #[tokio::test]
    async fn latency_based_falls_back_to_first_when_no_metrics() {
        let mut config = test_config();
        config.models[0].strategy = RoutingStrategy::LatencyBased;
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());

        let resolved = resolve_primary(&config, &state, "glm5.2").await.unwrap();
        assert_eq!(resolved.provider.id, "agg1");
    }

    #[tokio::test]
    async fn resolve_group_member_latency_based() {
        let mut config = group_test_config();
        config.groups[0].strategy = RoutingStrategy::LatencyBased;
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        state
            .metrics
            .record(crate::metrics::RequestLog {
                request_id: "r1".to_string(),
                local_model: "model-a".to_string(),
                provider_id: "agg1".to_string(),
                upstream_model: "upstream-a".to_string(),
                endpoint: "chat/completions".to_string(),
                stream: false,
                started_at_ms: now,
                duration_ms: 200,
                status: Some(200),
                success: true,
                error: None,
                retries: 0,
                fell_back: false,
                usage: None,
            })
            .await;

        state
            .metrics
            .record(crate::metrics::RequestLog {
                request_id: "r2".to_string(),
                local_model: "model-b".to_string(),
                provider_id: "agg2".to_string(),
                upstream_model: "upstream-b".to_string(),
                endpoint: "chat/completions".to_string(),
                stream: false,
                started_at_ms: now,
                duration_ms: 50,
                status: Some(200),
                success: true,
                error: None,
                retries: 0,
                fell_back: false,
                usage: None,
            })
            .await;

        let group = &config.groups[0];
        let excluded = HashSet::new();
        let resolved = resolve_group_member(&config, &state, group, &excluded)
            .await
            .unwrap();
        assert_eq!(resolved.provider.id, "agg2");
        assert_eq!(resolved.target.model_name, "upstream-b");
    }

    fn least_busy_config() -> AppConfig {
        AppConfig {
            providers: vec![
                ProviderConfig {
                    id: "busy".to_string(),
                    name: "Busy".to_string(),
                    base_url: "https://busy.example.com/v1".to_string(),
                    enabled: true,
                    concurrency_limit: 2,
                    ..Default::default()
                },
                ProviderConfig {
                    id: "idle".to_string(),
                    name: "Idle".to_string(),
                    base_url: "https://idle.example.com/v1".to_string(),
                    enabled: true,
                    concurrency_limit: 2,
                    ..Default::default()
                },
            ],
            models: vec![ModelMapping {
                local_name: "test-model".to_string(),
                strategy: RoutingStrategy::LeastBusy,
                targets: vec![
                    ModelTarget {
                        provider_id: "busy".to_string(),
                        model_name: "busy-model".to_string(),
                        tier: 1,
                        ..Default::default()
                    },
                    ModelTarget {
                        provider_id: "idle".to_string(),
                        model_name: "idle-model".to_string(),
                        tier: 1,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn least_busy_selects_less_busy_provider() {
        let config = least_busy_config();
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());

        let busy_runtime = state.clients.get(&config.providers[0]).await.unwrap();
        let _guard = busy_runtime.acquire().await.unwrap();

        let _idle_runtime = state.clients.get(&config.providers[1]).await.unwrap();

        let resolved = resolve_primary(&config, &state, "test-model").await.unwrap();
        assert_eq!(resolved.provider.id, "idle");
        assert_eq!(resolved.target.model_name, "idle-model");
    }

    #[tokio::test]
    async fn least_busy_selects_first_when_no_concurrency_limit() {
        let mut config = least_busy_config();
        config.providers[0].concurrency_limit = 0;
        config.providers[1].concurrency_limit = 0;
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());

        let resolved = resolve_primary(&config, &state, "test-model").await.unwrap();
        assert_eq!(resolved.provider.id, "busy");
    }

    #[tokio::test]
    async fn least_busy_selects_most_capacity_when_all_idle() {
        let config = AppConfig {
            providers: vec![
                ProviderConfig {
                    id: "small".to_string(),
                    name: "Small".to_string(),
                    base_url: "https://small.example.com/v1".to_string(),
                    enabled: true,
                    concurrency_limit: 1,
                    ..Default::default()
                },
                ProviderConfig {
                    id: "large".to_string(),
                    name: "Large".to_string(),
                    base_url: "https://large.example.com/v1".to_string(),
                    enabled: true,
                    concurrency_limit: 10,
                    ..Default::default()
                },
            ],
            models: vec![ModelMapping {
                local_name: "cap-model".to_string(),
                strategy: RoutingStrategy::LeastBusy,
                targets: vec![
                    ModelTarget {
                        provider_id: "small".to_string(),
                        model_name: "small-model".to_string(),
                        tier: 1,
                        ..Default::default()
                    },
                    ModelTarget {
                        provider_id: "large".to_string(),
                        model_name: "large-model".to_string(),
                        tier: 1,
                        ..Default::default()
                    },
                ],
                ..Default::default()
            }],
            ..Default::default()
        };
        let state = AppStateInner::new(config.clone(), std::path::PathBuf::new());

        let _small_rt = state.clients.get(&config.providers[0]).await.unwrap();
        let _large_rt = state.clients.get(&config.providers[1]).await.unwrap();

        let resolved = resolve_primary(&config, &state, "cap-model").await.unwrap();
        assert_eq!(resolved.provider.id, "large");
        assert_eq!(resolved.target.model_name, "large-model");
    }
}
