import { reactive, computed, toRefs } from 'vue';
import type { AppConfig } from '../types';
import * as tauri from '../services/tauri';

export function createAppStore(
  t: (key: string, args?: Record<string, unknown>) => string,
  showMessageFn: (text: string, type: 'success' | 'error' | 'warn' | 'info') => void,
) {
  const state = reactive<{
    config: AppConfig;
    serverStatus: { running: boolean; port: number };
    pendingKeys: Record<string, string>;
  }>({
    config: {
      port: 6123,
      local_api_token: null,
      enable_logging: false,
      providers: [],
      models: [],
      model_definitions: [],
      groups: [],
      fallback: {
        default_retries: 2,
        timeout_seconds: 60,
        cb_config: {
          failure_threshold: 5,
          failure_threshold_percentage: null,
          cooldown_interval_ms: 60000,
          failure_status_codes: null,
          minimum_requests: 10,
        },
      },
      enable_auto_health_check: true,
      health_check_interval_seconds: 300,
      cache: {
        enabled: false,
        mode: 'simple',
        max_age_seconds: 3600,
        max_entries: 1000,
      },
    },
    serverStatus: { running: false, port: 6123 },
    pendingKeys: {},
  });

  const allGroups = computed(() => {
    const set = new Set<string>();
    // 默认分组始终作为筛选选项可选项存在（不参与路由树分组实体）
    set.add('默认');
    for (const g of state.config.groups) {
      if (g.name && g.name !== '默认') set.add(g.name);
    }
    return Array.from(set);
  });

  const serverUrl = computed(() => `http://127.0.0.1:${state.serverStatus.port}`);

  async function loadConfig(): Promise<void> {
    try {
      const c = await tauri.getConfig();
      state.config = {
        ...state.config,
        ...c,
        providers: c.providers ?? [],
        models: c.models ?? [],
        groups: c.groups ?? [],
        fallback: c.fallback ?? state.config.fallback,
        cache: c.cache ?? state.config.cache,
      };
    } catch (e) {
      showMessageFn(t('common.loadFailed', { error: String(e) }), 'error');
    }
  }

  async function persistConfig(): Promise<void> {
    try {
      await tauri.saveConfig(state.config, state.pendingKeys);
      state.pendingKeys = {};
    } catch (e) {
      showMessageFn(t('common.saveFailed', { error: String(e) }), 'error');
    }
  }

  async function refreshStatus(): Promise<void> {
    try {
      state.serverStatus = await tauri.getServerStatus();
    } catch (e) {
      console.error('getServerStatus failed', e);
    }
  }

  async function toggleServer(): Promise<void> {
    try {
      state.serverStatus = state.serverStatus.running
        ? await tauri.stopServer()
        : await tauri.startServer();
    } catch (e) {
      showMessageFn(`${t('app.startService')}${t('common.saveFailed', { error: String(e) })}`, 'error');
    }
  }

  async function saveSettings(newConfig: AppConfig): Promise<void> {
    state.config = newConfig;
    await persistConfig();
    showMessageFn(t('settings.saved'), 'success');
  }

  return {
    ...toRefs(state),
    allGroups,
    serverUrl,
    loadConfig,
    persistConfig,
    refreshStatus,
    toggleServer,
    saveSettings,
  };
}

export type AppStore = ReturnType<typeof createAppStore>;
