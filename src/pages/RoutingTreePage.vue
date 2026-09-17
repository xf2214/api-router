<template>
  <section class="page">
    <header class="ph">
      <div>
        <div class="ph__title">{{ $t('routingTree.title') }}</div>
        <div class="ph__desc">{{ $t('routingTree.desc') }}</div>
      </div>
      <div class="ph__actions">
        <button class="btn btn--primary btn--sm" @click="startAddGroup">{{ $t('routingTree.addGroup') }}</button>
      </div>
    </header>

    <RoutingTreeToolbar />

    <!-- 分组编辑表单（同时支持新建与编辑） -->
    <div v-if="editingGroup != null" class="card card--pad" style="margin-top:16px;">
      <ModelGroupForm
        :group="editingGroup"
        :models="props.config.models"
        :existing-group-names="existingGroupNames"
        @save="onSaveGroup"
        @cancel="editingGroup = undefined"
        @move-models-to-group="(names, target) => emit('move-models-to-group', names, target)"
      />
    </div>

    <!-- 路由树 -->
    <div v-else style="margin-top:16px; display:flex; flex-direction:column; gap:12px;">
      <!-- 空状态：三步引导 + CTA -->
      <div v-if="loading && tree.length === 0" class="empty" style="padding: 56px 24px;">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" width="28" height="28" class="spin-anim" style="transform-origin: center; color: var(--primary);">
          <path d="M21 12a9 9 0 1 1-6.2-8.55"/>
        </svg>
        <div class="empty__desc">{{ $t('logs.refreshing') }}</div>
      </div>
      <div v-else-if="tree.length === 0" class="empty" style="padding: 56px 24px;">
        <div class="empty__icon" style="background: var(--primary-soft); color: var(--primary);">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="30" height="30">
            <path d="M3 6h18M3 12h18M3 18h18"/>
            <path d="M7 3v6M12 3v6M17 3v6M7 15v6M12 15v6M17 15v6"/>
          </svg>
        </div>
        <div class="empty__title">
          {{ $t('routingTree.noGroups') }}
        </div>
        <div class="empty__desc">
          {{ $t('routingTree.noGroupsDesc') }}
        </div>
        <button class="btn btn--primary btn--sm" @click="startAddGroup">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="14" height="14"><path d="M12 5v14M5 12h14"/></svg>
          {{ $t('routingTree.addGroup') }}
        </button>

        <div class="empty-guide" style="width: 100%; max-width: 720px; margin-top: 18px;">
          <div class="empty-step">
            <span class="empty-step__n">1</span>
            <div class="empty-step__t">{{ $t('routingTree.step1T') }}</div>
            <div class="empty-step__d">{{ $t('routingTree.step1D') }}</div>
          </div>
          <div class="empty-step">
            <span class="empty-step__n">2</span>
            <div class="empty-step__t">{{ $t('routingTree.step2T') }}</div>
            <div class="empty-step__d">{{ $t('routingTree.step2D') }}</div>
          </div>
          <div class="empty-step">
            <span class="empty-step__n">3</span>
            <div class="empty-step__t">{{ $t('routingTree.step3T') }}</div>
            <div class="empty-step__d">{{ $t('routingTree.step3D') }}</div>
          </div>
        </div>
      </div>
      <RouteGroupSection
        v-for="item in tree"
        :key="item.key"
        :group="resolveGroup(item.key)"
        :models="item.models"
        :providers="props.config.providers"
        @edit-group="editGroupByKey(item.key)"
        @delete-group="onDeleteGroup(item.key)"
        @jump-and-edit-node="(n) => emit('jump-and-edit-model', n.localName)"
        @delete-node="onDeleteNode"
        @reorder-member="(key, dir) => onReorderByGroupKey(item.key, key, dir)"
      />
    </div>
  </section>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { AppConfig, ModelGroup, ModelDefinition, ModelMapping, RoutingStrategy, AccessPoint, ModelTarget, RouteGroupTree, RouteModelNodeV2, RouteTargetLeaf } from '../types';
import RoutingTreeToolbar from '../components/routing-tree/RoutingTreeToolbar.vue';
import RouteGroupSection from '../components/routing-tree/RouteGroupSection.vue';
import ModelGroupForm from '../components/forms/ModelGroupForm.vue';

interface Props {
  config: AppConfig;
  modelDefinitions: ModelDefinition[];
  groups: ModelGroup[];
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
});
const emit = defineEmits<{
  (e: 'save-group', group: ModelGroup): void;
  (e: 'delete-group', name: string): void;
  (e: 'save-definition', def: ModelDefinition): void;
  (e: 'delete-definition', id: string): void;
  (e: 'show-message', text: string, type: 'success' | 'error' | 'warn' | 'info'): void;
  (e: 'update-model-group', localName: string, newGroup: string): void;
  (e: 'move-models-to-group', localNames: string[], targetGroup: string): void;
  (e: 'jump-and-edit-model', localName: string): void;
}>();

const { t } = useI18n();

const editingGroup = ref<ModelGroup | null | undefined>(undefined);

/** 现有分组名，新建时用于重名校验 */
const existingGroupNames = computed<string[]>(() =>
  props.groups.map((g) => g.name)
);

function providerNameById(id: string): string {
  return props.config.providers.find((p) => p.id === id)?.name || id;
}

function providerEnabledById(id: string): boolean {
  return props.config.providers.find((p) => p.id === id)?.enabled ?? false;
}

function buildTargetsFromMapping(mapping: ModelMapping): RouteTargetLeaf[] {
  if (mapping.model_id) {
    const def = props.modelDefinitions.find((d) => d.id === mapping.model_id);
    return (def?.access_points ?? [])
      .filter((a: AccessPoint) => a.enabled)
      .map((a: AccessPoint) => ({
        key: `${a.provider_id}::${a.upstream_model_name}::tier1`,
        providerId: a.provider_id,
        providerName: providerNameById(a.provider_id),
        upstreamModel: a.upstream_model_name,
        weight: a.weight ?? 1,
        tier: 1,
        enabled: providerEnabledById(a.provider_id),
      }));
  }
  return mapping.targets.map((tg: ModelTarget) => ({
    key: `${tg.provider_id}::${tg.model_name}::tier${tg.tier ?? 1}`,
    providerId: tg.provider_id,
    providerName: providerNameById(tg.provider_id),
    upstreamModel: tg.model_name,
    weight: tg.weight ?? 1,
    tier: tg.tier ?? 1,
    enabled: providerEnabledById(tg.provider_id),
  }));
}

function buildGroupTree(group: ModelGroup): RouteGroupTree {
  // 1. 候选成员 = 所有配置了 group === group.name 的模型映射
  const candidates = props.config.models.filter((m) => m.group === group.name);

  // 2. 从 group.members 里抽取已配置的顺序下标和权重覆盖
  const memberOrder = new Map<string, number>();
  const memberWeight = new Map<string, number>();
  for (let i = 0; i < group.members.length; i++) {
    const mm = group.members[i];
    memberOrder.set(mm.local_name, i);
    memberWeight.set(mm.local_name, mm.weight);
  }

  // 3. 组装带顺序/权重信息的中间数组并排序
  const arranged: { mapping: ModelMapping; sortKey: number; weight: number }[] = [];
  const configuredCount = memberOrder.size;
  for (const mapping of candidates) {
    let sortKey: number;
    let weight: number;
    if (memberOrder.has(mapping.local_name)) {
      sortKey = memberOrder.get(mapping.local_name)!;
      weight = memberWeight.get(mapping.local_name) ?? 1;
    } else {
      sortKey = configuredCount + arranged.length;
      weight = 1;
    }
    arranged.push({ mapping, sortKey, weight });
  }
  arranged.sort((a, b) => a.sortKey - b.sortKey);

  // 4. 构造路由树节点
  const models: RouteModelNodeV2[] = arranged.map(({ mapping, weight: memberW }) => {
    let displayLabel: string = mapping.local_name;
    const strategy: RoutingStrategy = mapping.strategy ?? 'priority';
    let targets: RouteTargetLeaf[] = [];
    let hasModelDefinition = false;
    let mappingModelId: string | undefined = undefined;

    if (mapping.model_id) {
      const def = props.modelDefinitions.find((d) => d.id === mapping.model_id);
      displayLabel = def?.display_name || mapping.model_id;
      hasModelDefinition = true;
      mappingModelId = mapping.model_id;
    } else {
      displayLabel = mapping.local_name;
    }
    targets = buildTargetsFromMapping(mapping);

    return {
      key: mapping.local_name,
      localName: mapping.local_name,
      displayLabel,
      strategy,
      memberWeight: memberW ?? 1,
      targets,
      hasModelDefinition,
      mappingModelId,
    };
  });

  return {
    key: group.name,
    groupName: group.name,
    strategy: group.strategy,
    fallbackEnabled: group.fallback_enabled,
    models,
  };
}

const tree = computed<RouteGroupTree[]>(() =>
  props.groups.map(buildGroupTree)
);

function resolveGroup(key: string): ModelGroup {
  return props.groups.find((g) => g.name === key)!;
}

function editGroupByKey(key: string): void {
  const g = resolveGroup(key);
  editingGroup.value = { ...g, members: g.members.map((m) => ({ ...m })) };
}

function onSaveGroup(g: ModelGroup): void {
  emit('save-group', g);
  editingGroup.value = undefined;
}

function onDeleteGroup(name: string): void {
  emit('delete-group', name);
}

function startAddGroup(): void {
  editingGroup.value = {
    name: '',
    members: [],
    strategy: 'priority',
    fallback_enabled: true,
  };
}

function onDeleteNode(node: RouteModelNodeV2): void {
  const owned = props.groups.some((g) => g.name === resolveOwnerGroupName(node.localName));
  if (!owned) {
    emit('show-message', t('routingTree.noGroupForNode'), 'warn');
    return;
  }
  const msg = `将「${node.displayLabel}」从分组移出（模型本身不会删除，会放回「默认」分组）。确定继续？`;
  if (!confirm(msg)) return;
  emit('update-model-group', node.localName, '默认');
}

/** 返回某个 local_name 归属的分组名（按 ModelMapping.group 字段决定） */
function resolveOwnerGroupName(localName: string): string | null {
  const m = props.config.models.find((x) => x.local_name === localName);
  return m?.group ?? null;
}

function onReorderByGroupKey(
  groupKey: string,
  memberKey: string,
  direction: 'up' | 'down'
): void {
  const group = resolveGroup(groupKey);
  // 1. 当前本分组实际展示顺序 = buildGroupTree 的顺序重算
  const displayed: string[] = buildGroupTree(group).models.map((n) => n.localName);
  const i = displayed.findIndex((n) => n === memberKey);
  if (i < 0) return;
  const j = direction === 'up' ? i - 1 : i + 1;
  if (j < 0 || j >= displayed.length) return;
  // 2. 交换 displayed[i] <-> displayed[j]
  const newDisplayed = [...displayed];
  const tmp = newDisplayed[i];
  newDisplayed[i] = newDisplayed[j];
  newDisplayed[j] = tmp;
  // 3. 按新顺序构造新的 group.members，保留已有的 weight 覆盖值
  const weightMap = new Map<string, number>();
  for (const m of group.members) weightMap.set(m.local_name, m.weight);
  const newMembers = newDisplayed.map((localName) => ({
    local_name: localName,
    weight: weightMap.get(localName) ?? 1,
  }));
  emit('save-group', { ...group, members: newMembers });
}
</script>
