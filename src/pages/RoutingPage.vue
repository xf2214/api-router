<template>
  <section class="page">
    <header class="ph">
      <div>
        <h1 class="ph__title">{{ $t('routing.title') }}</h1>
        <p class="ph__desc">{{ $t('routing.desc', { count: config.models.length }) }}</p>
      </div>
      <div class="ph__actions">
        <button class="btn btn--primary" @click="emit('add-model')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="16" height="16"><path d="M12 5v14M5 12h14"/></svg>
          {{ $t('routing.add') }}
        </button>
      </div>
    </header>

    <!-- 筛选栏：提示 + 分组 chip + 策略下拉 + 搜索 -->
    <div class="filter-bar" style="gap: 10px;">
      <span class="field__hint field__hint--sm" style="margin-right: auto;">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" style="vertical-align: -2px; margin-right: 4px; color: var(--info);">
          <circle cx="12" cy="12" r="10" />
          <line x1="12" y1="16" x2="12" y2="12" />
          <line x1="12" y1="8" x2="12.01" y2="8" />
        </svg>
        {{ $t('routing.groupsManagedInTree') }}
      </span>
      <div class="chip-group">
        <button
          class="chip-filter"
          :class="{ 'is-active': selectedGroup === '' }"
          @click="selectedGroup = ''"
        >
          {{ $t('routing.allGroups') }}
        </button>
        <button
          v-for="g in allGroups"
          :key="g"
          class="chip-filter"
          :class="{ 'is-active': selectedGroup === g }"
          @click="selectedGroup = g"
        >
          {{ g }}
        </button>
      </div>
      <select v-model="selectedStrategy" class="select" style="width: auto; flex: none;">
        <option value="">{{ $t('routing.allStrategies') }}</option>
        <option value="priority">{{ $t('routing.strategy.priority') }}</option>
        <option value="round_robin">{{ $t('routing.strategy.roundRobin') }}</option>
        <option value="weighted">{{ $t('routing.strategy.weighted') }}</option>
      </select>
      <div class="col-search" style="flex: 1 1 240px; min-width: 200px; margin: 0;">
        <svg class="col-search__icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="15" height="15">
          <circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/>
        </svg>
        <input v-model="modelSearch" class="input" :placeholder="$t('routing.searchPlaceholder')">
      </div>
    </div>

    <div>
      <div
        v-for="m in filteredModels"
        :key="m.local_name"
        class="mm-item-v2"
        @click="emit('edit-model', m)"
      >
        <!-- 头部：模型名 + 徽章组 -->
        <div class="mm-item-v2__head">
          <span class="mm-item-v2__name">{{ m.local_name }}</span>
          <div class="mm-item-v2__badges">
            <!-- 策略徽章 -->
            <span class="badge" :class="strategyBadgeClass(m.strategy)" style="padding: 3px 11px; font-size: 12px;">
              {{ strategyLabel(m.strategy) }}
            </span>
            <!-- 分组徽章 -->
            <span class="badge badge--mute" style="padding: 3px 11px;">
              {{ m.group || $t('routing.defaultGroup') }}
            </span>
            <!-- 来源徽章：引用定义 / 直接配置 差异化配色 -->
            <span class="badge" :class="m.model_id ? 'badge--info' : 'badge--mute'" style="padding: 3px 11px;">
              {{ mappingSourceLabel(m) }}
            </span>
            <!-- 无回退徽章 -->
            <span v-if="!m.fallback_enabled" class="badge badge--warn" style="padding: 3px 11px;">
              {{ $t('routing.noFallback') }}
            </span>
          </div>
          <div class="grow" />
        </div>

        <!-- 主体：摘要 + 引用定义卡 + 目标子卡片 -->
        <div class="mm-item-v2__body">
          <!-- 摘要条 -->
          <div class="summary-bar">
            <span class="summary-bar__icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="9"/><path d="M3 12h18"/><path d="M12 3a14 14 0 0 1 0 18"/><path d="M12 3a14 14 0 0 0 0 18"/>
              </svg>
            </span>
            {{ $t('routing.targetsSummary', { count: m.targets.length, retries: m.max_retries ?? 0, fallbackStatus: m.fallback_enabled ? $t('routing.fallbackEnabled') : $t('routing.fallbackDisabled') }) }}
          </div>

          <!-- 引用定义差异化卡片 -->
          <div v-if="m.model_id" class="def-card" @click.stop>
            <span class="def-card__icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
              </svg>
            </span>
            <div class="col" style="flex:1; min-width:0;">
              <span class="def-card__name">{{ definitionName(m.model_id) }}</span>
              <span class="def-card__desc">{{ $t('routing.refDefinitionHint', { count: definitionAccessPointCount(m.model_id) }) }}</span>
            </div>
          </div>

          <!-- 目标子卡片（按 tier 分组显示） -->
          <div v-if="m.targets.length > 0" class="mm-item-v2__targets">
            <!-- T1 分组 -->
            <div v-if="targetsByTier(m.targets, 1).length > 0" class="route-targets-tier">
              <div class="route-targets-tier__head route-targets-tier__head--1">
                T1 {{ $t('routing.tierPrimary') }} · {{ targetsByTier(m.targets, 1).length }}
              </div>
              <div
                v-for="(t, i) in targetsByTier(m.targets, 1)"
                :key="'t1-'+i"
                class="route-target-row"
              >
                <span class="route-target-row__pmark" :style="{ background: providerColor(t.provider_id) }">
                  {{ providerGlyph(t.provider_id, config.providers) }}
                </span>
                <span class="route-target-row__provider">{{ providerName(t.provider_id, config.providers) }}</span>
                <span class="mono route-target-row__model" style="font-size: 12px;">{{ t.model_name }}</span>
                <div class="route-target-row__meta">
                  <span v-if="m.strategy === 'weighted'" class="chip--tier-n" style="font-size: 11px;">w={{ t.weight }}</span>
                </div>
              </div>
            </div>

            <!-- T2 及以上 分组 -->
            <div v-if="targetsByTierGE(m.targets, 2).length > 0" class="route-targets-tier">
              <div class="route-targets-tier__head route-targets-tier__head--2">
                T2+ {{ $t('routing.tierFallback') }} · {{ targetsByTierGE(m.targets, 2).length }}
              </div>
              <div
                v-for="(t, i) in targetsByTierGE(m.targets, 2)"
                :key="'t2-'+i"
                class="route-target-row"
              >
                <span class="route-target-row__pmark" :style="{ background: providerColor(t.provider_id) }">
                  {{ providerGlyph(t.provider_id, config.providers) }}
                </span>
                <span class="route-target-row__provider">{{ providerName(t.provider_id, config.providers) }}</span>
                <span class="mono route-target-row__model" style="font-size: 12px;">{{ t.model_name }}</span>
                <div class="route-target-row__meta">
                  <span :class="`chip--tier-${t.tier <= 2 ? 2 : 'n'}`">T{{ t.tier }}</span>
                  <span v-if="m.strategy === 'weighted'" class="chip--tier-n" style="font-size: 11px;">w={{ t.weight }}</span>
                </div>
              </div>
            </div>
          </div>
          <div v-else class="row mt-8">
            <span class="tag">{{ $t('routingTree.noTargets') }}</span>
          </div>
        </div>

        <!-- 底部操作 -->
        <div class="mm-item-v2__foot" @click.stop>
          <button class="btn btn--ghost btn--sm" :disabled="testingModels[m.local_name]" @click="testModel(m.local_name)">
            <svg v-if="!testingModels[m.local_name]" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
              <path d="M22 2 11 13"/><path d="m22 2-7 20-4-9-9-4 20-7z"/>
            </svg>
            <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14" class="spin-anim" style="transform-origin: center;">
              <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
            </svg>
            {{ testingModels[m.local_name] ? $t('routing.testing') : $t('routing.test') }}
          </button>
          <button class="btn btn--ghost btn--sm" @click="emit('edit-model', m)">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
              <path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>
            </svg>
            {{ $t('routing.edit') }}
          </button>
          <div class="grow" />
          <button class="btn btn--danger btn--sm" @click="emit('delete-model', m.local_name)">
            {{ $t('routing.delete') }}
          </button>
        </div>
      </div>

      <!-- 空状态 -->
      <div v-if="loading && config.models.length === 0" class="empty" style="padding: 56px 24px">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="28" height="28" class="spin-anim" style="transform-origin: center; color: var(--primary);">
          <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
        </svg>
        <div class="empty__desc">{{ $t('logs.refreshing') }}</div>
      </div>
      <div v-else-if="filteredModels.length === 0" class="empty" style="padding: 56px 24px">
        <div class="empty__icon" style="background: var(--primary-soft); color: var(--primary);">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="30" height="30">
            <circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="6" r="2.5"/><circle cx="12" cy="18" r="2.5"/><path d="M8.2 7.2 16 16"/><path d="M7.5 8 11 15.5"/><path d="M15.8 7.5 13 15.5"/>
          </svg>
        </div>
        <div class="empty__title">
          {{ config.models.length === 0 ? $t('routing.noModelsYet') : $t('routing.noMatch') }}
        </div>
        <div class="empty__desc">
          {{ config.models.length === 0
            ? $t('routing.noModelsHint')
            : $t('routing.noMatchDesc') }}
        </div>
        <button v-if="config.models.length === 0" class="btn btn--primary btn--sm" @click="emit('add-model')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
          {{ $t('routing.add') }}
        </button>
      </div>
    </div>

    <!-- 测试结果面板（增强版：三列布局） -->
    <div v-if="modelTestResult" ref="testResultPanelRef" class="card card--pad mt-16">
      <div class="row mb-8">
        <span class="card__title" style="padding: 0;">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="16" height="16" style="vertical-align: -3px; margin-right: 6px; color: var(--primary);">
            <path d="M9 11l3 3L22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
          </svg>
          {{ $t('routing.testResult', { name: modelTestResult.local_name }) }}
        </span>
        <div class="grow" />
        <button class="btn btn--ghost btn--icon btn--sm" @click="modelTestResult = null">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="16" height="16"><path d="M18 6 6 18M6 6l12 12"/></svg>
        </button>
      </div>
      <div>
        <div v-for="(t, i) in modelTestResult.targets" :key="i" class="test-result-row">
          <div class="test-result-row__provider">
            <span class="health-dot" :class="t.online ? 'health-dot--ok' : 'health-dot--err'" />
            <span class="pmark--sm" :style="{ background: providerColor(t.provider_id) }">
              {{ providerGlyph(t.provider_id, config.providers) }}
            </span>
            <span class="test-result-row__provider-name">{{ providerName(t.provider_id, config.providers) }}</span>
          </div>
          <span class="mono test-result-row__model">{{ t.model_name }}</span>
          <div class="test-result-row__latency">
            <template v-if="t.latency_ms != null">
              <span class="mini-progress">
                <span class="mini-progress__bar">
                  <span
                    class="mini-progress__fill"
                    :class="t.online ? (t.latency_ms <= 200 ? 'mini-progress__fill--ok' : t.latency_ms <= 500 ? 'mini-progress__fill--warn' : 'mini-progress__fill--err') : 'mini-progress__fill--err'"
                    :style="{ width: t.online ? `${Math.min(100, (t.latency_ms / 500) * 100)}%` : '100%' }"
                  />
                </span>
              </span>
              <span class="mono" style="font-size: 11.5px;">{{ t.latency_ms }}ms</span>
            </template>
          </div>
          <span v-if="t.error" class="badge badge--err" style="max-width: 220px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            {{ t.error }}
          </span>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AppConfig, ModelMapping, ModelTestResult, ModelDefinition, ModelTarget } from '../types';
import * as tauri from '../services/tauri';
import {
  providerColor,
  providerGlyph,
  providerName,
  strategyLabel,
  strategyBadgeClass,
  sortedTargets,
} from '../utils/format';

const { t } = useI18n();

interface Props {
  config: AppConfig;
  modelDefinitions?: ModelDefinition[];
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelDefinitions: () => [],
  loading: false,
});
const emit = defineEmits<{
  'add-model': [];
  'edit-model': [m: ModelMapping];
  'delete-model': [localName: string];
  'show-message': [text: string, type: 'success' | 'error' | 'warn' | 'info'];
}>();

const selectedGroup = ref('');
const selectedStrategy = ref('');
const modelSearch = ref('');
const testingModels = ref<Record<string, boolean>>({});
const modelTestResult = ref<ModelTestResult | null>(null);
const testResultPanelRef = ref<HTMLElement | null>(null);

const allGroups = computed(() => {
  const set = new Set<string>();
  for (const m of props.config.models) {
    if (m.group && m.group !== t('routing.defaultGroup')) set.add(m.group);
  }
  return Array.from(set).sort();
});

const filteredModels = computed(() => {
  let list = props.config.models;
  // 分组筛选
  if (selectedGroup.value) {
    list = list.filter((m) => (m.group || t('routing.defaultGroup')) === selectedGroup.value);
  }
  // 策略筛选
  if (selectedStrategy.value) {
    list = list.filter((m) => m.strategy === selectedStrategy.value);
  }
  // 关键字搜索：本地名 + 上游目标模型名
  const q = modelSearch.value.trim().toLowerCase();
  if (q) {
    list = list.filter((m) => {
      if (m.local_name.toLowerCase().includes(q)) return true;
      for (const tg of m.targets) {
        if (tg.model_name.toLowerCase().includes(q)) return true;
      }
      return false;
    });
  }
  return list;
});

function definitionName(id: string): string {
  const def = props.modelDefinitions.find((d) => d.id === id);
  return def ? `${def.display_name} (${id})` : id;
}

function definitionAccessPointCount(id: string): number {
  const def = props.modelDefinitions.find((d) => d.id === id);
  return def?.access_points?.length ?? 0;
}

function mappingSourceLabel(m: ModelMapping): string {
  if (m.model_id) return t('routing.mappingSource.ref', { name: definitionName(m.model_id) });
  return t('routing.mappingSource.direct', { count: m.targets.length });
}

/** targets 按 tier 筛选（单 tier） */
function targetsByTier(targets: ModelTarget[], tier: number): ModelTarget[] {
  return sortedTargets(targets).filter((t) => (t.tier ?? 1) === tier);
}

/** targets tier >= N（降级路径） */
function targetsByTierGE(targets: ModelTarget[], minTier: number): ModelTarget[] {
  return sortedTargets(targets).filter((t) => (t.tier ?? 1) >= minTier);
}

async function testModel(localName: string): Promise<void> {
  testingModels.value[localName] = true;
  let hasError = false;
  try {
    modelTestResult.value = await tauri.testModelConnection(localName);
  } catch (e) {
    hasError = true;
    modelTestResult.value = {
      local_name: localName,
      targets: [
        {
          provider_id: '',
          model_name: '',
          online: false,
          latency_ms: null,
          error: String(e),
        },
      ],
    };
  } finally {
    testingModels.value[localName] = false;
  }
  // 滚动 + Toast 反馈
  await nextTick();
  if (testResultPanelRef.value) {
    testResultPanelRef.value.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
  const result = modelTestResult.value;
  if (result) {
    const total = result.targets.length;
    const okCount = result.targets.filter((t) => t.online).length;
    if (hasError || okCount === 0) {
      emit('show-message', `连通测试失败，请查看下方详情（${total} 个目标）`, 'error');
    } else if (okCount < total) {
      emit('show-message', `连通测试完成，${okCount}/${total} 个目标已就绪`, 'warn');
    } else {
      emit('show-message', `连通测试完成，${okCount} 个目标已就绪`, 'success');
    }
  }
}
</script>
