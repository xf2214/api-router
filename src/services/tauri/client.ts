import { invoke } from '@tauri-apps/api/core';

/**
 * Typed invoke wrapper that normalizes Tauri IPC errors to Error with readable message.
 */
export async function invokeTyped<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (raw) {
    throw normalizeInvokeError(raw);
  }
}

function normalizeInvokeError(raw: unknown): Error {
  if (raw instanceof Error) return raw;
  if (typeof raw === 'string') return new Error(raw);
  if (raw != null && typeof raw === 'object') {
    const obj = raw as Record<string, unknown>;
    if (typeof obj.message === 'string') return new Error(obj.message);
    if (typeof obj.error === 'string') return new Error(obj.error);
    // Tauri sometimes returns { code, message } or plain object
    try {
      return new Error(JSON.stringify(raw));
    } catch {
      return new Error(String(raw));
    }
  }
  return new Error(String(raw));
}

export const COMMANDS = {
  config: {
    getConfig: 'get_config',
    exportConfig: 'export_config',
    saveConfig: 'save_config',
  },
  provider: {
    deleteProvider: 'delete_provider',
    checkProviderHealth: 'check_provider_health',
    checkAllProvidersHealth: 'check_all_providers_health',
    getHealthStatus: 'get_health_status',
    fetchProviderModels: 'fetch_provider_models',
    testProviderTarget: 'test_provider_target',
  },
  models: {
    getGroups: 'get_groups',
    saveGroup: 'save_group',
    deleteGroup: 'delete_group',
    getModelDefinitions: 'get_model_definitions',
    saveModelDefinition: 'save_model_definition',
    deleteModelDefinition: 'delete_model_definition',
  },
  server: {
    startServer: 'start_server',
    stopServer: 'stop_server',
    getServerStatus: 'get_server_status',
    testModelConnection: 'test_model_connection',
    testModelConfig: 'test_model_config',
  },
  observability: {
    getRequestLogs: 'get_request_logs',
    getRequestStats: 'get_request_stats',
    clearRequestLogs: 'clear_request_logs',
  },
} as const;
