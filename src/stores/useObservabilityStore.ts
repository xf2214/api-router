import { reactive, toRefs } from 'vue';
import type { RequestLog, ProviderStat } from '../types';
import * as tauri from '../services/tauri';
import { useCsvExport } from '../composables/useCsvExport';

export function createObservabilityStore(
  t: (key: string, args?: Record<string, unknown>) => string,
  showMessageFn: (text: string, type: 'success' | 'error' | 'warn' | 'info') => void,
) {
  const state = reactive<{
    requestLogs: RequestLog[];
    providerStats: ProviderStat[];
    logsLimit: number;
    loadingLogs: boolean;
    loadingStats: boolean;
    statsError: string | null;
    lastError: string | null;
  }>({
    requestLogs: [],
    providerStats: [],
    logsLimit: 100,
    loadingLogs: false,
    loadingStats: false,
    statsError: null,
    lastError: null,
  });

  const { exportLogs: csvExportLogs } = useCsvExport(
    () => state.requestLogs,
    showMessageFn,
    t,
  );

  async function loadRequestLogs(): Promise<void> {
    state.loadingLogs = true;
    state.lastError = null;
    try {
      state.requestLogs = await tauri.getRequestLogs(state.logsLimit);
    } catch (e) {
      console.error('getRequestLogs failed', e);
      state.lastError = String(e);
    } finally {
      state.loadingLogs = false;
    }
  }

  async function loadProviderStats(): Promise<void> {
    state.loadingStats = true;
    state.statsError = null;
    try {
      state.providerStats = await tauri.getRequestStats();
    } catch (e) {
      console.error('getRequestStats failed', e);
      state.statsError = String(e);
    } finally {
      state.loadingStats = false;
    }
  }

  async function clearLogs(): Promise<void> {
    if (!confirm(t('logs.logsClearedConfirm'))) return;
    try {
      await tauri.clearRequestLogs();
      state.providerStats = [];
      state.requestLogs = [];
      showMessageFn(t('logs.logsCleared'), 'success');
    } catch (e) {
      showMessageFn(t('logs.logsClearFailed', { error: String(e) }), 'error');
    }
  }

  function exportLogs() {
    try {
      csvExportLogs();
    } catch (e) {
      showMessageFn(t('logs.exportFailed', { error: String(e) }), 'error');
    }
  }

  return {
    ...toRefs(state),
    loadRequestLogs,
    loadProviderStats,
    clearLogs,
    exportLogs,
  };
}

export type ObservabilityStore = ReturnType<typeof createObservabilityStore>;
