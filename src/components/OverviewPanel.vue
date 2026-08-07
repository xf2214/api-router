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
          <span class="endpoint">
            <span class="endpoint__url">{{ serverUrl }}</span>
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
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
        </div>
        <div class="stat__label">总请求数</div>
        <div class="stat__value">{{ overallStats.total.toLocaleString() }}</div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--g">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
        </div>
        <div class="stat__label">{{ $t('overviewPanel.successRate') }}</div>
        <div class="stat__value">
          {{ overallStats.total > 0 ? overallStats.successRate.toFixed(1) + '%' : '-' }}
        </div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--o">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
        </div>
        <div class="stat__label">{{ $t('overviewPanel.failureCount') }}</div>
        <div class="stat__value">{{ overallStats.failure.toLocaleString() }}</div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--v">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M12 2 2 7l10 5 10-5-10-5Z"/><path d="m2 17 10 5 10-5"/><path d="m2 12 10 5 10-5"/></svg>
        </div>
        <div class="stat__label">{{ $t('overviewPanel.totalTokens') }}</div>
        <div class="stat__value">{{ formatTokens(overallStats.totalTokens) }}</div>
      </div>
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
            <tr v-if="recentLogs.length === 0">
              <td colspan="6">
                <div class="empty" style="padding: 32px 24px;">
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
import type { AppConfig, ServerStatus, RequestLog } from '../types';
import {
  formatTime,
  formatDuration,
  formatTokens,
  providerName,
  statusBadgeClass,
} from '../utils/format';



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
}

const props = defineProps<Props>();

const serverUrl = computed(() => `http://127.0.0.1:${props.serverStatus.port}/v1`);
</script>

<style scoped>
.overview-panel {
  display: flex;
  flex-direction: column;
}
</style>
