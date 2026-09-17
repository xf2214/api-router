import type { AppConfig } from '../../types';
import { invokeTyped, COMMANDS } from './client';

export async function getConfig(): Promise<AppConfig> {
  return invokeTyped<AppConfig>(COMMANDS.config.getConfig);
}

export async function exportConfig(includeKeys = true): Promise<string> {
  return invokeTyped<string>(COMMANDS.config.exportConfig, { includeKeys } as Record<string, unknown>);
}

export async function importConfig(payload: string): Promise<{ providers: number; models: number; backup_path: string }> {
  return invokeTyped(COMMANDS.config.importConfig, { payload } as Record<string, unknown>);
}

export async function saveConfig(config: AppConfig, keys: Record<string, string>): Promise<void> {
  return invokeTyped<void>(COMMANDS.config.saveConfig, { config, keys } as Record<string, unknown>);
}
