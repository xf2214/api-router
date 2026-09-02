import type { ProviderHealth, ModelTestTargetResult, ProviderConfig } from '../../types';
import { invokeTyped, COMMANDS } from './client';

export async function deleteProvider(providerId: string): Promise<void> {
  return invokeTyped<void>(COMMANDS.provider.deleteProvider, { providerId } as Record<string, unknown>);
}

export async function checkProviderHealth(providerId: string): Promise<ProviderHealth> {
  return invokeTyped<ProviderHealth>(COMMANDS.provider.checkProviderHealth, { providerId } as Record<string, unknown>);
}

export async function checkAllProvidersHealth(): Promise<ProviderHealth[]> {
  return invokeTyped<ProviderHealth[]>(COMMANDS.provider.checkAllProvidersHealth);
}

export async function getHealthStatus(): Promise<ProviderHealth[]> {
  return invokeTyped<ProviderHealth[]>(COMMANDS.provider.getHealthStatus);
}

export interface ProviderModelsResult {
  models: string[];
  raw_json: string;
  info: Record<string, { context_length?: number; max_tokens?: number }>;
}

export async function fetchProviderModels(provider: ProviderConfig, apiKey: string): Promise<ProviderModelsResult> {
  return invokeTyped<ProviderModelsResult>(COMMANDS.provider.fetchProviderModels, { provider, apiKey } as Record<string, unknown>);
}

export async function testProviderTarget(
  provider: ProviderConfig,
  apiKey: string,
  modelName: string,
): Promise<ModelTestTargetResult> {
  return invokeTyped<ModelTestTargetResult>(COMMANDS.provider.testProviderTarget, { provider, apiKey, modelName } as Record<string, unknown>);
}
