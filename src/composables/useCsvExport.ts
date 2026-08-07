import type { RequestLog } from '../types';

export function useCsvExport(
  getLogs: () => RequestLog[],
  showMessage: (text: string, type: any) => void,
  t: (key: string, args?: any) => string,
) {
  function logsToCsv(logs: RequestLog[]): string {
    const headers = [
      'request_id',
      'local_model',
      'provider_id',
      'upstream_model',
      'endpoint',
      'stream',
      'started_at_ms',
      'duration_ms',
      'status',
      'success',
      'error',
      'retries',
      'fell_back',
      'prompt_tokens',
      'completion_tokens',
      'total_tokens',
    ];
    const lines = logs.map((log) => {
      const usage = log.usage ?? { prompt_tokens: 0, completion_tokens: 0, total_tokens: 0 };
      const values = [
        log.request_id,
        log.local_model,
        log.provider_id,
        log.upstream_model,
        log.endpoint,
        log.stream,
        log.started_at_ms,
        log.duration_ms,
        log.status ?? '',
        log.success,
        log.error ?? '',
        log.retries,
        log.fell_back,
        usage.prompt_tokens,
        usage.completion_tokens,
        usage.total_tokens,
      ];
      return values.map((v) => `"${String(v).replace(/"/g, '""')}"`).join(',');
    });
    return [headers.join(','), ...lines].join('\n');
  }

  function downloadBlob(blob: Blob, filename: string) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  function exportLogs() {
    const rows = getLogs();
    if (rows.length === 0) {
      showMessage(t('logs.noExportData'), 'warn');
      return;
    }
    const csv = logsToCsv(rows);
    const blob = new Blob(['\uFEFF' + csv], { type: 'text/csv;charset=utf-8;' });
    const filename = `logs_${new Date().toISOString().slice(0, 19).replace(/:/g, '-')}.csv`;
    downloadBlob(blob, filename);
    showMessage(t('logs.exported'), 'success');
  }

  return { logsToCsv, downloadBlob, exportLogs };
}
