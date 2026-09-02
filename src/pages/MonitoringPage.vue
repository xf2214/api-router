<template>
  <section class="page">
    <header class="ph">
      <div>
        <h1 class="ph__title">{{ $t('monitoring.title') }}</h1>
        <p class="ph__desc">{{ $t('monitoring.desc') }}</p>
      </div>
      <div class="ph__actions">
        <div class="seg">
          <button class="seg__btn" :class="{ 'is-active': monitorRange === 'today' }" @click="monitorRange = 'today'">今日</button>
          <button class="seg__btn" :class="{ 'is-active': monitorRange === 'week' }" @click="monitorRange = 'week'">本周</button>
          <button class="seg__btn" :class="{ 'is-active': monitorRange === 'month' }" @click="monitorRange = 'month'">本月</button>
        </div>
        <select v-model="monitorProviderFilter" class="select" style="width: 140px">
          <option value="">{{ $t('monitoring.allProviders') }}</option>
          <option v-for="p in config.providers" :key="p.id" :value="p.id">{{ p.name }}</option>
        </select>
        <button class="btn btn--ghost btn--sm" @click="emit('show-message', t('monitoring.exported'), 'success')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          {{ $t('monitoring.exportCsv') }}
        </button>
      </div>
    </header>

    <div class="grid-cols cols-4" style="margin-bottom: 18px">
      <div class="stat">
        <div class="stat__icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M22 12h-4l-3 9L9 3l-3 9H2"/></svg>
        </div>
        <div class="stat__label">{{ $t('monitoring.totalRequests') }}</div>
        <div class="stat__value">{{ overallStats.total.toLocaleString() }}</div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--v">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M12 2 2 7l10 5 10-5-10-5Z"/><path d="m2 17 10 5 10-5"/><path d="m2 12 10 5 10-5"/></svg>
        </div>
        <div class="stat__label">{{ $t('monitoring.totalTokens') }}</div>
        <div class="stat__value">{{ formatTokens(overallStats.totalTokens) }}</div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--g">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
        </div>
        <div class="stat__label">{{ $t('monitoring.avgLatency') }}</div>
        <div class="stat__value">{{ formatDuration(overallStats.avgLatency) }}</div>
      </div>
      <div class="stat">
        <div class="stat__icon stat__icon--o">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
        </div>
        <div class="stat__label">{{ $t('monitoring.errorRate') }}</div>
        <div class="stat__value">{{ overallStats.total > 0 ? ((overallStats.failure / overallStats.total) * 100).toFixed(1) : '0.0' }}<span class="unit">%</span></div>
      </div>
    </div>

    <!-- Request trend chart -->
    <div class="card card--flush" style="margin-bottom: 18px">
      <div class="card__head">
        <div>
          <div class="card__title">{{ $t('monitoring.requestTrend') }}</div>
          <div class="card__sub">{{ $t('monitoring.trendDesc', { range: monitorRangeLabel }) }}</div>
        </div>
        <div class="spacer" />
        <div class="row" style="gap: 12px; flex-wrap: wrap">
          <span v-for="(p, i) in config.providers.slice(0, 4)" :key="p.id" class="row legend-item" style="gap: 6px">
            <span class="dot" :style="{ background: CHART_COLORS[i % CHART_COLORS.length] }" />
            <span class="muted" style="font-size: 11.5px">{{ p.name }}</span>
          </span>
        </div>
      </div>
      <div style="padding: 10px 16px 14px">
        <svg viewBox="0 0 800 280" width="100%" style="display: block" preserveAspectRatio="none" aria-hidden="true">
          <defs>
            <linearGradient id="monArea" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="var(--c2)" stop-opacity="0.14"/>
              <stop offset="100%" stop-color="var(--c2)" stop-opacity="0"/>
            </linearGradient>
          </defs>
          <line class="gridline" x1="48" y1="30" x2="780" y2="30"/>
          <line class="gridline" x1="48" y1="90" x2="780" y2="90"/>
          <line class="gridline" x1="48" y1="150" x2="780" y2="150"/>
          <line class="gridline" x1="48" y1="210" x2="780" y2="210"/>
          <text class="axis-y" x="40" y="34" text-anchor="end">{{ maxBucketCount }}</text>
          <text class="axis-y" x="40" y="94" text-anchor="end">{{ Math.round(maxBucketCount * 0.75) }}</text>
          <text class="axis-y" x="40" y="154" text-anchor="end">{{ Math.round(maxBucketCount * 0.5) }}</text>
          <text class="axis-y" x="40" y="214" text-anchor="end">{{ Math.round(maxBucketCount * 0.25) }}</text>
          <text class="axis-y" x="40" y="244" text-anchor="end">0</text>
          <path fill="url(#monArea)" :d="trendAreaPath" />
          <path fill="none" stroke="var(--c2)" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" :d="trendLinePath" />
          <text
            v-for="(b, i) in trendBuckets"
            :key="i"
            class="axis-x"
            :x="trendPoints[i] ? trendPoints[i].x : 48"
            y="262"
            text-anchor="middle"
          >{{ b.label }}</text>
        </svg>
      </div>
    </div>

    <!-- Token distribution + latency distribution -->
    <div class="grid-cols cols-2" style="margin-bottom: 18px">
      <div class="card card--pad">
        <div class="row mb-8">
          <span class="card__title" style="padding: 0">{{ $t('monitoring.tokenDistribution') }}</span>
          <div class="spacer" />
          <span class="card__sub">{{ formatTokens(overallStats.totalTokens) }}</span>
        </div>
        <div style="display: flex; justify-content: center; margin: 4px 0 12px">
          <svg viewBox="0 0 200 200" width="160" height="160" aria-hidden="true">
            <circle cx="100" cy="100" r="70" fill="none" stroke="var(--surface-3)" stroke-width="26"/>
            <circle
              v-for="(slice, i) in tokenDonutSlices"
              :key="i"
              cx="100"
              cy="100"
              r="70"
              fill="none"
              :stroke="slice.color"
              stroke-width="26"
              :stroke-dasharray="slice.dash"
              :transform="slice.rotate"
            />
            <text x="100" y="95" text-anchor="middle" dominant-baseline="central" class="donut-center" style="font-size: 22px; font-weight: 700; letter-spacing: -0.02em">{{ formatTokensCompact(overallStats.totalTokens) }}</text>
            <text x="100" y="118" text-anchor="middle" dominant-baseline="central" style="font-size: 11px; fill: var(--text-3); font-weight: 600">{{ $t('monitoring.totalToken') }}</text>
          </svg>
        </div>
        <div style="display: flex; flex-direction: column; gap: 9px">
          <div v-for="(item, i) in tokenDistribution" :key="i" class="row" style="justify-content: space-between">
            <div class="row" style="gap: 9px">
              <span class="dot" :style="{ background: item.color }" />
              <span style="font-size: 12.5px; font-weight: 600">{{ item.label }}</span>
            </div>
            <div class="row" style="gap: 12px">
              <span class="mono muted" style="font-size: 11.5px">{{ formatTokens(item.tokens) }}</span>
              <span class="mono" style="font-size: 11.5px; color: var(--text-2); min-width: 36px; text-align: right">{{ item.percent }}%</span>
            </div>
          </div>
          <div v-if="tokenDistribution.length === 0" class="empty" style="padding: 20px">
            <div class="empty__desc">{{ $t('monitoring.noTokenData') }}</div>
          </div>
        </div>
      </div>

      <div class="card card--pad">
        <div class="row mb-8">
          <span class="card__title" style="padding: 0">{{ $t('monitoring.latencyDistribution') }}</span>
          <div class="spacer" />
          <span class="badge badge--info">{{ $t('monitoring.thisWeek') }}</span>
        </div>
        <div class="mono" style="font-size: 12px; color: var(--text-2); background: var(--surface-2); border: 1px solid var(--border); border-radius: var(--r-md); padding: 9px 12px; margin-bottom: 16px">
          <span v-html="latencyPercentilesHtml"></span>
        </div>
        <div style="display: flex; flex-direction: column; gap: 12px">
          <div v-for="(b, i) in latencyBuckets" :key="i" class="field" style="gap: 6px">
            <div class="row">
              <span class="field__label">{{ b.label }}</span>
              <div class="grow" />
              <span class="mono" style="font-size: 12px; font-weight: 700">{{ b.percent }}%</span>
            </div>
            <div class="bar"><div class="bar__fill" :style="{ width: b.percent + '%', background: b.color }" /></div>
          </div>
        </div>
      </div>
    </div>

    <!-- Aggregated stats table -->
    <div class="card card--flush" style="margin-bottom: 18px">
      <div class="card__head">
        <div>
          <div class="card__title">{{ $t('monitoring.aggregatedStats') }}</div>
          <div class="card__sub">{{ $t('monitoring.aggregatedStatsDesc') }}</div>
        </div>
        <div class="spacer" />
        <button class="btn btn--danger btn--sm" @click="emit('clear-logs')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
          {{ $t('monitoring.clearStats') }}
        </button>
      </div>
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr>
              <th data-sort="provider" :data-sort-dir="sortKey === 'provider' ? sortDir : undefined" @click="setSort('provider')">{{ $t('monitoring.provider') }}</th>
              <th data-sort="model" :data-sort-dir="sortKey === 'model' ? sortDir : undefined" @click="setSort('model')">{{ $t('monitoring.upstreamModel') }}</th>
              <th data-sort="total_requests" :data-sort-dir="sortKey === 'total_requests' ? sortDir : undefined" @click="setSort('total_requests')">{{ $t('monitoring.requests') }}</th>
              <th data-sort="success_count" :data-sort-dir="sortKey === 'success_count' ? sortDir : undefined" @click="setSort('success_count')">{{ $t('monitoring.success') }}</th>
              <th data-sort="failure_count" :data-sort-dir="sortKey === 'failure_count' ? sortDir : undefined" @click="setSort('failure_count')">{{ $t('monitoring.failure') }}</th>
              <th data-sort="success_rate" :data-sort-dir="sortKey === 'success_rate' ? sortDir : undefined" @click="setSort('success_rate')">{{ $t('monitoring.successRate') }}</th>
              <th data-sort="avg_latency" :data-sort-dir="sortKey === 'avg_latency' ? sortDir : undefined" @click="setSort('avg_latency')">{{ $t('monitoring.avgLatencyCol') }}</th>
              <th data-sort="total_retries" :data-sort-dir="sortKey === 'total_retries' ? sortDir : undefined" @click="setSort('total_retries')">{{ $t('monitoring.retries') }}</th>
              <th data-sort="total_tokens" :data-sort-dir="sortKey === 'total_tokens' ? sortDir : undefined" @click="setSort('total_tokens')">{{ $t('monitoring.token') }}</th>
              <th data-sort="last_request_at_ms" :data-sort-dir="sortKey === 'last_request_at_ms' ? sortDir : undefined" @click="setSort('last_request_at_ms')">{{ $t('monitoring.lastRequest') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(s, i) in sortedProviderStats" :key="i">
              <td>
                <div class="row" style="gap: 8px">
                  <span class="pmark--sm" :style="{ background: providerColor(s.provider_id), width: '22px', height: '22px', fontSize: '10px' }">{{ providerGlyph(s.provider_id, config.providers) }}</span>
                  <span style="font-weight: 600">{{ providerName(s.provider_id, config.providers) }}</span>
                </div>
              </td>
              <td class="mono" style="font-size: 11.5px">{{ s.upstream_model }}</td>
              <td class="mono">{{ s.total_requests.toLocaleString() }}</td>
              <td class="mono">{{ s.success_count.toLocaleString() }}</td>
              <td class="mono">{{ s.failure_count.toLocaleString() }}</td>
              <td><span class="badge" :class="successBadgeClass(statSuccessRate(s))">{{ statSuccessRate(s).toFixed(1) }}%</span></td>
              <td class="mono">{{ formatDuration(statAvgLatency(s)) }}</td>
              <td class="mono">{{ s.total_retries.toLocaleString() }}</td>
              <td class="mono">{{ formatTokens(s.total_tokens) }}</td>
              <td class="mono muted" style="font-size: 11.5px">{{ s.last_request_at_ms ? formatTime(s.last_request_at_ms) : '—' }}</td>
            </tr>
            <tr v-if="sortedProviderStats.length === 0">
              <td colspan="10">
                <div class="empty" style="padding: 40px 24px">
                  <div class="empty__desc">{{ $t('monitoring.noStats') }}</div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Failure analysis -->
    <div class="grid-cols cols-3">
      <div class="card card--pad" @click="emit('set-tab', 'logs')">
        <div class="row mb-8">
          <span class="quick-tile__icon" style="background: var(--error-soft); color: var(--error)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
          </span>
          <div>
            <div class="card__title">{{ $t('monitoring.failureAnalysis') }}</div>
            <div class="card__sub">{{ $t('monitoring.failureAnalysisDesc') }}</div>
          </div>
          <div class="spacer" />
          <span v-if="topFailureProvider" class="badge badge--err">{{ topFailureProvider.rate.toFixed(1) }}%</span>
        </div>
        <div v-if="topFailureProvider" class="row" style="gap: 10px">
          <span class="pmark--sm" :style="{ background: providerColor(topFailureProvider.stat.provider_id), width: '28px', height: '28px' }">{{ providerGlyph(topFailureProvider.stat.provider_id, config.providers) }}</span>
          <div>
            <div style="font-weight: 700; font-size: 13.5px">{{ providerName(topFailureProvider.stat.provider_id, config.providers) }} · {{ topFailureProvider.stat.upstream_model }}</div>
            <div class="muted" style="font-size: 12px">{{ $t('monitoring.failures', { count: topFailureProvider.stat.failure_count }) }}</div>
          </div>
        </div>
        <div v-else class="muted" style="font-size: 13px">{{ $t('monitoring.noFailureData') }}</div>
      </div>

      <div class="card card--pad" @click="emit('set-tab', 'logs')">
        <div class="row mb-8">
          <span class="quick-tile__icon" style="background: var(--warning-soft); color: var(--warning)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
          </span>
          <div>
            <div class="card__title">{{ $t('monitoring.latencyAnalysis') }}</div>
            <div class="card__sub">{{ $t('monitoring.latencyAnalysisDesc') }}</div>
          </div>
          <div class="spacer" />
        </div>
        <div v-if="topLatencyProvider" class="row" style="gap: 10px">
          <span class="pmark--sm" :style="{ background: providerColor(topLatencyProvider.provider_id), width: '28px', height: '28px' }">{{ providerGlyph(topLatencyProvider.provider_id, config.providers) }}</span>
          <div>
            <div style="font-weight: 700; font-size: 13.5px">{{ providerName(topLatencyProvider.provider_id, config.providers) }} · {{ topLatencyProvider.upstream_model }}</div>
            <div class="muted" style="font-size: 12px">{{ $t('monitoring.avgLatencyDesc', { duration: formatDuration(statAvgLatency(topLatencyProvider)) }) }}</div>
          </div>
        </div>
        <div v-else class="muted" style="font-size: 13px">{{ $t('monitoring.noLatencyData') }}</div>
      </div>

      <div class="card card--pad" @click="emit('set-tab', 'logs')">
        <div class="row mb-8">
          <span class="quick-tile__icon" style="background: var(--warning-soft); color: var(--warning)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18"><path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>
          </span>
          <div>
            <div class="card__title">{{ $t('monitoring.rateLimitAnalysis') }}</div>
            <div class="card__sub">{{ $t('monitoring.rateLimitAnalysisDesc') }}</div>
          </div>
          <div class="spacer" />
        </div>
        <div v-if="topRateLimitProvider" class="row" style="gap: 10px">
          <span class="pmark--sm" :style="{ background: providerColor(topRateLimitProvider.provider_id), width: '28px', height: '28px' }">{{ providerGlyph(topRateLimitProvider.provider_id, config.providers) }}</span>
          <div>
            <div style="font-weight: 700; font-size: 13.5px">{{ providerName(topRateLimitProvider.provider_id, config.providers) }} · {{ topRateLimitProvider.upstream_model }}</div>
            <div class="muted" style="font-size: 12px">{{ $t('monitoring.rateLimitHint', { count: topRateLimitProvider.count }) }}</div>
          </div>
        </div>
        <div v-else class="muted" style="font-size: 13px">{{ $t('monitoring.noRateLimitData') }}</div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AppConfig, ProviderStat, RequestLog } from '../types';
import {
  formatTime,
  formatDuration,
  formatTokens,
  formatTokensCompact,
  providerColor,
  providerGlyph,
  providerName,
  statSuccessRate,
  statAvgLatency,
  successBadgeClass,
} from '../utils/format';

const { t } = useI18n();

interface Props {
  config: AppConfig;
  providerStats: ProviderStat[];
  requestLogs: RequestLog[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'set-tab': [tab: string];
  'clear-logs': [];
  'show-message': [text: string, type: 'success' | 'error' | 'warn' | 'info'];
}>();

const CHART_COLORS = ['var(--c2)', 'var(--c4)', 'var(--c5)', 'var(--c3)', 'var(--c1)'];

const monitorRange = ref<'today' | 'week' | 'month'>('week');
const monitorProviderFilter = ref('');
const sortKey = ref<string>('total_requests');
const sortDir = ref<'asc' | 'desc'>('desc');

const overallStats = computed(() => {
  let total = 0;
  let success = 0;
  let failure = 0;
  let totalLatency = 0;
  let totalTokens = 0;
  for (const s of props.providerStats) {
    total += s.total_requests;
    success += s.success_count;
    failure += s.failure_count;
    totalLatency += s.total_duration_ms;
    totalTokens += s.total_tokens;
  }
  const avgLatency = total > 0 ? Math.round(totalLatency / total) : 0;
  const successRate = total > 0 ? (success / total) * 100 : 0;
  return { total, success, failure, avgLatency, totalTokens, successRate };
});

const monitorRangeLabel = computed(() => {
  if (monitorRange.value === 'today') return t('monitoring.today');
  if (monitorRange.value === 'week') return t('monitoring.week');
  return t('monitoring.month');
});

const filteredProviderStats = computed(() => {
  if (!monitorProviderFilter.value) return props.providerStats;
  return props.providerStats.filter((s) => s.provider_id === monitorProviderFilter.value);
});

const sortedProviderStats = computed(() => {
  const list = [...filteredProviderStats.value];
  list.sort((a, b) => {
    let av: number | string = 0;
    let bv: number | string = 0;
    switch (sortKey.value) {
      case 'provider':
        av = providerName(a.provider_id, props.config.providers);
        bv = providerName(b.provider_id, props.config.providers);
        break;
      case 'model':
        av = a.upstream_model;
        bv = b.upstream_model;
        break;
      case 'total_requests':
        av = a.total_requests;
        bv = b.total_requests;
        break;
      case 'success_count':
        av = a.success_count;
        bv = b.success_count;
        break;
      case 'failure_count':
        av = a.failure_count;
        bv = b.failure_count;
        break;
      case 'success_rate':
        av = statSuccessRate(a);
        bv = statSuccessRate(b);
        break;
      case 'avg_latency':
        av = statAvgLatency(a);
        bv = statAvgLatency(b);
        break;
      case 'total_retries':
        av = a.total_retries;
        bv = b.total_retries;
        break;
      case 'total_tokens':
        av = a.total_tokens;
        bv = b.total_tokens;
        break;
      case 'last_request_at_ms':
        av = a.last_request_at_ms;
        bv = b.last_request_at_ms;
        break;
    }
    if (av < bv) return sortDir.value === 'asc' ? -1 : 1;
    if (av > bv) return sortDir.value === 'asc' ? 1 : -1;
    return 0;
  });
  return list;
});

function setSort(key: string): void {
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc';
  } else {
    sortKey.value = key;
    sortDir.value = 'desc';
  }
}

const topFailureProvider = computed(() => {
  const candidates = props.providerStats.filter((s) => s.failure_count > 0);
  if (candidates.length === 0) return null;
  let top = candidates[0];
  let topRate = statSuccessRate(top);
  for (const s of candidates) {
    const rate = statSuccessRate(s);
    if (rate < topRate) {
      top = s;
      topRate = rate;
    }
  }
  return { stat: top, rate: 100 - topRate };
});

const topLatencyProvider = computed(() => {
  const candidates = props.providerStats.filter((s) => s.total_requests > 0);
  if (candidates.length === 0) return null;
  let top = candidates[0];
  let topLat = statAvgLatency(top);
  for (const s of candidates) {
    const lat = statAvgLatency(s);
    if (lat > topLat) {
      top = s;
      topLat = lat;
    }
  }
  return top;
});

function isRateLimitLog(log: RequestLog): boolean {
  if (log.status === 429) return true;
  const text = `${log.error ?? ''}`.toLowerCase();
  return (
    text.includes('rate limit') ||
    text.includes('too many requests') ||
    text.includes('429')
  );
}

const topRateLimitProvider = computed(() => {
  const map: Record<string, { provider_id: string; upstream_model: string; count: number }> = {};
  for (const log of props.requestLogs) {
    if (!isRateLimitLog(log)) continue;
    const key = `${log.provider_id}::${log.upstream_model}`;
    if (!map[key]) {
      map[key] = { provider_id: log.provider_id, upstream_model: log.upstream_model, count: 0 };
    }
    map[key].count++;
  }
  const entries = Object.values(map);
  if (entries.length === 0) return null;
  let top = entries[0];
  for (const e of entries) {
    if (e.count > top.count) top = e;
  }
  return top;
});

const latencyValues = computed(() => {
  const vals: number[] = [];
  for (const s of props.providerStats) {
    const avg = statAvgLatency(s);
    if (avg > 0) vals.push(avg);
  }
  return vals.sort((a, b) => a - b);
});

function percentile(arr: number[], p: number): number {
  if (arr.length === 0) return 0;
  const idx = Math.ceil((p / 100) * arr.length) - 1;
  return arr[Math.max(0, idx)];
}

const latencyP50 = computed(() => percentile(latencyValues.value, 50));
const latencyP95 = computed(() => percentile(latencyValues.value, 95));
const latencyP99 = computed(() => percentile(latencyValues.value, 99));

interface LatencyBucket {
  label: string;
  max: number;
  color: string;
  count: number;
  percent: number;
}

const latencyPercentilesHtml = computed(() => {
  return t('monitoring.latencyPercentiles', {
    p50: '<strong style="color: var(--text)">' + latencyP50.value + '</strong>',
    p95: '<strong style="color: var(--text)">' + latencyP95.value + '</strong>',
    p99: '<strong style="color: var(--text)">' + latencyP99.value + '</strong>',
  });
});

const latencyBuckets = computed<LatencyBucket[]>(() => {
  const buckets: { label: string; max: number; color: string; count: number }[] = [
    { label: t('monitoring.latencyBuckets.under200'), max: 200, color: 'var(--success)', count: 0 },
    { label: t('monitoring.latencyBuckets._200to500'), max: 500, color: 'var(--c2)', count: 0 },
    { label: t('monitoring.latencyBuckets._500to1s'), max: 1000, color: 'var(--warning)', count: 0 },
    { label: t('monitoring.latencyBuckets._1sto3s'), max: 3000, color: 'var(--c5)', count: 0 },
    { label: t('monitoring.latencyBuckets.over3s'), max: Infinity, color: 'var(--error)', count: 0 },
  ];
  for (const s of props.providerStats) {
    const avg = statAvgLatency(s);
    const b = buckets.find((bucket) => avg < bucket.max);
    if (b) b.count += s.total_requests;
  }
  const total = buckets.reduce((sum, b) => sum + b.count, 0) || 1;
  return buckets.map((b) => ({ ...b, percent: Math.round((b.count / total) * 100) }));
});

const tokenDistribution = computed(() => {
  const map: Record<string, number> = {};
  for (const s of props.providerStats) {
    map[s.provider_id] = (map[s.provider_id] ?? 0) + s.total_tokens;
  }
  const total = overallStats.value.totalTokens || 1;
  return Object.entries(map).map(([id, tokens], i) => ({
    label: providerName(id, props.config.providers),
    tokens,
    percent: Math.round((tokens / total) * 100),
    color: CHART_COLORS[i % CHART_COLORS.length],
  }));
});

const tokenDonutSlices = computed(() => {
  const total = overallStats.value.totalTokens || 1;
  let offset = 0;
  const circumference = 2 * Math.PI * 70;
  return tokenDistribution.value.map((item) => {
    const frac = item.tokens / total;
    const dash = `${frac * circumference} ${circumference}`;
    const rotate = `rotate(${offset * 360 - 90} 100 100)`;
    offset += frac;
    return { color: item.color, dash, rotate };
  });
});

// 真实趋势：按自然日（本地时区）统计最近 8 天的请求数。
// 数据来源为请求日志（后端环形缓冲，最近 500 条），不再伪造曲线；
// 无日志的日子显示 0。趋势描述文案中的 range 由 monitorRangeLabel 提供。
const TREND_DAYS = 8;

const trendBuckets = computed(() => {
  const now = new Date();
  const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  const firstDayStart = todayStart - (TREND_DAYS - 1) * 86400000;
  const buckets: { label: string; count: number }[] = [];
  for (let i = 0; i < TREND_DAYS; i++) {
    const d = new Date(firstDayStart + i * 86400000);
    buckets.push({ label: `${d.getMonth() + 1}/${d.getDate()}`, count: 0 });
  }
  for (const log of props.requestLogs) {
    const d = new Date(log.started_at_ms);
    const dayStart = new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
    const idx = Math.round((dayStart - firstDayStart) / 86400000);
    if (idx >= 0 && idx < TREND_DAYS) {
      buckets[idx].count++;
    }
  }
  return buckets;
});

const maxBucketCount = computed(() =>
  Math.max(1, ...trendBuckets.value.map((b) => b.count))
);

const trendPoints = computed(() => {
  const days = trendBuckets.value;
  const maxC = maxBucketCount.value;
  return days.map((b, i) => ({
    x: 48 + (i / Math.max(1, days.length - 1)) * (780 - 48),
    y: 240 - (b.count / maxC) * 210,
  }));
});

const trendLinePath = computed(() => {
  return trendPoints.value.map((p, i) => `${i === 0 ? 'M' : 'L'} ${p.x} ${p.y}`).join(' ');
});

const trendAreaPath = computed(() => {
  const pts = trendPoints.value;
  if (pts.length === 0) return '';
  const first = pts[0];
  const last = pts[pts.length - 1];
  return `M ${first.x} 240 ${pts.map((p) => `L ${p.x} ${p.y}`).join(' ')} L ${last.x} 240 Z`;
});
</script>
