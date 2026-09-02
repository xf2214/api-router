<template>
  <div class="rmn-v2">
    <!-- 头部：序号 + 上下移 + 名 + 徽章 + 操作 -->
    <header class="rmn-v2__head">
      <span class="rmn-v2__index" :title="$t('routingTree.priority')">{{ index + 1 }}</span>
      <div class="row" style="gap: 4px;">
        <button
          type="button"
          class="btn btn--ghost btn--icon btn--sm"
          :disabled="index === 0"
          :title="$t('routingTree.moveUp')"
          @click.stop="emit('move-up')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="18 15 12 9 6 15" />
          </svg>
        </button>
        <button
          type="button"
          class="btn btn--ghost btn--icon btn--sm"
          :disabled="index === total - 1"
          :title="$t('routingTree.moveDown')"
          @click.stop="emit('move-down')"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </button>
      </div>
      <span class="mono rmn-v2__name" style="font-size: 13.5px;" :title="node.localName">{{ node.displayLabel }}</span>
      <div class="rmn-v2__badges">
        <span class="badge" :class="strategyBadgeClass(node.strategy)" style="padding: 2px 10px; font-size: 11px;">
          {{ strategyLabel(node.strategy) }}
        </span>
        <span class="badge badge--mute" style="padding: 2px 10px; font-size: 11px;">
          {{ $t('routingTree.memberWeight') }}: {{ node.memberWeight }}
        </span>
        <span v-if="node.targets.length > 0" class="badge badge--mute" style="padding: 2px 10px; font-size: 11px;">
          {{ $t('routing.targetsN', { count: node.targets.length }) }}
        </span>
      </div>
      <div class="grow" />
      <div class="row" style="gap: 6px;">
        <button type="button" class="btn btn--ghost btn--sm" @click.stop="onJumpAndEdit">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
            <path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>
          </svg>
          {{ $t('routingTree.editNode') }}
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="12" height="12" style="margin-left: 2px; opacity: .7;">
            <path d="M7 17 17 7"/><path d="M8 7h9v9"/>
          </svg>
        </button>
        <button
          type="button"
          class="btn btn--ghost btn--icon btn--sm"
          :title="$t('app.delete')"
          style="color: var(--error);"
          @click.stop="emit('delete')"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
            <path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
          </svg>
        </button>
      </div>
    </header>

    <!-- 主体：定义卡片 + tier 目标分组 -->
    <div class="rmn-v2__body">
      <!-- 引用模型定义时的差异化说明卡 -->
      <div v-if="node.hasModelDefinition && node.mappingModelId" class="def-card" style="margin: 0 0 8px 0; padding: 10px 12px;">
        <span class="def-card__icon" style="width: 26px; height: 26px; border-radius: 7px;">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
          </svg>
        </span>
        <div class="col" style="flex:1; min-width:0;">
          <span class="def-card__name" style="font-size: 12px;">{{ $t('routing.mappingSource.ref', { name: node.mappingModelId }) }}</span>
          <span class="def-card__desc" style="font-size: 11px;">
            {{ node.localName !== node.displayLabel ? `节点标签: ${node.displayLabel}` : $t('routing.refDefinitionSimple') }}
          </span>
        </div>
      </div>

      <!-- T1 主路径 -->
      <div v-if="tier1.length > 0" class="rmn-v2__tier-block">
        <div class="route-targets-tier__head route-targets-tier__head--1">
          T1 · {{ $t('routing.tierPrimary') }} · {{ tier1.length }}
        </div>
        <div v-for="t in tier1" :key="t.key" class="route-target-row" :class="{ 'is-disabled': !t.enabled }">
          <span class="route-target-row__pmark" :style="{ background: providerColor(t.providerId) }">
            {{ providerGlyph(t.providerId, providers) }}
          </span>
          <span class="route-target-row__provider" :class="{ muted: !t.enabled }">{{ t.providerName }}</span>
          <span class="mono route-target-row__model">{{ t.upstreamModel }}</span>
          <div class="route-target-row__meta">
            <span v-if="node.strategy === 'weighted'" class="chip--tier-n" style="font-size: 10.5px;">w={{ t.weight }}</span>
            <span v-if="!t.enabled" class="badge badge--danger" style="font-size: 10.5px;">{{ $t('routingTree.providerDisabled') }}</span>
          </div>
        </div>
      </div>

      <!-- T2+ 降级路径 -->
      <div v-if="tierGE2.length > 0" class="rmn-v2__tier-block">
        <div class="route-targets-tier__head route-targets-tier__head--2">
          T2+ · {{ $t('routing.tierFallback') }} · {{ tierGE2.length }}
        </div>
        <div v-for="t in tierGE2" :key="t.key" class="route-target-row" :class="{ 'is-disabled': !t.enabled }">
          <span class="route-target-row__pmark" :style="{ background: providerColor(t.providerId) }">
            {{ providerGlyph(t.providerId, providers) }}
          </span>
          <span class="route-target-row__provider" :class="{ muted: !t.enabled }">{{ t.providerName }}</span>
          <span class="mono route-target-row__model">{{ t.upstreamModel }}</span>
          <div class="route-target-row__meta">
            <span :class="`chip--tier-${t.tier <= 2 ? 2 : 'n'}`" style="font-size: 10.5px;">T{{ t.tier }}</span>
            <span v-if="node.strategy === 'weighted'" class="chip--tier-n" style="font-size: 10.5px;">w={{ t.weight }}</span>
            <span v-if="!t.enabled" class="badge badge--danger" style="font-size: 10.5px;">{{ $t('routingTree.providerDisabled') }}</span>
          </div>
        </div>
      </div>

      <!-- 空目标提示 -->
      <div v-if="node.targets.length === 0" class="row" style="padding: 8px 4px;">
        <span class="tag">{{ $t('routingTree.noTargets') }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { RouteModelNodeV2, RouteTargetLeaf } from '../../types/routing';
import type { ProviderConfig } from '../../types/config';
import {
  providerColor,
  providerGlyph,
  strategyLabel,
  strategyBadgeClass,
} from '../../utils/format';

interface Props {
  node: RouteModelNodeV2;
  index: number;
  total: number;
  providers: ProviderConfig[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'jump-and-edit'): void;
  (e: 'delete'): void;
  (e: 'move-up'): void;
  (e: 'move-down'): void;
}>();

const tier1 = computed<RouteTargetLeaf[]>(() =>
  [...props.node.targets]
    .filter(t => t.tier === 1)
    .sort((a, b) => a.weight - b.weight)
);
const tierGE2 = computed<RouteTargetLeaf[]>(() =>
  [...props.node.targets]
    .filter(t => t.tier >= 2)
    .sort((a, b) => a.tier - b.tier || a.weight - b.weight)
);

function onJumpAndEdit(): void {
  emit('jump-and-edit');
}
</script>

<style scoped>
.rmn-v2__index {
  width: 22px;
  height: 22px;
  border-radius: 50%;
  background: var(--primary-soft);
  color: var(--primary);
  font-weight: 700;
  font-size: 11.5px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex: none;
}
.rmn-v2__tier-block + .rmn-v2__tier-block {
  margin-top: 10px;
}
.muted {
  color: var(--text-3);
}
</style>
