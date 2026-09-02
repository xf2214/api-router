import type { AppConfig } from '../../types';
import { invokeTyped, COMMANDS } from './client';

export async function getConfig(): Promise<AppConfig> {
  return invokeTyped<AppConfig>(COMMANDS.config.getConfig);
}

export async function exportConfig(): Promise<string> {
  return invokeTyped<string>(COMMANDS.config.exportConfig, { includeKeys: true });
}

export async function saveConfig(config: AppConfig, keys: Record<string, string>): Promise<void> {
  return invokeTyped<void>(COMMANDS.config.saveConfig, { config, keys } as Record<string, unknown>);
}
