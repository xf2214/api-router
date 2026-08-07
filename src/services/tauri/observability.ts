import { invoke } from '@tauri-apps/api/core';
import type { RequestLog, ProviderStat } from '../../types';

export async function getRequestLogs(limit?: number): Promise<RequestLog[]> {
  return invoke<RequestLog[]>('get_request_logs', { limit: limit ?? null });
}

export async function getRequestStats(): Promise<ProviderStat[]> {
  return invoke<ProviderStat[]>('get_request_stats');
}

export async function clearRequestLogs(): Promise<void> {
  return invoke('clear_request_logs');
}
