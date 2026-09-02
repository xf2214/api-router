import type { RequestLog, ProviderStat } from '../../types';
import { invokeTyped, COMMANDS } from './client';

export async function getRequestLogs(limit?: number): Promise<RequestLog[]> {
  return invokeTyped<RequestLog[]>(COMMANDS.observability.getRequestLogs, { limit: limit ?? null } as Record<string, unknown>);
}

export async function getRequestStats(): Promise<ProviderStat[]> {
  return invokeTyped<ProviderStat[]>(COMMANDS.observability.getRequestStats);
}

export async function clearRequestLogs(): Promise<void> {
  return invokeTyped<void>(COMMANDS.observability.clearRequestLogs);
}
