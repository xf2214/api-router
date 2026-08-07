use rand::distributions::WeightedIndex;
use rand::prelude::*;

use crate::config::RoutingStrategy;
use crate::error::AppError;
use crate::state::AppStateInner;

use super::routing_types::ResolvedTarget;

pub async fn select_target<'a>(
    strategy: RoutingStrategy,
    candidates: &[ResolvedTarget<'a>],
    state: &'a AppStateInner,
) -> Result<ResolvedTarget<'a>, AppError> {
    if candidates.is_empty() {
        return Err(AppError::NoAvailableBackend("empty candidates".to_string()));
    }
    match strategy {
        RoutingStrategy::Priority => Ok(candidates[0].clone()),
        RoutingStrategy::RoundRobin => {
            let idx = state.next_round_robin() % candidates.len();
            Ok(candidates[idx].clone())
        }
        RoutingStrategy::Weighted => {
            let weights: Vec<u32> = candidates.iter().map(|t| t.target.weight.max(1)).collect();
            let dist = WeightedIndex::new(&weights)
                .map_err(|e| AppError::Config(format!("Invalid weights: {e}")))?;
            let mut rng = thread_rng();
            Ok(candidates[dist.sample(&mut rng)].clone())
        }
        RoutingStrategy::LeastBusy => {
            let mut best_idx = 0;
            let mut best_permits = state
                .clients
                .available_permits(&candidates[0].provider.id)
                .await;
            for (i, candidate) in candidates.iter().enumerate().skip(1) {
                let permits = state
                    .clients
                    .available_permits(&candidate.provider.id)
                    .await;
                if permits > best_permits {
                    best_permits = permits;
                    best_idx = i;
                }
            }
            Ok(candidates[best_idx].clone())
        }
        RoutingStrategy::LatencyBased => {
            let stats = state.metrics.all_stats().await;
            let mut best_idx = 0;
            let mut best_latency = f64::MAX;

            for (i, candidate) in candidates.iter().enumerate() {
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

            Ok(candidates[best_idx].clone())
        }
    }
}
