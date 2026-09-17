<template>
  <div class="overview-panel">
    <!-- Service status -->
    <div class="card card--pad" style="margin-bottom: 18px;">
      <div class="row" style="gap: 18px; flex-wrap: wrap;">
        <div class="row" style="gap: 12px;">
          <span class="dot" :class="serverStatus.running ? 'dot--ok' : 'dot--err'" />
          <div>
            <div style="font-size: 15px; font-weight: 700;">
              {{ serverStatus.running ? $t('overviewPanel.serviceRunning') : $t('overviewPanel.serviceStopped') }}
            </div>
            <div class="muted" style="font-size: 12px;">
              {{ serverStatus.running ? `127.0.0.1:${serverStatus.port}` : '本地端点尚未启动' }}
            </div>
          </div>
        </div>
        <div class="cache-divider" />
        <div class="row" style="gap: 10px;">
          <span class="endpoint" style="display: inline-flex; align-items: center; gap: 4px;">
            <span class="endpoint__url">{{ serverUrl }}</span>
            <button type="button" class="btn btn--ghost btn--icon" :title="$t('app.copy')" @click="copyBaseUrl">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            </button>
          </span>
        </div>
        <div class="spacer" />
        <div class="row" style="gap: 8px;">
          <span class="badge" :class="config.local_api_token ? 'badge--ok' : 'badge--mute'">
            {{ config.local_api_token ? 'Token 鉴权已启用' : 'Token 鉴权未启用' }}
          </span>
        </div>
      </div>
    </div>

    <!-- Core stats -->
    <div class="grid-cols cols-4" style="margin-bottom: 18px;">
      <div class="stat">
        <div class="stat__icon">
          <svg v-if="!loadingStats" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="18" height="18" class="spin-anim" style="transform-origin: center;"><path d="M21 12a9 9 0 1 1-6.2-8.55"/></svg>
        </div>
        <div class="stat__label">总请求数</div>
        <div class="stat__value" :style="loadingStats ? 'opacity: 0.45;' : ''">{{ overallStats.total.toLocaleString() }}</div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--g">
          <svg v-if="!loadingStats" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="18" height="18" class="spin-anim" style="transform-origin: center;"><path d="M21 12a9 9 0 1 1-6.2-8.55"/></svg>
        </div>
        <div class="stat__label">{{ $t('overviewPanel.successRate') }}</div>
        <div class="stat__value" :style="loadingStats ? 'opacity: 0.45;' : ''">
          {{ overallStats.total > 0 ? overallStats.successRate.toFixed(1) + '%' : '-' }}
        </div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--o">
          <svg v-if="!loadingStats" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="18" height="18" class="spin-anim" style="transform-origin: center;"><path d="M21 12a9 9 0 1 1-6.2-8.55"/></svg>
        </div>
        <div class="stat__label">{{ $t('overviewPanel.failureCount') }}</div>
        <div class="stat__value" :style="loadingStats ? 'opacity: 0.45;' : ''">{{ overallStats.failure.toLocaleString() }}</div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--v">
          <svg v-if="!loadingStats" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M12 2 2 7l10 5 10-5-10-5Z"/><path d="m2 17 10 5 10-5"/><path d="m2 12 10 5 10-5"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="18" height="18" class="spin-anim" style="transform-origin: center;"><path d="M21 12a9 9 0 1 1-6.2-8.55"/></svg>
        </div>
        <div class="stat__label">{{ $t('overviewPanel.totalTokens') }}</div>
        <div class="stat__value" :style="loadingStats ? 'opacity: 0.45;' : ''">{{ formatTokens(overallStats.totalTokens) }}</div>
      </div>
    </div>

    <div v-if="config.providers.length === 0" class="info-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
        <circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>
      </svg>
      <span>{{ $t('provider.noProvidersYet') }}</span>
      <div class="spacer" />
      <button class="btn btn--soft btn--sm" @click="emit('set-tab', 'providers')">{{ $t('provider.add') }}</button>
    </div>

    <!-- Recent logs -->
    <div class="card card--flush">
      <div class="card__head">
        <div>
          <div class="card__title">最近 5 条请求日志</div>
          <div class="card__sub">实时转发记录摘要</div>
        </div>
      </div>
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr>
              <th>时间</th>
              <th>本地模型</th>
              <th>供应商</th>
              <th>上游模型</th>
              <th>状态</th>
              <th>耗时</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="log in recentLogs" :key="log.request_id">
              <td class="mono">{{ formatTime(log.started_at_ms) }}</td>
              <td>{{ log.local_model }}</td>
              <td>{{ providerName(log.provider_id, config.providers) }}</td>
              <td class="mono">{{ log.upstream_model }}</td>
              <td>
                <span class="badge" :class="statusBadgeClass(log)">
                  {{ log.status ?? (log.success ? 'OK' : 'ERR') }}
                </span>
              </td>
              <td class="mono">{{ formatDuration(log.duration_ms) }}</td>
            </tr>
            <tr v-if="loadingStats">
              <td colspan="6">
                <div class="empty" style="padding: 32px 24px;">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="28" height="28" class="spin-anim" style="transform-origin: center; color: var(--primary);">
                    <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
                  </svg>
                  <div class="empty__desc">{{ $t('logs.refreshing') }}</div>
                </div>
              </td>
            </tr>
            <tr v-else-if="recentLogs.length === 0">
              <td colspan="6">
                <div class="empty" style="padding: 32px 24px;">
                  <div class="empty__icon">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="30" height="30">
                      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M9 13h6"/><path d="M9 17h4"/>
                    </svg>
                  </div>
                  <div class="empty__title">{{ $t('common.noData') }}</div>
                  <div class="empty__desc">{{ $t('overviewPanel.noLogs') }}</div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AppConfig, ServerStatus, RequestLog } from '../types';
import {
  formatTime,
  formatDuration,
  formatTokens,
  providerName,
  statusBadgeClass,
} from '../utils/format';
import { writeTextWithFallback } from '../utils/clipboard';



interface Props {
  serverStatus: ServerStatus;
  config: AppConfig;
  overallStats: {
    total: number;
    success: number;
    failure: number;
    successRate: number;
    totalTokens: number;
  };
  recentLogs: RequestLog[];
  loadingStats?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  loadingStats: false,
});
const emit = defineEmits<{
  'show-message': [text: string, type: 'success' | 'error' | 'warn' | 'info'];
  'set-tab': [tab: string];
}>();
const { t } = useI18n();

const serverUrl = computed(() => `http://127.0.0.1:${props.serverStatus.port}/v1`);

async function copyBaseUrl(): Promise<void> {
  const ok: boolean = await writeTextWithFallback(serverUrl.value);
  emit('show-message', ok ? t('app.copied') : t('app.copyFailed'), ok ? 'success' : 'error');
}
</script>

<style scoped>
.overview-panel {
  display: flex;
  flex-direction: column;
}
</style>
