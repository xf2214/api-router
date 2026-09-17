<template>
  <section class="page">
    <header class="ph">
      <div>
        <h1 class="ph__title">{{ $t('logs.title') }}</h1>
        <p class="ph__desc">{{ $t('logs.desc') }}</p>
      </div>
      <div class="ph__actions">
        <div class="seg">
          <button class="seg__btn" :class="{ 'is-active': logQuickStatus === 'all' }" @click="logQuickStatus = 'all'">{{ $t('logs.all') }}</button>
          <button class="seg__btn" :class="{ 'is-active': logQuickStatus === 'success' }" @click="logQuickStatus = 'success'">{{ $t('logs.success') }}</button>
          <button class="seg__btn" :class="{ 'is-active': logQuickStatus === 'fail' }" @click="logQuickStatus = 'fail'">{{ $t('logs.error') }}</button>
        </div>
        <button class="btn btn--ghost btn--sm" :disabled="loadingLogs || loadingStats" @click="emit('refresh-logs')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>
          </svg>
          {{ loadingLogs || loadingStats ? $t('logs.refreshing') : $t('logs.refresh') }}
        </button>
        <button class="btn btn--danger btn--sm" @click="emit('clear-logs')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          {{ $t('logs.clear') }}
        </button>
        <button class="btn btn--ghost btn--sm" @click="emit('export-logs')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          {{ $t('logs.export') }}
        </button>
      </div>
    </header>

    <div v-if="!config.enable_logging" class="info-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
        <circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>
      </svg>
      <span>{{ $t('logs.loggingDisabled') }}</span>
      <div class="spacer" />
      <button class="btn btn--soft btn--sm" @click="emit('set-tab', 'settings')">{{ $t('app.goToSettings') }}</button>
    </div>

    <div v-if="lastError" class="info-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
        <circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>
      </svg>
      <span>{{ lastError }}</span>
      <div class="spacer" />
      <button class="btn btn--soft btn--sm" @click="emit('refresh-logs')">{{ $t('app.refresh') }}</button>
    </div>

    <div class="filter-bar">
      <div class="col-search">
        <svg class="col-search__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="15" height="15">
          <circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/>
        </svg>
        <input v-model="logSearch" class="input" :placeholder="$t('logs.searchPlaceholder')">
      </div>
      <select v-model="logModelFilter" class="select" style="width: 160px">
        <option value="">{{ $t('logs.allModels') }}</option>
        <option v-for="m in allModelNames" :key="m" :value="m">{{ m }}</option>
      </select>
      <select v-model="logStatusFilter" class="select" style="width: 130px">
        <option value="all">{{ $t('logs.allStatus') }}</option>
        <option value="success">{{ $t('logs.success') }}</option>
        <option value="fail">{{ $t('logs.error') }}</option>
        <option value="4xx">4xx</option>
        <option value="5xx">5xx</option>
      </select>
      <select v-model="logStreamFilter" class="select" style="width: 120px">
        <option value="">{{ $t('logs.allStream') }}</option>
        <option value="stream">{{ $t('logs.streamOnly') }}</option>
        <option value="nostream">{{ $t('logs.nonStreamOnly') }}</option>
      </select>
      <select v-model="logTimeFilter" class="select" style="width: 130px">
        <option value="1h">{{ $t('logs.lastHour') }}</option>
        <option value="24h">{{ $t('logs.last24h') }}</option>
        <option value="7d">{{ $t('logs.last7d') }}</option>
      </select>
      <button class="btn btn--soft btn--sm" @click="applyLogFilters">{{ $t('logs.applyFilter') }}</button>
    </div>

    <div class="card card--flush">
      <div class="table-wrap">
        <table class="table log-table" :style="loadingLogs ? 'opacity: .6' : undefined">
          <thead>
            <tr>
              <th>{{ $t('logs.time') }}</th>
              <th>{{ $t('logs.requestId') }}</th>
              <th>{{ $t('logs.localModel') }}</th>
              <th>{{ $t('logs.provider') }}</th>
              <th>{{ $t('logs.upstreamModel') }}</th>
              <th>{{ $t('logs.endpoint') }}</th>
              <th>{{ $t('logs.stream') }}</th>
              <th>{{ $t('logs.status') }}</th>
              <th>{{ $t('logs.duration') }}</th>
              <th>{{ $t('logs.retries') }}</th>
              <th>{{ $t('logs.fallback') }}</th>
              <th>{{ $t('logs.token') }}</th>
              <th>{{ $t('logs.errorCol') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="log in filteredLogs"
              :key="log.request_id"
              class="log-row"
              :class="{ 'is-selected': selectedLog?.request_id === log.request_id, 'op-70': log.status && log.status >= 400 }"
              @click="selectLog(log)"
            >
              <td class="mono">{{ formatTime(log.started_at_ms) }}</td>
              <td class="mono" style="font-size: 11.5px">{{ log.request_id }}</td>
              <td>
                <div class="row" style="gap: 7px">
                  <span
                    class="pmark--sm"
                    :style="{ background: providerColor(log.provider_id), width: '20px', height: '20px', fontSize: '9px' }"
                  >{{ providerGlyph(log.provider_id, config.providers) }}</span>
                  <span class="nowrap" style="font-weight: 600; font-size: 12.5px">{{ log.local_model }}</span>
                </div>
              </td>
              <td><span style="color: var(--text-2); font-size: 12.5px">{{ providerName(log.provider_id, config.providers) }}</span></td>
              <td class="mono" style="font-size: 11.5px">{{ log.upstream_model }}</td>
              <td class="mono" style="font-size: 11px; color: var(--text-3)">{{ log.endpoint }}</td>
              <td>
                <span v-if="log.stream" class="badge badge--info">{{ $t('logs.stream') }}</span>
                <span v-else class="muted" style="font-size: 11.5px">{{ $t('app.no') }}</span>
              </td>
              <td>
                <span class="badge" :class="statusBadgeClass(log)">{{ log.status ?? (log.success ? 'OK' : 'ERR') }}</span>
              </td>
              <td class="mono">{{ formatDuration(log.duration_ms) }}</td>
              <td class="mono">{{ log.retries || '—' }}</td>
              <td style="font-size: 11.5px">{{ log.fell_back ? $t('app.yes') : $t('app.no') }}</td>
              <td class="mono">{{ log.usage ? log.usage.total_tokens.toLocaleString() : '—' }}</td>
              <td class="mono" style="font-size: 11px; color: var(--error)">{{ log.error ?? '—' }}</td>
            </tr>
            <tr v-if="loadingLogs">
              <td colspan="13">
                <div class="empty" style="padding: 40px 24px">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="28" height="28" class="spin-anim" style="transform-origin: center; color: var(--primary);">
                    <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
                  </svg>
                  <div class="empty__desc">{{ $t('logs.refreshing') }}</div>
                </div>
              </td>
            </tr>
            <tr v-else-if="filteredLogs.length === 0">
              <td colspan="13">
                <div class="empty" style="padding: 40px 24px">
                  <div class="empty__icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="30" height="30">
                      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M9 13h6"/><path d="M9 17h4"/>
                    </svg>
                  </div>
                  <div class="empty__title">{{ $t('logs.noMatch') }}</div>
                  <div class="empty__desc">{{ $t('logs.noMatchDesc') }}</div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <div class="row" style="padding: 12px 16px; border-top: 1px solid var(--border)">
        <span class="muted" style="font-size: 12.5px">{{ $t('logs.total', { count: filteredLogs.length }) }}</span>
        <div class="grow" />
        <div class="row" style="gap: 4px">
          <button class="btn btn--ghost btn--sm" style="padding: 0; width: 28px" :aria-label="$t('app.prevPage')" @click="logPage = Math.max(1, logPage - 1)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="m15 18-6-6 6-6"/></svg>
          </button>
          <button v-for="p in logPages" :key="p" class="btn btn--sm" :class="p === logPage ? 'btn--soft' : 'btn--ghost'" style="min-width: 30px; padding: 0 8px" @click="logPage = p">{{ p }}</button>
          <button class="btn btn--ghost btn--sm" style="padding: 0; width: 28px" :aria-label="$t('app.nextPage')" @click="logPage = Math.min(logTotalPages, logPage + 1)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="m9 18 6-6-6-6"/></svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Log detail panel -->
    <div v-if="selectedLog" class="card card--pad log-detail is-open" style="margin-top: 14px">
      <div class="row mb-16">
        <span class="card__title" style="padding: 0">{{ $t('logs.detail') }}</span>
        <span class="mono muted" style="font-size: 11px">#{{ selectedLog.request_id }}</span>
        <div class="grow" />
        <button class="btn btn--ghost btn--sm" @click="copyCurl">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15V5a2 2 0 0 1 2-2h10"/>
          </svg>
          {{ $t('logs.copyCurl') }}
        </button>
        <button class="btn btn--ghost btn--sm" disabled :title="$t('logs.replayNotSupported')" @click="replayLog">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15"><polygon points="6 4 20 12 6 20 6 4"/></svg>
          {{ $t('logs.replay') }}
        </button>
      </div>
      <div class="log-detail__grid">
        <div class="mini-stat"><span class="mini-stat__v mono">{{ selectedLog.local_model }}</span><span class="mini-stat__l">{{ $t('logs.localModelLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v mono">{{ providerName(selectedLog.provider_id, config.providers) }}</span><span class="mini-stat__l">{{ $t('logs.providerLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v mono">{{ selectedLog.upstream_model || '—' }}</span><span class="mini-stat__l">{{ $t('logs.upstreamModelLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v mono">{{ selectedLog.endpoint }}</span><span class="mini-stat__l">{{ $t('logs.endpointLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v mono">{{ formatDuration(selectedLog.duration_ms) }}</span><span class="mini-stat__l">{{ $t('logs.durationLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v mono">{{ selectedLog.usage ? selectedLog.usage.total_tokens.toLocaleString() : '—' }}</span><span class="mini-stat__l">{{ $t('logs.tokenLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v mono">{{ selectedLog.stream ? $t('app.yes') : $t('app.no') }}</span><span class="mini-stat__l">{{ $t('logs.streamLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v mono">{{ selectedLog.retries }}</span><span class="mini-stat__l">{{ $t('logs.retriesLabel') }}</span></div>
        <div class="mini-stat"><span class="mini-stat__v">{{ selectedLog.fell_back ? $t('app.yes') : $t('app.no') }}</span><span class="mini-stat__l">{{ $t('logs.fallbackLabel') }}</span></div>
        <div class="mini-stat">
          <span class="mini-stat__v">
            <span class="badge" :class="statusBadgeClass(selectedLog)">{{ selectedLog.status ?? (selectedLog.success ? '200 OK' : 'ERR') }}</span>
          </span>
          <span class="mini-stat__l">{{ $t('logs.statusLabel') }}</span>
        </div>
      </div>
      <div class="section-title mt-16">{{ $t('logs.errorInfo') }}</div>
      <pre class="log-detail__code">{{ selectedLog.error || $t('logs.noError') }}</pre>
      <div class="section-title mt-16">{{ $t('logs.requestBody') }}</div>
      <pre class="log-detail__code">{{ logRequestBody(selectedLog) }}</pre>
      <div class="section-title mt-16">{{ $t('logs.responseBody') }}</div>
      <pre class="log-detail__code">{{ logResponseBody(selectedLog) }}</pre>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AppConfig, RequestLog } from '../types';
import {
  formatTime,
  formatDuration,
  providerColor,
  providerGlyph,
  providerName,
  statusBadgeClass,
  statusCategory,
} from '../utils/format';

const { t } = useI18n();

interface Props {
  config: AppConfig;
  requestLogs: RequestLog[];
  loadingLogs: boolean;
  loadingStats: boolean;
  lastError?: string | null;
  serverUrl: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'set-tab': [tab: string];
  'refresh-logs': [];
  'clear-logs': [];
  'export-logs': [];
  'show-message': [text: string, type: 'success' | 'error' | 'warn' | 'info'];
}>();

const LOG_PAGE_SIZE = 10;

const logSearch = ref('');
const logModelFilter = ref('');
const logStatusFilter = ref('all');
const logStreamFilter = ref('');
const logTimeFilter = ref('24h');
const logQuickStatus = ref<'all' | 'success' | 'fail'>('all');
const selectedLog = ref<RequestLog | null>(null);
const logPage = ref(1);

const allModelNames = computed(() => {
  const set = new Set<string>();
  for (const m of props.config.models) set.add(m.local_name);
  return Array.from(set).sort();
});

function matchLogStatus(log: RequestLog, mode: string): boolean {
  if (mode === 'all') return true;
  if (mode === 'success') return statusCategory(log) === 'success';
  if (mode === 'fail') return statusCategory(log) !== 'success';
  if (mode === '4xx') return statusCategory(log) === '4xx';
  if (mode === '5xx') return statusCategory(log) === '5xx';
  return true;
}

function matchLogTime(log: RequestLog, mode: string): boolean {
  if (!log.started_at_ms) return true;
  const now = Date.now();
  const delta = now - log.started_at_ms;
  if (mode === '1h') return delta <= 3600_000;
  if (mode === '24h') return delta <= 86400_000;
  if (mode === '7d') return delta <= 604800_000;
  return true;
}

const filteredLogsAll = computed(() => {
  const q = logSearch.value.trim().toLowerCase();
  const mode = logStatusFilter.value !== 'all' ? logStatusFilter.value : logQuickStatus.value;
  return props.requestLogs.filter((log) => {
    if (q) {
      const s = `${log.local_model} ${log.request_id} ${log.provider_id} ${log.upstream_model}`.toLowerCase();
      if (!s.includes(q)) return false;
    }
    if (logModelFilter.value && log.local_model !== logModelFilter.value) return false;
    if (!matchLogStatus(log, mode)) return false;
    if (logStreamFilter.value === 'stream' && !log.stream) return false;
    if (logStreamFilter.value === 'nostream' && log.stream) return false;
    if (!matchLogTime(log, logTimeFilter.value)) return false;
    return true;
  });
});

const logTotalPages = computed(() => Math.max(1, Math.ceil(filteredLogsAll.value.length / LOG_PAGE_SIZE)));
const logPages = computed(() => {
  const total = logTotalPages.value;
  const pages: number[] = [];
  for (let i = 1; i <= Math.min(total, 5); i++) pages.push(i);
  return pages;
});

const filteredLogs = computed(() => {
  const start = (logPage.value - 1) * LOG_PAGE_SIZE;
  return filteredLogsAll.value.slice(start, start + LOG_PAGE_SIZE);
});

watch([logSearch, logModelFilter, logStatusFilter, logStreamFilter, logTimeFilter, logQuickStatus], () => {
  logPage.value = 1;
});

function applyLogFilters(): void {
  logPage.value = 1;
}

function selectLog(log: RequestLog): void {
  selectedLog.value = log;
}

function logRequestBody(log: RequestLog): string {
  if (log.endpoint === '/v1/embeddings') {
    return `POST ${log.endpoint}\n{\n  "model": "${log.local_model}",\n  "input": "hello world"\n}`;
  }
  return `POST ${log.endpoint}\n{\n  "model": "${log.local_model}",\n  "messages": [{ "role": "user", "content": "..." }],\n  "temperature": 0.7,\n  "stream": ${log.stream}\n}`;
}

function logResponseBody(log: RequestLog): string {
  if (!log.success) return log.error ?? t('logs.upstreamError');
  return '{\n  "id": "' + log.request_id + '",\n  "object": "chat.completion",\n  "model": "' + log.upstream_model + '",\n  "usage": {\n    "prompt_tokens": ' + (log.usage?.prompt_tokens ?? 0) + ',\n    "completion_tokens": ' + (log.usage?.completion_tokens ?? 0) + ',\n    "total_tokens": ' + (log.usage?.total_tokens ?? 0) + '\n  }\n}';
}

function copyCurl(): void {
  const model = selectedLog.value?.local_model || 'gpt-4o';
  const curl = `curl ${props.serverUrl}/v1/chat/completions \\\n  -H "Content-Type: application/json" \\\n  -d '{"model":"${model}","messages":[{"role":"user","content":"你好"}]}'`;
  navigator.clipboard?.writeText(curl).catch(() => {});
  emit('show-message', t('logs.curlCopied'), 'success');
}

function replayLog(): void {
  emit('show-message', t('logs.replayNotSupported'), 'warn');
}
</script>
