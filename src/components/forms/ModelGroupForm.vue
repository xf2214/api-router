<template>
  <form class="group-form" @submit.prevent="submit">
    <div class="card card--pad group-form__section">
      <div class="section-title">{{ $t('routingTree.form.basicInfo') }}</div>
      <div class="grid-cols cols-2">
        <div class="field">
          <label class="field__label">{{ $t('routingTree.form.name') }}</label>
          <input
            v-model="form.name"
            :placeholder="$t('routingTree.form.namePlaceholder')"
            required
            :disabled="isEdit"
            class="input input--mono"
            :class="{ 'is-invalid': submitted && (!form.name.trim() || nameConflict) }"
          />
          <p v-if="isEdit" class="field__hint" style="margin-top:6px;">
            {{ $t('routingTree.form.nameReadOnlyHint') }}
          </p>
          <p v-else class="field__hint" style="margin-top:6px;">
            {{ $t('routingTree.form.nameCreateHint') }}
          </p>
          <p v-if="submitted && nameConflict" class="field__hint" style="margin-top:4px; color: var(--error);">
            {{ $t('routingTree.groupExistsError') }}
          </p>
        </div>
        <div class="field">
          <label class="field__label">{{ $t('routingTree.form.strategy') }}</label>
          <select v-model="form.strategy" class="select">
            <option value="priority">{{ $t('routing.strategy.priority') }}</option>
            <option value="weighted">{{ $t('routing.strategy.weighted') }}</option>
            <option value="round_robin">{{ $t('routing.strategy.roundRobin') }}</option>
          </select>
        </div>
      </div>

      <div class="set-row mt-16">
        <div class="set-row__txt">
          <div class="set-row__t">{{ $t('routingTree.form.fallback') }}</div>
          <div class="set-row__d">{{ $t('routingTree.form.fallbackDesc') }}</div>
        </div>
        <label class="switch" style="flex-shrink: 0;">
          <input v-model="form.fallback_enabled" type="checkbox" />
          <span class="switch__track"><span class="switch__thumb" /></span>
        </label>
      </div>
    </div>

    <!-- 成员管理：当前成员 + 可分配模型 -->
    <div class="card card--pad group-form__section">
      <div class="section-title">
        {{ $t('routingTree.form.membersManage') }}
        <span class="badge badge--info">{{ currentMembers.length }} / {{ selectedMembers.size }} + {{ currentMembers.length }}</span>
      </div>
      <p class="field__hint" style="margin-top:8px;">
        {{ $t('routingTree.form.addMemberHint') }}。
        <span v-if="!isEdit">{{ $t('routingTree.form.defaultGroupOnly') }}</span>
      </p>

      <div v-if="allModels.length === 0" class="empty empty--sm mt-12">
        <div class="empty__desc">{{ $t('routingTree.form.noModels') }}</div>
      </div>

      <div v-else class="grid-cols cols-2 mt-12" style="gap: 16px;">
        <!-- 当前成员 -->
        <div>
          <div class="eyebrow" style="margin-bottom: 8px;">
            {{ $t('routingTree.form.currentMembers') }} · {{ currentMembers.length }}
          </div>
          <div v-if="currentMembers.length === 0" class="empty empty--sm">
            <div class="empty__desc">{{ $t('routingTree.emptyGroup') }}</div>
          </div>
          <div v-else class="member-list">
            <div
              v-for="(item, i) in currentMembers"
              :key="item.local_name"
              class="member-item"
            >
              <div class="member-item__idx">{{ i + 1 }}</div>
              <div class="member-item__info">
                <span class="member-item__name">{{ item.local_name }}</span>
                <span class="member-item__meta">
                  weight={{ item.weight }}
                  <span v-if="item.orderHint === 'from-members'">· {{ $t('routingTree.priority') }}</span>
                </span>
              </div>
              <div class="grow" />
              <span class="badge badge--mute">{{ strategyLabel(form.strategy) }}</span>
            </div>
          </div>
        </div>

        <!-- 可分配模型（勾选） -->
        <div>
          <div class="eyebrow" style="margin-bottom: 8px;">
            {{ $t('routingTree.form.availableModels') }} · {{ assignableModels.length }}
          </div>
          <div v-if="assignableModels.length === 0" class="empty empty--sm">
            <div class="empty__desc">—</div>
          </div>
          <div v-else class="member-assign">
            <label
              v-for="m in assignableModels"
              :key="m.local_name"
              class="member-assign__item"
              :title="m.group || '默认'"
            >
              <input
                type="checkbox"
                :value="m.local_name"
                v-model="checkedAssignable"
                class="checkbox"
              />
              <span class="member-assign__name">{{ m.local_name }}</span>
              <span class="badge badge--mute" style="font-size: 11px; padding: 1px 6px;">
                {{ m.group || $t('routing.defaultGroup') }}
              </span>
            </label>
          </div>
        </div>
      </div>
    </div>

    <div class="row mt-24" style="justify-content: flex-end;">
      <button type="button" class="btn btn--ghost" @click="emit('cancel')">{{ $t('routingTree.form.cancel') }}</button>
      <button type="submit" class="btn btn--primary">
        {{ isEdit ? $t('routingTree.form.save') : $t('routingTree.form.create') }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ModelGroup, ModelMapping, RoutingStrategy } from '../../types';

interface Props {
  group: ModelGroup;
  models: ModelMapping[];
  existingGroupNames?: string[];
}

const props = withDefaults(defineProps<Props>(), {
  existingGroupNames: () => [],
});
const emit = defineEmits<{
  (e: 'save', group: ModelGroup): void;
  (e: 'cancel'): void;
  (e: 'move-models-to-group', localNames: string[], targetGroup: string): void;
}>();

const { t } = useI18n();

/** 编辑模式 = props.group.name 已有值；新建模式 = name 为空 */
const isEdit = computed<boolean>(() => !!props.group?.name?.trim());

/** 与当前 props.group 重名冲突（仅新建模式） */
const nameConflict = computed<boolean>(() => {
  if (isEdit.value) return false;
  const name = form.value.name.trim();
  if (!name) return false;
  return props.existingGroupNames.includes(name);
});

const defaultGroup = (): ModelGroup => ({
  name: '',
  members: [],
  strategy: 'priority',
  fallback_enabled: true,
});

const form = ref<ModelGroup>(defaultGroup());
const submitted = ref(false);
const checkedAssignable = ref<string[]>([]);

const allModels = computed(() => props.models || []);

/**
 * 当前归属到本分组的成员：
 *  - 候选：所有 model.group === 本分组名 的模型
 *  - 顺序：优先按 group.members 里显式登记的下标排，剩余按 models 数组顺序追加
 *  - 权重：查 group.members 对应项，没找到则 weight=1
 */
const currentMembers = computed<{ local_name: string; weight: number; orderHint: 'from-members' | 'default' }[]>(() => {
  const groupName = props.group?.name ?? form.value.name.trim();
  if (!groupName) return [];

  const candidates = allModels.value.filter((m) => m.group === groupName);
  const memberIdx = new Map<string, number>();
  const memberWeight = new Map<string, number>();
  form.value.members.forEach((mm, idx) => {
    memberIdx.set(mm.local_name, idx);
    memberWeight.set(mm.local_name, mm.weight);
  });

  const arranged: { local_name: string; idx: number; weight: number; orderHint: 'from-members' | 'default' }[] = [];
  const maxIdx = memberIdx.size;
  for (const c of candidates) {
    if (memberIdx.has(c.local_name)) {
      arranged.push({
        local_name: c.local_name,
        idx: memberIdx.get(c.local_name)!,
        weight: memberWeight.get(c.local_name) ?? 1,
        orderHint: 'from-members',
      });
    } else {
      arranged.push({
        local_name: c.local_name,
        idx: maxIdx + arranged.length,
        weight: 1,
        orderHint: 'default',
      });
    }
  }
  arranged.sort((a, b) => a.idx - b.idx);
  return arranged.map(({ local_name, weight, orderHint }) => ({ local_name, weight, orderHint }));
});

/** 可被分配进本分组的候选：不在本分组里的所有模型（新建模式时任何模型都可以选；编辑模式同理） */
const assignableModels = computed<ModelMapping[]>(() => {
  const groupName = isEdit.value ? props.group.name : form.value.name.trim();
  const currentSet = new Set(currentMembers.value.map((m) => m.local_name));
  const checkedSet = new Set(checkedAssignable.value);
  return allModels.value.filter((m) => {
    // 当前组的成员不再出现在可分配里
    if (currentSet.has(m.local_name)) return false;
    // 新建模式且还没填 groupName：只允许勾选（groupName 为空时也展示，但保存时需先填 name）
    // 编辑模式：任何其他组 / 默认组的模型都可勾选移入
    // 另外保留已勾选的项（避免 groupName 改变导致勾选消失）
    if (checkedSet.has(m.local_name)) return true;
    // 剩下只要不是本组的都能选
    return (m.group || '默认') !== groupName;
  });
});

const selectedMembers = computed<Set<string>>(() => new Set(checkedAssignable.value));

function strategyLabel(s: RoutingStrategy): string {
  if (s === 'priority') return t('routing.strategy.priority');
  if (s === 'weighted') return t('routing.strategy.weighted');
  return t('routing.strategy.roundRobin');
}

function submit(): void {
  submitted.value = true;
  if (!form.value.name.trim()) return;
  if (nameConflict.value) return;

  const targetGroupName = form.value.name.trim();

  // 1. 把勾选的可分配模型批量迁移到本分组
  const toMove = checkedAssignable.value.slice();
  if (toMove.length > 0) {
    // 对新建模式：保存分组后立即 emit move-models-to-group；父组件（RoutingTreePage -> App -> useModelStore）负责调用 saveModel 迁移
    // 对编辑模式：同样机制（可以把已在默认组/其他组的模型移进来）
    emit('move-models-to-group', toMove, targetGroupName);
  }

  // 2. 提交时合并当前成员到 members 里（不丢已有的权重/顺序配置）
  const existing = new Map(form.value.members.map((m) => [m.local_name, m]));
  const mergedMembers = [...form.value.members];
  for (const c of currentMembers.value) {
    if (!existing.has(c.local_name)) {
      mergedMembers.push({ local_name: c.local_name, weight: c.weight });
    }
  }
  // 另外把勾选要迁移进来的模型也加入 members（按权重 1）
  for (const n of toMove) {
    if (!existing.has(n)) {
      mergedMembers.push({ local_name: n, weight: 1 });
    }
  }

  emit('save', {
    ...form.value,
    name: targetGroupName,
    members: mergedMembers.map((m) => ({ ...m })),
  });
}

watch(
  () => props.group,
  (g) => {
    submitted.value = false;
    checkedAssignable.value = [];
    if (g) {
      form.value = {
        ...defaultGroup(),
        ...g,
        members: g.members.map((m) => ({ ...m })),
      };
    } else {
      form.value = defaultGroup();
    }
  },
  { immediate: true }
);
</script>

<style scoped>
.group-form {
  display: flex;
  flex-direction: column;
}

.group-form__section + .group-form__section {
  margin-top: 16px;
}

.empty--sm {
  padding: 32px 16px;
}

.member-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.member-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface);
}

.member-item__idx {
  flex: none;
  width: 22px;
  height: 22px;
  border-radius: 6px;
  background: var(--surface-2);
  color: var(--text-3);
  font-size: 11.5px;
  font-weight: 700;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.member-item__info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.member-item__name {
  font-family: var(--font-mono);
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.member-item__meta {
  font-size: 11.5px;
  color: var(--text-3);
  font-family: var(--font-mono);
}

/* 可分配模型列表 */
.member-assign {
  display: flex;
  flex-direction: column;
  gap: 4px;
  max-height: 360px;
  overflow-y: auto;
  padding: 4px;
  border: 1px solid var(--border);
  border-radius: var(--r-md);
  background: var(--surface);
}

.member-assign__item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 6px;
  cursor: pointer;
}

.member-assign__item:hover {
  background: var(--surface-2);
}

.member-assign__name {
  font-family: var(--font-mono);
  font-size: 13px;
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 720px) {
  .member-item,
  .member-assign__item {
    flex-wrap: wrap;
  }
}
</style>
