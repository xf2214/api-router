import { watch, type Ref, onBeforeUnmount, onScopeDispose } from 'vue';

export interface UseAutoRefreshOptions {
  intervalMs?: number;
}

export function useAutoRefresh(
  activeTab: Ref<string>,
  loadRequestLogs: () => Promise<void>,
  loadProviderStats: () => Promise<void>,
  options: UseAutoRefreshOptions = {},
) {
  const intervalMs = options.intervalMs ?? 5000;
  let logsTimer: ReturnType<typeof setInterval> | null = null;

  function stopLogsAutoRefresh() {
    if (logsTimer) {
      clearInterval(logsTimer);
      logsTimer = null;
    }
  }

  function startLogsAutoRefresh() {
    if (typeof document !== 'undefined' && document.hidden) return;
    stopLogsAutoRefresh();
    logsTimer = setInterval(() => {
      loadRequestLogs();
      loadProviderStats();
    }, intervalMs);
  }

  function handleVisibilityChange() {
    if (document.hidden) {
      stopLogsAutoRefresh();
    } else if (activeTab.value === 'logs') {
      startLogsAutoRefresh();
    }
  }

  if (typeof document !== 'undefined') {
    document.addEventListener('visibilitychange', handleVisibilityChange);
  }

  const stopWatch = watch(activeTab, async (tab) => {
    if (tab === 'logs' || tab === 'stats') {
      await loadRequestLogs();
      await loadProviderStats();
      if (tab === 'logs') startLogsAutoRefresh();
    } else {
      stopLogsAutoRefresh();
    }
  });

  function cleanup() {
    stopLogsAutoRefresh();
    stopWatch();
    if (typeof document !== 'undefined') {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    }
  }

  onBeforeUnmount(cleanup);
  try {
    onScopeDispose(cleanup);
  } catch {
    /* ignore if called outside effect scope */
  }

  return { startLogsAutoRefresh, stopLogsAutoRefresh };
}
