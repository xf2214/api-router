<template>
  <form class="model-form" @submit.prevent="submit">
    <div class="card card--pad model-form__section">
      <div class="section-title">{{ $t('routing.form.basicInfo') }}</div>
      <div class="grid-cols cols-2">
        <div class="field">
          <label class="field__label">{{ $t('routing.form.localName') }}</label>
          <input
            v-model="form.local_name"
            :disabled="isEdit"
            :placeholder="localNamePlaceholder || $t('routing.form.localNamePlaceholder')"
            required
            class="input input--mono"
            :class="{ 'is-invalid': submitted && !form.local_name }"
          />
          <p v-if="!isEdit && !form.local_name && upstreamNameHint" class="field__hint" style="margin-top:6px;">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true" style="vertical-align: -2px; margin-right: 4px; color: var(--info);">
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" /><polyline points="22 4 12 14.01 9 11.01" />
            </svg>
            {{ $t('modelForm.emptyLocalNameFallsBackTo', { fallback: upstreamNameHint }) }}
          </p>
          <p class="field__hint" v-else style="margin-top:6px;">
            {{ $t('routing.groupsManagedInTree') }}
          </p>
        </div>
        <div class="field">
          <label class="field__label">{{ $t('routing.form.group') }}</label>
          <div style="margin-top: 6px; display: flex; align-items: center; gap: 8px;">
            <span class="badge badge--mute" style="font-family: var(--font-mono); font-size: 13px; padding: 4px 10px;">
              {{ form.group || $t('routing.defaultGroup') }}
            </span>
            <span class="field__hint" style="margin: 0;">{{ $t('routing.groupsManagedInTree') }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="card card--pad model-form__section">
      <div class="section-title">{{ $t('routing.form.source') }}</div>
      <div class="seg" style="width: 100%;">
        <button
          type="button"
          class="seg__btn"
          :class="{ 'is-active': mode === 'definition' }"
          style="flex: 1;"
          @click="mode = 'definition'"
        >
          {{ $t('routing.form.refDefinition') }}
        </button>
        <button
          type="button"
          class="seg__btn"
          :class="{ 'is-active': mode === 'direct' }"
          style="flex: 1;"
          @click="mode = 'direct'"
        >
          {{ $t('routing.form.directTarget') }}
        </button>
      </div>

      <div v-if="mode === 'definition'" class="field mt-16">
        <label class="field__label">{{ $t('routing.form.selectDefinition') }}</label>
        <select v-model="form.model_id" class="select" :class="{ 'is-invalid': submitted && !form.model_id }">
          <option value="">{{ $t('routing.form.pleaseSelect') }}</option>
          <option v-for="d in safeDefinitions" :key="d.id" :value="d.id">{{ d.display_name }} ({{ d.id }})</option>
        </select>
        <span v-if="safeDefinitions.length === 0" class="field__hint" style="color: var(--warning)">
          {{ $t('routing.form.noDefinitions') }}
        </span>
      </div>

      <div v-else class="field__hint mt-12">{{ $t('routing.form.directHint') }}</div>
    </div>

    <div class="card card--pad model-form__section">
      <div class="section-title">{{ $t('routing.form.routingStrategy') }}</div>
      <div class="grid-cols cols-3">
        <div class="field">
          <label class="field__label">{{ $t('routing.form.routingStrategy') }}</label>
          <select v-model="form.strategy" class="select">
            <option value="priority">{{ $t('routing.strategy.priority') }}</option>
            <option value="weighted">{{ $t('routing.strategy.weighted') }}</option>
            <option value="round_robin">{{ $t('routing.strategy.roundRobin') }}</option>
          </select>
        </div>
      </div>
    </div>

    <div v-if="mode === 'direct'" class="card card--pad model-form__section">
      <div class="section-title">
        {{ $t('routing.form.upstreamTargets') }}
        <span class="badge badge--info">{{ form.targets.length }}</span>
      </div>

      <div
        v-for="(target, index) in form.targets"
        :key="index"
        class="card mt-12 target-block"
        style="padding: 14px 16px;"
      >
        <div class="row" style="justify-content: space-between; align-items: center;">
          <span class="eyebrow">{{ $t('routing.form.target', { index: index + 1 }) }}</span>
          <button type="button" class="btn btn--danger btn--sm" @click="removeTarget(index)">{{ $t('app.delete') }}</button>
        </div>

        <div class="grid-cols" :class="form.strategy === 'weighted' ? 'cols-4' : 'cols-3'" style="gap: 12px; margin-top: 12px;">
          <div class="field">
            <label class="field__label">{{ $t('routing.form.provider') }}</label>
            <select v-model="target.provider_id" required class="select" :class="{ 'is-invalid': submitted && !target.provider_id }">
              <option value="">{{ $t('routing.form.selectProvider') }}</option>
              <option v-for="p in safeProviders" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
          </div>
          <div class="field">
            <label class="field__label">{{ $t('routing.form.modelName') }}</label>
            <input
              v-model="target.model_name"
              :list="`model-suggestions-${index}`"
              :placeholder="$t('routing.form.modelNamePlaceholder')"
              required
              class="input input--mono"
              :class="{ 'is-invalid': submitted && !target.model_name }"
            />
            <datalist :id="`model-suggestions-${index}`">
              <option v-for="m in suggestionsFor(target.provider_id)" :key="m" :value="m" />
            </datalist>
          </div>
          <div class="field">
            <label class="field__label">{{ $t('routing.form.tier') }}</label>
            <input v-model.number="target.tier" type="number" min="1" placeholder="1" class="input" />
          </div>
          <div v-if="form.strategy === 'weighted'" class="field">
            <label class="field__label">{{ $t('routing.form.weight') }}</label>
            <input v-model.number="target.weight" type="number" min="1" placeholder="1" class="input" />
          </div>
        </div>

        <div v-if="suggestionsFor(target.provider_id).length > 0" class="row mt-12" style="flex-wrap: wrap; align-items: center;">
          <span class="field__hint">{{ $t('routing.form.quickSelect') }}</span>
          <button
            v-for="m in suggestionsFor(target.provider_id)"
            :key="m"
            type="button"
            class="badge"
            :class="target.model_name === m ? 'badge--info' : 'badge--mute'"
            @click="target.model_name = m"
          >
            {{ m }}
          </button>
        </div>

        <div class="field mt-12">
          <label class="field__label">{{ $t('routing.form.overrideParams') }}</label>
          <textarea
            v-model="overrideInputs[index]"
            rows="2"
            :placeholder="$t('routing.form.overrideParamsPlaceholder')"
            class="input input--mono"
            :class="{ 'is-invalid': submitted && overrideInputs[index] && !isValidOverride(overrideInputs[index]) }"
          />
        </div>
      </div>

      <button type="button" class="btn btn--soft mt-16" style="width: 100%;" @click="addTarget">
        {{ $t('routing.form.addTarget') }}
      </button>
    </div>

    <!-- 参数覆盖 -->
    <div class="card card--pad model-form__section">
      <button type="button" class="section-title section-title--collapse" @click="showOverridePanel = !showOverridePanel">
        <span>{{ $t('routing.form.overrideSection') }}</span>
        <span class="section-title__arrow" :class="{ 'is-open': showOverridePanel }">▾</span>
      </button>
      <div v-show="showOverridePanel" class="mt-12">
        <div class="field">
          <label class="field__label">{{ $t('routing.form.modelOverrideParams') }}</label>
          <textarea
            v-model="overrideParamsInput"
            rows="4"
            :placeholder="$t('routing.form.overrideParamsPlaceholder')"
            class="input input--mono"
            :class="{ 'is-invalid': submitted && overrideParamsInput && !isValidOverride(overrideParamsInput) }"
          />
          <span v-if="submitted && overrideParamsInput && !isValidOverride(overrideParamsInput)" class="field__hint" style="color: var(--error)">
            {{ $t('routing.form.jsonError') }}
          </span>
          <span v-else class="field__hint">{{ $t('routing.form.jsonHint') }}</span>
        </div>
      </div>
    </div>

    <!-- 重试策略 -->
    <div class="card card--pad model-form__section">
      <button type="button" class="section-title section-title--collapse" @click="showRetryPanel = !showRetryPanel">
        <span>{{ $t('routing.form.retryStrategy') }}</span>
        <span class="section-title__arrow" :class="{ 'is-open': showRetryPanel }">▾</span>
      </button>
      <div v-show="showRetryPanel" class="grid-cols cols-2 mt-12">
        <div class="field">
          <label class="field__label">{{ $t('routing.form.maxRetries') }}</label>
          <input v-model.number="form.max_retries" type="number" min="0" class="input" />
        </div>
      </div>
    </div>

    <!-- 熔断器配置 -->
    <div class="card card--pad model-form__section">
      <button type="button" class="section-title section-title--collapse" @click="showCbPanel = !showCbPanel">
        <span>{{ $t('routing.form.cbConfig') }}</span>
        <span class="section-title__arrow" :class="{ 'is-open': showCbPanel }">▾</span>
      </button>
      <div v-show="showCbPanel" class="mt-12">
        <div class="cb-panel">
          <div class="cb-panel__row">
            <div class="field">
              <label class="field__label">{{ $t('routing.form.cbFailureThreshold') }}</label>
              <input v-model.number="form.cb_config.failure_threshold" type="number" min="0" class="input input--mono" />
            </div>
            <div class="field">
              <label class="field__label">{{ $t('routing.form.cbFailurePercentage') }}</label>
              <input v-model.number="cbFailurePercentage" type="number" min="0" max="100" class="input input--mono" />
            </div>
          </div>
          <div class="cb-panel__row">
            <div class="field">
              <label class="field__label">{{ $t('routing.form.cbMinRequests') }}</label>
              <input v-model.number="form.cb_config.minimum_requests" type="number" min="0" class="input input--mono" />
            </div>
            <div class="field">
              <label class="field__label">{{ $t('routing.form.cbCooldown') }}</label>
              <input v-model.number="form.cb_config.cooldown_interval_ms" type="number" min="0" class="input input--mono" />
            </div>
          </div>
        </div>
        <div class="field mt-16">
          <label class="field__label">{{ $t('routing.form.cbStatusCodes') }}</label>
          <div class="tag-input">
            <span v-for="(code, i) in cbFailureCodes" :key="i" class="retry-code-chip">
              {{ code }}
              <span class="chip__x" @click="removeCbCode(i)">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="11" height="11"><path d="M18 6 6 18M6 6l12 12"/></svg>
              </span>
            </span>
            <input v-model="cbCodeInput" class="tag-input__input" :placeholder="$t('routing.form.cbCodePlaceholder')" @keydown.enter.prevent="addCbCode">
          </div>
          <span class="field__hint">{{ $t('routing.form.cbStatusCodesHint') }}</span>
        </div>
      </div>
    </div>

    <div class="card card--pad model-form__section">
      <div class="section-title">{{ $t('routing.form.advanced') }}</div>
      <div class="grid-cols cols-2">
        <div class="field">
          <label class="field__label">{{ $t('routing.form.contextLength') }}</label>
          <input v-model.number="form.context_length" type="number" min="0" :placeholder="$t('routing.form.contextLengthPlaceholder')" class="input" />
        </div>
        <div class="field">
          <label class="field__label">{{ $t('routing.form.maxTokens') }}</label>
          <input v-model.number="form.max_tokens" type="number" min="0" :placeholder="$t('routing.form.maxTokensPlaceholder')" class="input" />
        </div>
      </div>
    </div>

    <div v-if="testResult && mode === 'direct'" class="card card--pad model-form__section" style="background: var(--surface-2);">
      <div class="section-title">{{ $t('routing.form.testResult') }}</div>
      <div v-for="(t, i) in testResult.targets" :key="i" class="row mt-8" style="flex-wrap: wrap;">
        <span class="dot" :class="t.online ? 'dot--ok' : 'dot--err'" />
        <span class="mono">{{ providerName(t.provider_id) }} / {{ t.model_name }}</span>
        <span v-if="t.latency_ms" class="muted">({{ t.latency_ms }}ms)</span>
        <span v-if="t.error" class="badge badge--err">{{ t.error }}</span>
      </div>
    </div>

    <div class="card card--pad model-form__section" style="display: flex; align-items: center; justify-content: space-between; flex-wrap: wrap; gap: 12px;">
      <div>
        <div style="font-weight: 700; font-size: 14px; color: var(--text);">{{ $t('routing.form.fallback') }}</div>
        <div style="font-size: 12.5px; color: var(--text-3); margin-top: 2px;">{{ $t('routing.form.fallbackDesc') }}</div>
      </div>
      <label class="switch" style="flex-shrink: 0;">
        <input v-model="form.fallback_enabled" type="checkbox" />
        <span class="switch__track"><span class="switch__thumb" /></span>
      </label>
    </div>

    <div class="row mt-24" style="justify-content: flex-end;">
      <button type="button" class="btn btn--ghost" @click="emit('cancel')">{{ $t('app.cancel') }}</button>
      <button
        type="button"
        class="btn btn--soft"
        :disabled="testing || mode === 'definition' || form.targets.length === 0"
        @click="testModel"
      >
        {{ testing ? $t('app.testing') : $t('app.test') }}
      </button>
      <button
        type="submit"
        class="btn btn--primary"
        :disabled="mode === 'definition' ? !form.model_id : form.targets.length === 0"
      >
        {{ isEdit ? $t('routing.form.save') : $t('routing.form.create') }}
      </button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ModelMapping, ModelTarget, ProviderConfig, ModelDefinition, ModelTestResult, CircuitBreakerConfig } from '../../types';
import * as tauri from '../../services/tauri';

const { t } = useI18n();

interface Props {
  model?: ModelMapping | null;
  providers: ProviderConfig[];
  modelDefinitions?: ModelDefinition[];
}

const props = withDefaults(defineProps<Props>(), {
  modelDefinitions: () => [],
});
const emit = defineEmits<{
  (e: 'save', model: ModelMapping): void;
  (e: 'cancel'): void;
}>();

const isEdit = computed(() => !!props.model);

const defaultCbConfig: CircuitBreakerConfig = {
  failure_threshold: 5,
  failure_threshold_percentage: null,
  cooldown_interval_ms: 60000,
  failure_status_codes: null,
  minimum_requests: 10,
};

// 内部表单类型：cb_config 永不为 undefined（模板 v-model 不支持可选链，必须在类型层保证）
type FormModelMapping = Omit<ModelMapping, 'cb_config'> & { cb_config: CircuitBreakerConfig };

const defaultModel = (): FormModelMapping => ({
  local_name: '',
  strategy: 'priority',
  fallback_enabled: true,
  max_retries: 2,
  targets: [],
  group: '默认',
  cb_config: { ...defaultCbConfig },
});

const form = ref<FormModelMapping>(defaultModel());
const mode = ref<'definition' | 'direct'>('direct');
const testing = ref(false);
const testResult = ref<ModelTestResult | null>(null);
const submitted = ref(false);
const overrideInputs = ref<string[]>([]);
const overrideParamsInput = ref('');
const showOverridePanel = ref(false);
const showRetryPanel = ref(false);
const showCbPanel = ref(false);
const cbFailureCodes = ref<number[]>([500, 502, 503, 504]);
const cbFailurePercentage = ref(50);
const cbCodeInput = ref('');

const safeProviders = computed<ProviderConfig[]>(() =>
  Array.isArray(props.providers) ? props.providers : []
);
const safeDefinitions = computed<ModelDefinition[]>(() =>
  Array.isArray(props.modelDefinitions) ? props.modelDefinitions : []
);

const providerNameMap = computed(() => {
  const map: Record<string, string> = {};
  for (const p of safeProviders.value) map[p.id] = p.name;
  return map;
});

const providerModelsMap = computed(() => {
  const map: Record<string, string[]> = {};
  for (const p of safeProviders.value) map[p.id] = p.default_models ?? [];
  return map;
});

/** D2：根据当前已选来源推断建议的本地模型名（用于 placeholder 与提示文案） */
const upstreamNameHint = computed<string | null>(() => {
  if (mode.value === 'definition') {
    if (!form.value.model_id) return null;
    const d = safeDefinitions.value.find((x) => x.id === form.value.model_id);
    return d ? (d.display_name || d.id) : null;
  }
  const firstTarget = form.value.targets[0];
  if (!firstTarget) return null;
  if (firstTarget.model_name) return firstTarget.model_name;
  return null;
});

/** D2：placeholder（用于 input 的占位符展示，不自动改值） */
const localNamePlaceholder = computed<string>(() => '');

function providerName(id: string) {
  return providerNameMap.value[id] ?? id;
}

function suggestionsFor(providerId: string): string[] {
  if (!providerId) return [];
  return providerModelsMap.value[providerId] ?? [];
}

function stringifyOverride(params?: Record<string, any>): string {
  if (!params || Object.keys(params).length === 0) return '';
  try {
    return JSON.stringify(params, null, 2);
  } catch {
    return '';
  }
}

function parseOverride(text: string): Record<string, any> | undefined {
  const trimmed = text.trim();
  if (!trimmed) return undefined;
  try {
    return JSON.parse(trimmed) as Record<string, any>;
  } catch {
    return undefined;
  }
}

function isValidOverride(text: string): boolean {
  const trimmed = text.trim();
  if (!trimmed) return true;
  try {
    JSON.parse(trimmed);
    return true;
  } catch {
    return false;
  }
}

function buildTargets(): ModelTarget[] {
  return form.value.targets.map((t, i) => ({
    ...t,
    override_params: parseOverride(overrideInputs.value[i]),
  }));
}

function buildCbConfig(): CircuitBreakerConfig {
  return {
    ...(form.value.cb_config ?? defaultCbConfig),
    failure_threshold_percentage: cbFailurePercentage.value,
    failure_status_codes: cbFailureCodes.value.length > 0 ? [...cbFailureCodes.value] : null,
  };
}

function addCbCode(): void {
  const raw = cbCodeInput.value.trim();
  if (!raw) return;
  const c = Number(raw);
  if (Number.isNaN(c) || c <= 0) return;
  if (!cbFailureCodes.value.includes(c)) cbFailureCodes.value.push(c);
  cbCodeInput.value = '';
}

function removeCbCode(index: number): void {
  cbFailureCodes.value.splice(index, 1);
}

/**
 * 一次性重置表单辅助元数据（非核心 form 字段），
 * 避免散落在 watch 内造成响应式连锁赋值。
 */
function resetFormMeta(targetsCopy: ModelTarget[], overrideParams?: Record<string, any>, baseCb?: CircuitBreakerConfig): void {
  overrideInputs.value = targetsCopy.map((t) => stringifyOverride(t.override_params));
  overrideParamsInput.value = stringifyOverride(overrideParams);
  const cb = baseCb ?? { ...defaultCbConfig };
  cbFailureCodes.value = Array.isArray(cb.failure_status_codes)
    ? [...cb.failure_status_codes]
    : [500, 502, 503, 504];
  cbFailurePercentage.value = cb.failure_threshold_percentage ?? 50;
  testResult.value = null;
}

watch(
  () => props.model,
  (m) => {
    submitted.value = false;
    if (m) {
      // ========== 编辑模式 ==========
      const baseCb: CircuitBreakerConfig = {
        ...defaultCbConfig,
        ...(m.cb_config || {}),
      };
      const targetsCopy = m.targets.map((t) => ({ ...t }));

      mode.value = m.model_id ? 'definition' : 'direct';
      form.value = {
        ...defaultModel(),
        ...m,
        targets: targetsCopy,
        cb_config: baseCb,
      };

      resetFormMeta(targetsCopy, m.override_params, baseCb);
    } else {
      // ========== 新建模式 ==========
      const baseModel = defaultModel();
      mode.value = 'direct';
      form.value = baseModel;
      resetFormMeta(baseModel.targets, baseModel.override_params, baseModel.cb_config);
    }
  },
  { immediate: true }
);

/**
 * D2：自动回填本地模型名
 *  - 新建模式 + 用户未手动输入过 local_name 时：
 *    若切换到 definition 模式并选了 model_id，则本地名回填为定义的 display_name/id
 *    若 direct 模式下首个 target.model_name 改变，则本地名回填为该名称
 */
let _lastLocalNameTouched = false;
watch(
  () => form.value.local_name,
  (v) => {
    if (v && v.trim()) _lastLocalNameTouched = true;
  }
);
function maybeBackfillLocalName(): void {
  if (isEdit.value) return;
  if (_lastLocalNameTouched) return;
  if (form.value.local_name && form.value.local_name.trim()) {
    _lastLocalNameTouched = true;
    return;
  }
  const hint = upstreamNameHint.value;
  if (!hint) return;
  form.value.local_name = hint;
  // 回填视为未手动修改过
  _lastLocalNameTouched = false;
}
watch(
  () => [form.value.targets[0]?.model_name, form.value.model_id, mode.value] as const,
  () => maybeBackfillLocalName(),
  { immediate: true }
);

function addTarget(): void {
  form.value.targets.push({ provider_id: '', model_name: '', weight: 1, tier: 1 });
  overrideInputs.value.push('');
}

function removeTarget(index: number): void {
  form.value.targets.splice(index, 1);
  overrideInputs.value.splice(index, 1);
}

async function testModel(): Promise<void> {
  if (mode.value === 'definition') return;
  testing.value = true;
  try {
    if (overrideInputs.value.some((o) => o && !isValidOverride(o))) {
      throw new Error(t('routing.form.configError'));
    }
    if (overrideParamsInput.value && !isValidOverride(overrideParamsInput.value)) {
      throw new Error(t('routing.form.modelConfigError'));
    }
    const model: ModelMapping = {
      ...form.value,
      targets: buildTargets(),
      override_params: parseOverride(overrideParamsInput.value),
      cb_config: buildCbConfig(),
    };
    testResult.value = await tauri.testModelConfig(model);
  } catch (e) {
    testResult.value = {
      local_name: form.value.local_name,
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
    testing.value = false;
  }
}

function submit(): void {
  submitted.value = true;
  // 本地名没写时，自动用上游名称（定义名 或 首个 target.model_name）
  if (!form.value.local_name.trim()) {
    const fallback = upstreamNameHint.value;
    if (fallback) form.value.local_name = fallback;
  }
  if (!form.value.local_name.trim()) return;
  if (mode.value === 'definition') {
    if (!form.value.model_id) return;
  } else {
    for (const t of form.value.targets) {
      if (!t.provider_id || !t.model_name) return;
    }
  }
  if (overrideInputs.value.some((o) => o && !isValidOverride(o))) return;
  if (overrideParamsInput.value && !isValidOverride(overrideParamsInput.value)) return;

  const isDefinition = mode.value === 'definition';
  emit('save', {
    ...form.value,
    local_name: form.value.local_name.trim(),
    model_id: isDefinition ? form.value.model_id : undefined,
    targets: isDefinition ? [] : buildTargets(),
    override_params: parseOverride(overrideParamsInput.value),
    cb_config: buildCbConfig(),
  });
}
</script>

<style scoped>
.model-form {
  display: flex;
  flex-direction: column;
}

.model-form__section + .model-form__section {
  margin-top: 16px;
}

.target-block {
  background: var(--surface);
}

.section-title--collapse {
  width: 100%;
  justify-content: space-between;
  cursor: pointer;
  background: none;
  border: none;
  padding: 0;
  margin: 0;
  font: inherit;
  color: inherit;
}

.section-title__arrow {
  color: var(--text-3);
  transition: transform 0.2s;
}

.section-title__arrow.is-open {
  transform: rotate(180deg);
}
</style>
