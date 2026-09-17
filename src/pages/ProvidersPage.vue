<template>
  <section class="page">
    <header class="ph">
      <div>
        <h1 class="ph__title">{{ $t('provider.title') }}</h1>
        <p class="ph__desc">{{ $t('provider.desc', { count: config.providers.length }) }}</p>
      </div>
      <div class="ph__actions">
        <button class="btn btn--ghost btn--sm" :disabled="checkingAll" @click="emit('check-all')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
            <path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>
          </svg>
          {{ $t('provider.healthCheckAll') }}
        </button>
        <button class="btn btn--primary" @click="emit('add-provider')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="16" height="16"><path d="M12 5v14M5 12h14"/></svg>
          {{ $t('provider.add') }}
        </button>
      </div>
    </header>

    <div v-if="checkError" class="info-banner">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
        <circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/>
      </svg>
      <span>{{ checkError }}</span>
      <div class="spacer" />
      <button class="btn btn--soft btn--sm" @click="emit('check-all')">{{ $t('app.refresh') }}</button>
    </div>

    <!-- 筛选栏：状态 Chip 组 + 搜索 -->
    <div class="prov-filter-bar">
      <div class="chip-group">
        <button
          v-for="opt in statusFilterOpts"
          :key="opt.value"
          class="chip-filter"
          :class="{ 'is-active': statusFilter === opt.value }"
          @click="statusFilter = opt.value"
        >
          {{ opt.label }}
        </button>
      </div>
      <div class="col-search" style="margin: 0; flex: 1 1 240px;">
        <svg class="col-search__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="15" height="15">
          <circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/>
        </svg>
        <input v-model="providerSearch" class="input" :placeholder="$t('provider.searchPlaceholder')">
      </div>
    </div>

    <div>
      <div
        v-for="p in filteredProviders"
        :key="p.id"
        class="prov-item-v2"
        :class="{ 'is-disabled': !p.enabled }"
        @click="emit('edit-provider', p)"
      >
        <!-- 头部：标识 + 名称 + 健康 + 开关 + 菜单 -->
        <div class="prov-item-v2__head">
          <span class="pmark pmark--sm" :style="{ background: providerColor(p.id) }">{{ providerGlyph(p.id, config.providers) }}</span>
          <span class="prov-item-v2__name">{{ p.name }}</span>
          <span v-if="!p.enabled" class="prov-item-v2__disabled-tag">{{ $t('provider.disabledBadge') }}</span>
          <span class="health" :class="'health--' + healthClass(p.id, healthMap)">
            <span class="health-dot" :class="'health-dot--' + healthClass(p.id, healthMap)" />
            {{ healthText(p.id, healthMap) }}
          </span>
          <!-- 延迟进度条 -->
          <span v-if="healthMap[p.id]?.latency_ms != null" class="mini-progress">
            <span class="mini-progress__bar">
              <span
                class="mini-progress__fill"
                :class="latencyFillClass(healthMap[p.id]!.latency_ms!)"
                :style="{ width: latencyFillWidth(healthMap[p.id]!.latency_ms!) }"
              />
            </span>
            <span class="mono" style="font-size: 11px; color: var(--text-3);">{{ healthMap[p.id]?.latency_ms }}ms</span>
          </span>
          <div class="grow" />
          <label class="switch" :title="$t('provider.toggle')" @click.stop>
            <input
              type="checkbox"
              :checked="p.enabled"
              @change="toggleProviderEnabled(p)"
            >
            <span class="switch__track">
              <span class="switch__thumb" />
            </span>
          </label>
          <!-- ⋮ 操作菜单 -->
          <div class="menu-wrap" @click.stop>
            <button
              type="button"
              class="menu-btn"
              :title="$t('app.more')"
              @click.stop="toggleMenu(p.id)"
            >
              <svg viewBox="0 0 24 24" fill="currentColor" width="16" height="16">
                <circle cx="12" cy="5" r="1.8"/><circle cx="12" cy="12" r="1.8"/><circle cx="12" cy="19" r="1.8"/>
              </svg>
            </button>
            <div v-if="openMenuId === p.id" class="menu-dropdown" @click.stop>
              <button type="button" class="menu-item" @click.stop="onMenuCheck(p.id)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 12a9 9 0 1 0 3-6.7L3 8"/><path d="M3 3v5h5"/>
                </svg>
                {{ $t('provider.check') }}
              </button>
              <button type="button" class="menu-item" @click.stop="onMenuEdit(p)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>
                </svg>
                {{ $t('provider.edit') }}
              </button>
              <div class="menu-divider" />
              <button type="button" class="menu-item menu-item--danger" @click.stop="onMenuDelete(p.id)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
                </svg>
                {{ $t('provider.delete') }}
              </button>
            </div>
          </div>
        </div>

        <!-- 信息区 -->
        <div class="prov-item-v2__body">
          <!-- URL 行 -->
          <span class="prov-item-v2__url" :title="p.base_url">{{ p.base_url }}</span>
          <div class="prov-item-v2__meta">
            <span class="info-row">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13">
                <circle cx="12" cy="12" r="6"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="M4.93 4.93l1.41 1.41"/><path d="M17.66 17.66l1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="M4.93 19.07l1.41-1.41"/><path d="M17.66 6.34l1.41-1.41"/>
              </svg>
              {{ $t('provider.models', { count: p.default_models.length }) }}
            </span>
            <span class="info-row">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13">
                <path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z"/>
              </svg>
              {{ $t('provider.qps', { limit: p.qps_limit || '∞' }) }}
            </span>
            <span class="info-row">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13">
                <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
              </svg>
              {{ p.timeout_seconds || 60 }}s
            </span>
            <span class="info-row" :class="{ 'info-row--muted': !p.disable_proxy }">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13">
                <path d="M5 12h14"/><path d="m12 5 7 7-7 7"/>
              </svg>
              {{ p.disable_proxy ? $t('provider.proxyDisabledShort') : $t('provider.proxySystemShort') }}
            </span>
            <span v-if="healthMap[p.id]" class="info-row info-row--muted">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13">
                <path d="M12 8v4l3 3"/><circle cx="12" cy="12" r="10"/>
              </svg>
              {{ healthTime(p.id, healthMap) }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="loading && config.providers.length === 0" class="empty" style="padding: 56px 24px">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="28" height="28" class="spin-anim" style="transform-origin: center; color: var(--primary);">
          <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
        </svg>
        <div class="empty__desc">{{ $t('logs.refreshing') }}</div>
      </div>
      <div v-else-if="filteredProviders.length === 0" class="empty" style="padding: 56px 24px">
        <div class="empty__icon" style="background: var(--primary-soft); color: var(--primary);">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="30" height="30">
            <rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>
          </svg>
        </div>
        <div class="empty__title">
          {{ config.providers.length === 0 ? $t('provider.noProvidersYet') : $t('provider.noMatch') }}
        </div>
        <div class="empty__desc">
          {{ config.providers.length === 0
            ? $t('provider.noProvidersHint')
            : $t('provider.noMatchDesc') }}
        </div>
        <button v-if="config.providers.length === 0" class="btn btn--primary btn--sm" @click="emit('add-provider')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
          {{ $t('provider.add') }}
        </button>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';

import type { AppConfig, ProviderConfig, ProviderHealth } from '../types';
import {
  providerColor,
  providerGlyph,
  healthClass,
  healthText,
  healthTime,
} from '../utils/format';

const { t } = useI18n();

interface Props {
  config: AppConfig;
  healthStatus: ProviderHealth[];
  checkingAll: boolean;
  loading?: boolean;
  checkError?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  checkError: null,
});
const emit = defineEmits<{
  'add-provider': [];
  'edit-provider': [p: ProviderConfig];
  'delete-provider': [id: string];
  'check-provider': [id: string];
  'check-all': [];
  'persist-config': [];
}>();

const providerSearch = ref('');
const statusFilter = ref('all');
const openMenuId = ref<string | null>(null);

const statusFilterOpts = computed(() => [
  { value: 'all', label: t('provider.filterAll', { count: props.config.providers.length }) },
  { value: 'enabled', label: t('provider.filterEnabled', { count: props.config.providers.filter(p => p.enabled).length }) },
  { value: 'disabled', label: t('provider.filterDisabled', { count: props.config.providers.filter(p => !p.enabled).length }) },
]);

const healthMap = computed(() => {
  const map: Record<string, ProviderHealth> = {};
  for (const h of props.healthStatus) {
    map[h.provider_id] = h;
  }
  return map;
});

const filteredProviders = computed(() => {
  let list = props.config.providers;
  // 状态筛选
  if (statusFilter.value === 'enabled') list = list.filter((p) => p.enabled);
  else if (statusFilter.value === 'disabled') list = list.filter((p) => !p.enabled);
  // 关键字搜索
  const q = providerSearch.value.trim().toLowerCase();
  if (q) {
    list = list.filter(
      (p) => p.name.toLowerCase().includes(q) || p.base_url.toLowerCase().includes(q)
    );
  }
  return list;
});

function toggleProviderEnabled(p: ProviderConfig) {
  p.enabled = !p.enabled;
  emit('persist-config');
}

/* ---------- 延迟进度条计算 ---------- */
function latencyFillWidth(ms: number): string {
  // 0ms = 0%, 500ms = 100%，超过 500ms 截断
  const pct = Math.min(100, (ms / 500) * 100);
  return `${pct}%`;
}

function latencyFillClass(ms: number): string {
  if (ms <= 200) return 'mini-progress__fill--ok';
  if (ms <= 500) return 'mini-progress__fill--warn';
  return 'mini-progress__fill--err';
}

/* ---------- ⋮ 菜单点击外部关闭 ---------- */
function closeMenuOnClick(e: MouseEvent): void {
  if (openMenuId.value == null) return;
  const target = e.target as HTMLElement;
  if (!target.closest?.('.menu-wrap')) openMenuId.value = null;
}

function closeMenuOnEsc(e: KeyboardEvent): void {
  if (e.key === 'Escape') openMenuId.value = null;
}

onMounted(() => {
  document.addEventListener('click', closeMenuOnClick);
  document.addEventListener('keydown', closeMenuOnEsc);
});

onBeforeUnmount(() => {
  document.removeEventListener('click', closeMenuOnClick);
  document.removeEventListener('keydown', closeMenuOnEsc);
});

function toggleMenu(id: string): void {
  openMenuId.value = openMenuId.value === id ? null : id;
}

function onMenuCheck(id: string): void {
  openMenuId.value = null;
  emit('check-provider', id);
}

function onMenuEdit(p: ProviderConfig): void {
  openMenuId.value = null;
  emit('edit-provider', p);
}

function onMenuDelete(id: string): void {
  openMenuId.value = null;
  emit('delete-provider', id);
}
</script>
