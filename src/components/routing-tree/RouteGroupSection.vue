<template>
  <section class="route-group-v2">
    <!-- 头部：分组标识 + 名称 + 徽章 + 操作（折叠/编辑/删除） -->
    <header class="route-group-v2__head">
      <span class="pmark" :style="{ background: colorFromName(displayName) }">{{ groupGlyph(displayName) }}</span>
      <span class="route-group-v2__name">{{ displayName }}</span>
      <div class="route-group-v2__badges">
        <span class="badge" :class="strategyBadgeClass(group.strategy)" style="font-size: 11px; padding: 2px 10px;">
          {{ strategyLabel(group.strategy) }}
        </span>
        <span class="badge" :class="group.fallback_enabled ? 'badge--ok' : 'badge--warn'" style="font-size: 11px; padding: 2px 10px;">
          {{ group.fallback_enabled ? $t('routingTree.enabled') : $t('routingTree.disabled') }}
        </span>
        <span class="badge badge--mute" style="font-size: 11px; padding: 2px 10px;">
          {{ $t('routing.modelCount', { count: models.length }) }}
        </span>
      </div>
      <div class="grow" />
      <button
        type="button"
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
      <div class="menu-wrap" @click.stop>
        <button
          type="button"
          class="menu-btn menu-btn--sm"
          :title="$t('routing.moreActions')"
          @click.stop="openMenu = !openMenu"
        >
          <svg viewBox="0 0 24 24" fill="currentColor" width="14" height="14">
            <circle cx="12" cy="5" r="1.8"/>
            <circle cx="12" cy="12" r="1.8"/>
            <circle cx="12" cy="19" r="1.8"/>
          </svg>
        </button>
        <div v-if="openMenu" class="menu-dropdown" @click.stop>
          <button type="button" class="menu-item" @click="onEditGroup">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13">
              <path d="M12 20h9"/><path d="M16.5 3.5a2.12 2.12 0 0 1 3 3L7 19l-4 1 1-4Z"/>
            </svg>
            {{ $t('routingTree.edit') }}
          </button>
          <div class="menu-divider" />
          <button type="button" class="menu-item menu-item--danger" @click="onDeleteGroup">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="13" height="13">
              <path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
            {{ $t('app.delete') }}
          </button>
        </div>
      </div>
    </header>

    <!-- 体：模型节点列表 / 空分组引导 -->
    <div v-if="!collapsed" class="route-group-v2__body">
      <div v-if="models.length === 0" class="empty" style="padding: 32px 20px; gap: 10px;">
        <div class="empty__icon" style="background: var(--primary-soft); color: var(--primary); width: 48px; height: 48px; border-radius: 14px;">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="24" height="24">
            <circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="6" r="2.5"/><circle cx="12" cy="18" r="2.5"/><path d="M8.2 7.2 16 16"/>
          </svg>
        </div>
        <div class="empty__title" style="font-size: 14px;">
          {{ $t('routingTree.emptyGroup') }}
        </div>
        <div class="empty__desc" style="font-size: 12px; color: var(--text-3);">
          {{ $t('routing.emptyGroupDesc') }}
        </div>
      </div>

      <RouteModelNode
        v-for="(n, i) in models"
        :key="n.key"
        :node="n"
        :index="i"
        :total="models.length"
        :providers="providers"
        @jump-and-edit="emit('jump-and-edit-node', n)"
        @delete="emit('delete-node', n)"
        @move-up="reorderNode(n, 'up')"
        @move-down="reorderNode(n, 'down')"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ModelGroup, RouteModelNodeV2 } from '../../types/routing';
import type { ProviderConfig } from '../../types/config';
import RouteModelNode from './RouteModelNode.vue';
import { strategyLabel, strategyBadgeClass } from '../../utils/format';

interface Props {
  group: ModelGroup;
  models: RouteModelNodeV2[];
  providers: ProviderConfig[];
  defaultCollapsed?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  defaultCollapsed: false,
});

const emit = defineEmits<{
  (e: 'edit-group'): void;
  (e: 'delete-group'): void;
  (e: 'jump-and-edit-node', node: RouteModelNodeV2): void;
  (e: 'delete-node', node: RouteModelNodeV2): void;
  (e: 'reorder-member', memberKey: string, direction: 'up' | 'down'): void;
}>();

const { t } = useI18n();
const collapsed = ref(props.defaultCollapsed);
const openMenu = ref<boolean>(false);

const displayName = computed(() => props.group.name || t('routing.defaultGroup', '默认'));

function colorFromName(name: string): string {
  const palette = ['#6366F1', '#14B8A6', '#F59E0B', '#EF4444', '#8B5CF6', '#06B6D4', '#EC4899', '#10B981'];
  let h = 0;
  for (let i = 0; i < name.length; i++) h = (h * 31 + name.charCodeAt(i)) >>> 0;
  return palette[h % palette.length];
}

function groupGlyph(name: string): string {
  const clean = name.trim();
  if (!clean) return '\u2022';
  return clean.charAt(0).toUpperCase();
}

function onEditGroup(): void {
  openMenu.value = false;
  emit('edit-group');
}

function onDeleteGroup(): void {
  openMenu.value = false;
  emit('delete-group');
}

function reorderNode(node: RouteModelNodeV2, direction: 'up' | 'down'): void {
  emit('reorder-member', node.localName, direction);
}

/* ---------- ⋮ 菜单：点击外部 / ESC 关闭 ---------- */
function closeMenu(e: MouseEvent): void {
  if (!openMenu.value) return;
  const target = e.target as HTMLElement;
  const el = target.closest?.('.menu-wrap');
  if (!el) openMenu.value = false;
}
function onEsc(e: KeyboardEvent): void {
  if (e.key === 'Escape') openMenu.value = false;
}
onMounted(() => {
  document.addEventListener('click', closeMenu);
  document.addEventListener('keydown', onEsc);
});
onBeforeUnmount(() => {
  document.removeEventListener('click', closeMenu);
  document.removeEventListener('keydown', onEsc);
});
</script>
