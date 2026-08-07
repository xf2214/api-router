<template>
  <div class="rmn-v2">
    <!-- 头部三列：名 + 徽章 + 操作 -->
    <header class="rmn-v2__head">
      <span class="mono rmn-v2__name" style="font-size: 13.5px;">{{ model.local_name }}</span>
      <div class="rmn-v2__badges">
        <span class="badge" :class="strategyBadgeClass(model.strategy)" style="padding: 2px 10px; font-size: 11px;">
          {{ strategyLabel(model.strategy) }}
        </span>
        <span v-if="!model.fallback_enabled" class="badge badge--warn" style="padding: 2px 10px; font-size: 11px;">
          {{ $t('routing.noFallback') }}
        </span>
        <span v-if="model.targets.length > 0" class="badge badge--mute" style="padding: 2px 10px; font-size: 11px;">
          {{ $t('routing.targetsN', { count: model.targets.length }) }}
        </span>
        <span v-if="model.max_retries && model.max_retries > 0" class="badge badge--mute" style="padding: 2px 10px; font-size: 11px;">
          ↻ {{ model.max_retries }}
        </span>
      </div>
      <div class="grow" />
      <div class="row" style="gap: 6px;">
        <button class="btn btn--ghost btn--icon btn--sm" :title="$t('routing.test')" @click="emit('test-model', model.local_name)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
            <path d="M22 2 11 13"/><path d="m22 2-7 20-4-9-9-4 20-7z"/>
          </svg>
        </button>
        <button class="btn btn--ghost btn--icon btn--sm" :title="$t('routing.edit')" @click="emit('edit-model', model.local_name)">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
            <path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>
          </svg>
        </button>
        <!-- ⋮ 菜单：删除等次级操作 -->
        <div class="menu-wrap">
          <button class="menu-btn menu-btn--sm" :title="$t('routing.moreActions', '更多')" @click.stop="openMenu = !openMenu">
            <svg viewBox="0 0 24 24" fill="currentColor" width="15" height="15"><circle cx="12" cy="5" r="1.8"/><circle cx="12" cy="12" r="1.8"/><circle cx="12" cy="19" r="1.8"/></svg>
          </button>
          <div v-if="openMenu" class="menu-dropdown" @click.stop>
            <button class="menu-item" @click="onEdit">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13">
                <path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>
              </svg>
              {{ $t('routing.edit') }}
            </button>
            <button class="menu-item" @click="onTest">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13">
                <path d="M22 2 11 13"/><path d="m22 2-7 20-4-9-9-4 20-7z"/>
              </svg>
              {{ $t('routing.test') }}
            </button>
            <div class="menu-divider"/>
            <button class="menu-item menu-item--danger" @click="onDelete">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="13" height="13">
                <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"/>
              </svg>
              {{ $t('routing.delete') }}
            </button>
          </div>
        </div>
      </div>
    </header>

    <!-- 主体：摘要 + 目标 tier 分组 -->
    <div class="rmn-v2__body">
      <!-- 引用定义摘要条（有 model_id 时显示） -->
      <div v-if="model.model_id" class="def-card" style="margin: 0; padding: 10px 12px;">
        <span class="def-card__icon" style="width: 26px; height: 26px; border-radius: 7px;">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
          </svg>
        </span>
        <div class="col" style="flex:1; min-width:0;">
          <span class="def-card__name" style="font-size: 12px;">{{ defName }}</span>
          <span class="def-card__desc" style="font-size: 11px;">{{ $t('routing.accessPointCount', { count: defCount }) }}</span>
        </div>
      </div>

      <!-- T1 主路径 -->
      <div v-if="tier1.length > 0" class="rmn-v2__tier-block">
        <div class="route-targets-tier__head route-targets-tier__head--1" style="border-radius: 8px 8px 0 0;">
          T1 · {{ $t('routing.tierPrimary', '主路径') }} · {{ tier1.length }}
        </div>
        <div class="route-target-row" v-for="(t, i) in tier1" :key="'n1-'+i">
          <span class="route-target-row__pmark" :style="{ background: providerColor(t.provider_id) }">
            {{ providerGlyph(t.provider_id, providers) }}
          </span>
          <span class="route-target-row__provider">{{ providerName(t.provider_id, providers) }}</span>
          <span class="route-target-row__model mono">{{ t.model_name }}</span>
          <div class="route-target-row__meta">
            <span v-if="model.strategy === 'weighted'" class="chip--tier-n" style="font-size: 10.5px;">w={{ t.weight }}</span>
          </div>
        </div>
      </div>

      <!-- T2+ 降级路径 -->
      <div v-if="tierGE2.length > 0" class="rmn-v2__tier-block">
        <div class="route-targets-tier__head route-targets-tier__head--2" style="border-radius: 8px 8px 0 0;">
          T2+ · {{ $t('routing.tierFallback', '降级路径') }} · {{ tierGE2.length }}
        </div>
        <div class="route-target-row" v-for="(t, i) in tierGE2" :key="'n2-'+i">
          <span class="route-target-row__pmark" :style="{ background: providerColor(t.provider_id) }">
            {{ providerGlyph(t.provider_id, providers) }}
          </span>
          <span class="route-target-row__provider">{{ providerName(t.provider_id, providers) }}</span>
          <span class="route-target-row__model mono">{{ t.model_name }}</span>
          <div class="route-target-row__meta">
            <span :class="`chip--tier-${t.tier <= 2 ? 2 : 'n'}`" style="font-size: 10.5px;">T{{ t.tier }}</span>
            <span v-if="model.strategy === 'weighted'" class="chip--tier-n" style="font-size: 10.5px;">w={{ t.weight }}</span>
          </div>
        </div>
      </div>

      <!-- 完全空目标 -->
      <div v-if="model.targets.length === 0" class="row" style="padding: 8px 4px;">
        <span class="tag">{{ $t('routingTree.noTargets') }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onBeforeUnmount } from 'vue';
import type { ProviderConfig, ModelMapping, ModelDefinition } from '../types';
import {
  providerColor,
  providerGlyph,
  providerName,
  strategyLabel,
  strategyBadgeClass,
  sortedTargets,
} from '../utils/format';

interface Props {
  model: ModelMapping;
  providers: ProviderConfig[];
  modelDefinitions?: ModelDefinition[];
}

const props = withDefaults(defineProps<Props>(), {
  modelDefinitions: () => [],
});
const emit = defineEmits<{
  'edit-model': [localName: string];
  'delete-model': [localName: string];
  'test-model': [localName: string];
}>();

const openMenu = ref(false);

function closeMenu(e: MouseEvent) {
  if (!openMenu.value) return;
  const el = (e.target as HTMLElement | null)?.closest('.menu-wrap');
  if (!el) openMenu.value = false;
}
function onKey(e: KeyboardEvent) {
  if (e.key === 'Escape') openMenu.value = false;
}
onMounted(() => {
  window.addEventListener('click', closeMenu, true);
  window.addEventListener('keydown', onKey);
});
onBeforeUnmount(() => {
  window.removeEventListener('click', closeMenu, true);
  window.removeEventListener('keydown', onKey);
});

function onEdit() {
  openMenu.value = false;
  emit('edit-model', props.model.local_name);
}
function onTest() {
  openMenu.value = false;
  emit('test-model', props.model.local_name);
}
function onDelete() {
  openMenu.value = false;
  emit('delete-model', props.model.local_name);
}

const tier1 = computed(() => sortedTargets(props.model.targets).filter((t) => (t.tier ?? 1) === 1));
const tierGE2 = computed(() => sortedTargets(props.model.targets).filter((t) => (t.tier ?? 1) >= 2));

const defName = computed(() => {
  const d = props.modelDefinitions.find((x) => x.id === props.model.model_id);
  return d ? `${d.display_name} (${d.id})` : props.model.model_id ?? '';
});
const defCount = computed(() => props.modelDefinitions.find((x) => x.id === props.model.model_id)?.access_points?.length ?? 0);
</script>
