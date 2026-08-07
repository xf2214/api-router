<template>
  <div class="access-point-list">
    <div class="section-title">
      {{ $t('models.accessPointList.title') }}
      <span class="badge badge--info">{{ $t('models.accessPointList.count', { count: accessPoints.length }) }}</span>
    </div>

    <div class="card card--flush">
      <div class="table-wrap">
        <table class="table">
          <thead>
            <tr>
              <th>{{ $t('models.accessPointList.provider') }}</th>
              <th>{{ $t('models.accessPointList.modelName') }}</th>
              <th>{{ $t('models.accessPointList.enabled') }}</th>
              <th>{{ $t('models.accessPointList.weight') }}</th>
              <th class="right">{{ $t('models.actions') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="(ap, index) in localPoints" :key="index">
              <td>
                <select v-model="ap.provider_id" class="select">
                  <option value="">{{ $t('models.accessPointList.selectProvider') }}</option>
                  <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.name }}</option>
                </select>
              </td>
              <td>
                <input
                  v-model="ap.upstream_model_name"
                  placeholder="例如 gpt-4o"
                  class="input input--mono"
                />
              </td>
              <td>
                <label class="switch">
                  <input v-model="ap.enabled" type="checkbox" />
                  <span class="switch__track"><span class="switch__thumb" /></span>
                </label>
              </td>
              <td style="width: 110px;">
                <input v-model.number="ap.weight" type="number" min="0" class="input input--mono" />
              </td>
              <td class="right">
                <button type="button" class="btn btn--danger btn--sm" @click="remove(index)">{{ $t('models.delete') }}</button>
              </td>
            </tr>
            <tr v-if="localPoints.length === 0">
              <td colspan="5">
                <div class="empty" style="padding: 32px 24px;">
                  <div class="empty__desc">{{ $t('models.accessPointList.noAccessPoints') }}</div>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="add-row mt-12">
      <select v-model="newProviderId" class="select" style="width: 180px;">
        <option value="">{{ $t('models.accessPointList.selectProvider') }}</option>
        <option v-for="p in providers" :key="p.id" :value="p.id">{{ p.name }}</option>
      </select>
      <input
        v-model="newUpstreamModelName"
        :placeholder="$t('models.accessPointList.addHint')"
        class="input input--mono"
        style="flex: 1; min-width: 140px;"
      />
      <button type="button" class="btn btn--soft" :disabled="!canAdd" @click="add">
        {{ $t('models.accessPointList.add') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import type { AccessPoint, ProviderConfig } from '../types';

interface Props {
  accessPoints: AccessPoint[];
  providers: ProviderConfig[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'update', accessPoints: AccessPoint[]): void;
}>();

const localPoints = ref<AccessPoint[]>([]);
const newProviderId = ref('');
const newUpstreamModelName = ref('');
const updatingFromProps = ref(false);

const canAdd = computed(() => newProviderId.value.trim() && newUpstreamModelName.value.trim());

// 初始化手动同步一次（避免 immediate watch 触发循环）
localPoints.value = (props.accessPoints ?? []).map((ap) => ({ ...ap }));

// Watch 1: props.accessPoints 引用变化时同步到 localPoints
// 注意：仅监听引用变化（不移除 deep 为避免深层对象复用时遗漏，但加标志位防回流）
watch(
  () => props.accessPoints,
  (newList, oldList) => {
    if (updatingFromProps.value) return;
    if (newList === oldList) return;
    updatingFromProps.value = true;
    try {
      localPoints.value = (newList ?? []).map((ap) => ({ ...ap }));
    } finally {
      nextTick(() => {
        updatingFromProps.value = false;
      });
    }
  }
);

// Watch 2: localPoints 任何变化 → emit 到父组件
watch(
  localPoints,
  () => {
    updatingFromProps.value = true;
    try {
      emit('update', localPoints.value.map((ap) => ({ ...ap })));
    } finally {
      nextTick(() => {
        updatingFromProps.value = false;
      });
    }
  },
  { deep: true }
);

function add(): void {
  if (!canAdd.value) return;
  localPoints.value.push({
    provider_id: newProviderId.value,
    upstream_model_name: newUpstreamModelName.value.trim(),
    enabled: true,
    weight: 1,
  });
  newProviderId.value = '';
  newUpstreamModelName.value = '';
}

function remove(index: number): void {
  localPoints.value.splice(index, 1);
}
</script>

<style scoped>
.access-point-list {
  display: flex;
  flex-direction: column;
}

.add-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.add-row .select,
.add-row .input {
  height: 36px;
}
</style>
