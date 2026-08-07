import { invoke } from '@tauri-apps/api/core';
import type { ProviderHealth, ModelTestTargetResult, ProviderConfig } from '../../types';

export async function deleteProvider(providerId: string): Promise<void> {
  return invoke('delete_provider', { providerId });
}

export async function checkProviderHealth(providerId: string): Promise<ProviderHealth> {
  return invoke<ProviderHealth>('check_provider_health', { providerId });
}

export async function checkAllProvidersHealth(): Promise<ProviderHealth[]> {
  return invoke<ProviderHealth[]>('check_all_providers_health');
}

export async function getHealthStatus(): Promise<ProviderHealth[]> {
  return invoke<ProviderHealth[]>('get_health_status');
}

export interface ProviderModelsResult {
  models: string[];
  raw_json: string;
  info: Record<string, { context_length?: number; max_tokens?: number }>;
}

export async function fetchProviderModels(provider: ProviderConfig, apiKey: string): Promise<ProviderModelsResult> {
  return invoke<ProviderModelsResult>('fetch_provider_models', { provider, apiKey });
}

export async function testProviderTarget(
  provider: ProviderConfig,
  apiKey: string,
  modelName: string,
): Promise<ModelTestTargetResult> {
  return invoke<ModelTestTargetResult>('test_provider_target', { provider, apiKey, modelName });
}
