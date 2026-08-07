<template>
  <form class="model-definition-form" @submit.prevent="submit">
    <div class="card card--pad model-definition-form__section">
      <div class="section-title">{{ $t('models.definitionForm.title') }}</div>
      <div class="grid-cols cols-2">
        <div class="field">
          <label class="field__label">{{ $t('models.definitionForm.modelId') }}</label>
          <input
            v-model="form.id"
            :disabled="isEdit"
            :placeholder="$t('models.definitionForm.modelIdPlaceholder')"
            required
            class="input input--mono"
            :class="{ 'is-invalid': submitted && !form.id.trim() }"
          />
          <span class="field__hint">{{ $t('models.definitionForm.modelIdHint') }}</span>
        </div>
        <div class="field">
          <label class="field__label">{{ $t('models.definitionForm.displayName') }}</label>
          <input
            v-model="form.display_name"
            :placeholder="$t('models.definitionForm.displayNamePlaceholder')"
            required
            class="input"
            :class="{ 'is-invalid': submitted && !form.display_name.trim() }"
          />
        </div>
        <div class="field">
          <label class="field__label">{{ $t('models.definitionForm.contextLength') }}</label>
          <input
            v-model.number="form.context_length"
            type="number"
            min="0"
            placeholder="例如 128000"
            class="input input--mono"
          />
        </div>
        <div class="field">
          <label class="field__label">{{ $t('models.definitionForm.maxTokens') }}</label>
          <input
            v-model.number="form.max_tokens"
            type="number"
            min="0"
            placeholder="例如 4096"
            class="input input--mono"
          />
        </div>
      </div>

      <div class="set-row mt-16">
        <div class="set-row__txt">
          <div class="set-row__t">自动聚合</div>
          <div class="set-row__d">开启后把多个接入点聚合为单一模型，否则每个接入点独立展示</div>
        </div>
        <label class="switch" style="flex-shrink: 0;">
          <input v-model="form.auto_aggregate" type="checkbox" />
          <span class="switch__track"><span class="switch__thumb" /></span>
        </label>
      </div>
    </div>

    <div class="row mt-24" style="justify-content: flex-end;">
      <button type="button" class="btn btn--ghost" @click="emit('cancel')">{{ $t('app.cancel') }}</button>
      <button type="submit" class="btn btn--primary">
        {{ isEdit ? $t('models.definitionForm.save') : $t('models.definitionForm.create') }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import type { ModelDefinition } from '../../types';

interface Props {
  definition?: ModelDefinition | null;
}

const props = withDefaults(defineProps<Props>(), {
  definition: null,
});

const emit = defineEmits<{
  (e: 'save', def: ModelDefinition): void;
  (e: 'cancel'): void;
}>();

const isEdit = computed(() => !!props.definition?.id);

const defaultDef = (): ModelDefinition => ({
  id: '',
  display_name: '',
  context_length: undefined,
  max_tokens: undefined,
  auto_aggregate: true,
  access_points: [],
});

const form = ref<ModelDefinition>(defaultDef());
const submitted = ref(false);

watch(
  () => props.definition,
  (d) => {
    submitted.value = false;
    if (d) {
      form.value = {
        ...defaultDef(),
        ...d,
        access_points: d.access_points ? [...d.access_points] : [],
      };
    } else {
      form.value = defaultDef();
    }
  },
  { immediate: true }
);

function submit(): void {
  submitted.value = true;
  if (!form.value.id.trim() || !form.value.display_name.trim()) return;

  emit('save', {
    ...form.value,
    id: form.value.id.trim(),
    display_name: form.value.display_name.trim(),
    context_length: form.value.context_length || undefined,
    max_tokens: form.value.max_tokens || undefined,
    access_points: form.value.access_points ?? [],
  });
}
</script>

<style scoped>
.model-definition-form {
  display: flex;
  flex-direction: column;
}

.model-definition-form__section + .model-definition-form__section {
  margin-top: 16px;
}
</style>
