<template>
  <section class="route-group-v2">
    <!-- 头部：分组名 + 徽章 + 操作 -->
    <header class="route-group-v2__head">
      <span class="pmark" :style="{ background: colorFromName(group.name || $t('routing.defaultGroup')) }">{{ groupGlyph(group.name || $t('routing.defaultGroup')) }}</span>
      <span class="route-group-v2__name">{{ group.name || $t('routing.defaultGroup') }}</span>
      <div class="route-group-v2__badges">
        <span class="badge badge--mute" style="font-size: 11px; padding: 2px 10px;">
          {{ $t('routing.modelCount', { count: group.models.length }) }}
        </span>
        <span v-if="group.exported" class="badge badge--ok" style="font-size: 11px; padding: 2px 10px;">
          {{ $t('routing.exportedBadge', '已导出') }}
        </span>
        <span v-if="group.fallback" class="badge badge--info" style="font-size: 11px; padding: 2px 10px;">
          {{ $t('routing.fallbackBadge', { name: group.fallback }, `降级→${group.fallback}`) }}
        </span>
      </div>
      <div class="grow" />
      <button
        class="btn btn--ghost btn--icon btn--sm"
        :title="$t('routing.collapse')"
        @click="collapsed = !collapsed"
      >
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          width="15"
          height="15"
          :style="{ transform: collapsed ? 'rotate(-90deg)' : 'none', transition: 'transform .18s ease' }"
        >
          <polyline points="6 9 12 15 18 9" />
        </svg>
      </button>
      <button class="btn btn--ghost btn--sm" @click="emit('add-model', group.name || '')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="14" height="14">
          <path d="M12 5v14M5 12h14"/>
        </svg>
        {{ $t('routing.add') }}
      </button>
    </header>

    <!-- 体：模型列表 / 空状态 -->
    <div v-if="!collapsed" class="route-group-v2__body">
      <!-- 空分组引导 CTA -->
      <div v-if="group.models.length === 0" class="empty" style="padding: 32px 20px; gap: 10px;">
        <div class="empty__icon" style="background: var(--primary-soft); color: var(--primary); width: 48px; height: 48px; border-radius: 14px;">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="24" height="24">
            <circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="6" r="2.5"/><circle cx="12" cy="18" r="2.5"/><path d="M8.2 7.2 16 16"/>
          </svg>
        </div>
        <div class="empty__title" style="font-size: 14px;">
          {{ $t('routing.emptyGroup', '此分组下还没有模型') }}
        </div>
        <div class="empty__desc" style="font-size: 12px; color: var(--text-3);">
          {{ $t('routing.emptyGroupDesc', '添加第一个模型，将其路由到上游目标。') }}
        </div>
        <button class="btn btn--primary btn--sm" @click="emit('add-model', group.name || '')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
          {{ $t('routing.addModelInGroup', '添加模型') }}
        </button>
      </div>

      <!-- 有模型时显示节点列表 -->
      <RouteModelNode
        v-for="m in group.models"
        :key="m.local_name"
        :model="m"
        :providers="config.providers"
        :modelDefinitions="modelDefinitions"
        @edit-model="n => emit('edit-model', n)"
        @delete-model="n => emit('delete-model', n)"
        @test-model="n => emit('test-model', n)"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import RouteModelNode from './RouteModelNode.vue';
import type { AppConfig, ModelMapping, ModelDefinition } from '../types';

interface RouteGroupLocal {
  name?: string;
  exported?: boolean;
  fallback?: string;
  models: ModelMapping[];
}

interface Props {
  group: RouteGroupLocal;
  config: AppConfig;
  modelDefinitions?: ModelDefinition[];
  defaultCollapsed?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  modelDefinitions: () => [],
  defaultCollapsed: false,
});
const emit = defineEmits<{
  'add-model': [groupName: string];
  'edit-model': [localName: string];
  'delete-model': [localName: string];
  'test-model': [localName: string];
}>();

const collapsed = ref(props.defaultCollapsed);

function colorFromName(name: string): string {
  const palette = ['#6366F1', '#14B8A6', '#F59E0B', '#EF4444', '#8B5CF6', '#06B6D4', '#EC4899', '#10B981'];
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0;
  return palette[h % palette.length];
}

function groupGlyph(name: string): string {
  const clean = name.trim();
  if (!clean) return '•';
  return clean.charAt(0).toUpperCase();
}
</script>
