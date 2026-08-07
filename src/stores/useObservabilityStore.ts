import { reactive, toRefs } from 'vue';
import type { RequestLog, ProviderStat } from '../types';
import * as tauri from '../services/tauri';
import { useCsvExport } from '../composables/useCsvExport';

export function createObservabilityStore(
  t: (key: string, args?: any) => string,
  showMessageFn: (text: string, type: any) => void,
) {
  const state = reactive<{
    requestLogs: RequestLog[];
    providerStats: ProviderStat[];
    logsLimit: number;
    loadingLogs: boolean;
    loadingStats: boolean;
  }>({
    requestLogs: [],
    providerStats: [],
    logsLimit: 100,
    loadingLogs: false,
    loadingStats: false,
  });

  const { exportLogs: csvExportLogs } = useCsvExport(
    () => state.requestLogs,
    showMessageFn,
    t,
  );

  async function loadRequestLogs(): Promise<void> {
    state.loadingLogs = true;
    try {
      state.requestLogs = await tauri.getRequestLogs(state.logsLimit);
    } catch (e) {
      console.error('getRequestLogs failed', e);
    } finally {
      state.loadingLogs = false;
    }
  }

  async function loadProviderStats(): Promise<void> {
    state.loadingStats = true;
    try {
      state.providerStats = await tauri.getRequestStats();
    } catch (e) {
      console.error('getRequestStats failed', e);
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
