<template>
  <section class="page">
    <header class="ph">
      <div>
        <h1 class="ph__title">{{ $t('overview.title') }}</h1>
        <p class="ph__desc">{{ $t('overview.desc') }}</p>
      </div>
      <div class="ph__actions">
        <button class="btn btn--ghost btn--sm" @click="emit('set-tab', 'settings')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <circle cx="12" cy="12" r="3"/><path d="M12 1v6m0 10v6M4.22 4.22l4.24 4.24m7.08 7.08 4.24 4.24M1 12h6m10 0h6M4.22 19.78l4.24-4.24m7.08-7.08 4.24-4.24"/>
          </svg>
          {{ $t('overview.serviceSettings') }}
        </button>
      </div>
    </header>

    <OverviewPanel
      :server-status="serverStatus"
      :config="config"
      :overall-stats="overallStats"
      :recent-logs="recentLogs"
    />

    <!-- Quick actions -->
    <div class="section-title" style="margin: 24px 0 12px;">{{ $t('overview.quickActions') }}</div>
    <div class="quick-grid">
      <button class="quick-tile" @click="emit('check-all')">
        <span class="quick-tile__icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
        </span>
        <span class="quick-tile__t">{{ $t('overview.checkAllProviders') }}</span>
        <span class="quick-tile__d">{{ $t('overview.checkAllProvidersDesc') }}</span>
      </button>
      <button class="quick-tile" @click="emit('set-tab', 'stats')">
        <span class="quick-tile__icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M3 3v18h18"/><path d="m19 9-5 5-4-4-3 3"/></svg>
        </span>
        <span class="quick-tile__t">{{ $t('overview.openStats') }}</span>
        <span class="quick-tile__d">{{ $t('overview.openStatsDesc') }}</span>
      </button>
      <button class="quick-tile" @click="emit('set-tab', 'logs')">
        <span class="quick-tile__icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M9 13h6"/><path d="M9 17h4"/></svg>
        </span>
        <span class="quick-tile__t">{{ $t('overview.openLogs') }}</span>
        <span class="quick-tile__d">{{ $t('overview.openLogsDesc') }}</span>
      </button>
      <button class="quick-tile" @click="emit('toggle-server')">
        <span class="quick-tile__icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><rect x="2" y="6" width="20" height="12" rx="2"/><path d="M6 12h4"/><path d="M15 12h.01"/><path d="M18 12h.01"/></svg>
        </span>
        <span class="quick-tile__t">{{ serverStatus.running ? $t('app.stopService') : $t('app.startService') }}</span>
        <span class="quick-tile__d">{{ $t('overview.toggleService') }}</span>
      </button>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { AppConfig, ServerStatus, ProviderHealth, RequestLog } from '../types';
import OverviewPanel from '../components/OverviewPanel.vue';

interface Props {
  config: AppConfig;
  serverStatus: ServerStatus;
  healthStatus: ProviderHealth[];
  requestLogs: RequestLog[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'set-tab': [tab: string];
  'check-all': [];
  'toggle-server': [];
}>();

const overallStats = computed(() => {
  let total = 0;
  let success = 0;
  let failure = 0;
  let totalTokens = 0;
  for (const log of props.requestLogs) {
    total++;
    if (log.success) success++;
    else failure++;
    totalTokens += log.usage?.total_tokens ?? 0;
  }
  const successRate = total > 0 ? (success / total) * 100 : 0;
  return { total, success, failure, successRate, totalTokens };
});

const recentLogs = computed(() => {
  return [...props.requestLogs].sort((a, b) => b.started_at_ms - a.started_at_ms).slice(0, 5);
});
</script>
