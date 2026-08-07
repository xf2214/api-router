import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '../../types';

export async function getConfig(): Promise<AppConfig> {
  return invoke<AppConfig>('get_config');
}

export async function exportConfig(): Promise<string> {
  return invoke<string>('export_config');
}

export async function saveConfig(config: AppConfig, keys: Record<string, string>): Promise<void> {
  return invoke('save_config', { config, keys });
}
