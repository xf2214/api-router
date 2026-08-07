import { watch, type Ref, onBeforeUnmount } from 'vue';

export function useAutoRefresh(
  activeTab: Ref<string>,
  loadRequestLogs: () => Promise<void>,
  loadProviderStats: () => Promise<void>,
) {
  let logsTimer: ReturnType<typeof setInterval> | null = null;

  function stopLogsAutoRefresh() {
    if (logsTimer) {
      clearInterval(logsTimer);
      logsTimer = null;
    }
  }

  function startLogsAutoRefresh() {
    stopLogsAutoRefresh();
    logsTimer = setInterval(() => {
      loadRequestLogs();
      loadProviderStats();
    }, 5000);
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

  onBeforeUnmount(() => {
    stopLogsAutoRefresh();
    stopWatch();
  });

  return { startLogsAutoRefresh, stopLogsAutoRefresh };
}
