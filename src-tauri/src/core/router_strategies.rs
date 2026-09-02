use rand::distr::weighted::WeightedIndex;
use rand::prelude::*;

use crate::config::RoutingStrategy;
use crate::error::AppError;
use crate::state::AppStateInner;

use super::routing_types::ResolvedTarget;

/// 按策略为候选集选出下标（两种候选表示形式的共享实现）。
///
/// - `weights`：每个候选的权重（调用方需已做 `.max(1)` 钳制），仅 Weighted 使用；
/// - `provider_ids`：每个候选的 provider id，LeastBusy / LatencyBased 用它查询运行时状态；
/// - 返回值是 `0..len` 内的下标，由调用方自行取出对应候选。
///
/// 调用方必须保证 `weights.len() == provider_ids.len()` 且非空。
pub(crate) async fn pick_candidate_index(
    strategy: RoutingStrategy,
    weights: &[u32],
    provider_ids: &[&str],
    state: &AppStateInner,
) -> Result<usize, AppError> {
    debug_assert_eq!(weights.len(), provider_ids.len(), "candidate meta mismatch");
    if provider_ids.is_empty() {
        return Err(AppError::NoAvailableBackend("empty candidates".to_string()));
    }
    match strategy {
        RoutingStrategy::Priority => Ok(0),
        RoutingStrategy::RoundRobin => Ok(state.next_round_robin() % provider_ids.len()),
        RoutingStrategy::Weighted => {
            let dist = WeightedIndex::new(weights)
                .map_err(|e| AppError::Config(format!("Invalid weights: {e}")))?;
            Ok(dist.sample(&mut rand::rng()))
        }
        RoutingStrategy::LeastBusy => {
            let mut best_idx = 0;
            let mut best_permits = state.clients.available_permits(provider_ids[0]).await;
            for (i, id) in provider_ids.iter().enumerate().skip(1) {
                let permits = state.clients.available_permits(id).await;
                if permits > best_permits {
                    best_permits = permits;
                    best_idx = i;
                }
            }
            Ok(best_idx)
        }
        RoutingStrategy::LatencyBased => {
            let stats = state.metrics.all_stats().await;
            let mut best_idx = 0;
            let mut best_latency = f64::MAX;

            for (i, pid) in provider_ids.iter().enumerate() {
                let provider_stats: Vec<_> =
                    stats.iter().filter(|s| s.provider_id == **pid).collect();

                let avg_latency = if provider_stats.is_empty() {
                    f64::MAX
                } else {
                    let total_duration: u64 =
                        provider_stats.iter().map(|s| s.total_duration_ms).sum();
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
            Ok(best_idx)
        }
    }
}

pub async fn select_target<'a>(
    strategy: RoutingStrategy,
    candidates: &[ResolvedTarget<'a>],
    state: &'a AppStateInner,
) -> Result<ResolvedTarget<'a>, AppError> {
    if candidates.is_empty() {
        return Err(AppError::NoAvailableBackend("empty candidates".to_string()));
    }
    let weights: Vec<u32> = candidates.iter().map(|t| t.target.weight.max(1)).collect();
    let ids: Vec<&str> = candidates.iter().map(|t| t.provider.id.as_str()).collect();
    let idx = pick_candidate_index(strategy, &weights, &ids, state).await?;
    Ok(candidates[idx].clone())
}
