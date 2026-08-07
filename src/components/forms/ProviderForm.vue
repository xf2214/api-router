<template>
  <form class="provider-form" @submit.prevent="submit">
    <div class="modal__body">
      <div class="seg">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          :class="['seg__btn', { 'is-active': activeTab === tab.key }]"
          @click="activeTab = tab.key"
        >
          {{ tab.label }}
        </button>
      </div>

      <div v-if="activeTab === 'basic'" class="provider-form__tab">
        <div v-if="!isEdit" class="field">
          <label class="field__label">{{ $t('provider.selectPreset') }}</label>
          <select v-model="selectedPresetId" class="select" @change="applyPreset">
            <option value="">{{ $t('provider.manual') }}</option>
            <optgroup v-for="g in presetGroups" :key="g.category" :label="g.label">
              <option v-for="p in g.presets" :key="p.id" :value="p.id">{{ p.name }}</option>
            </optgroup>
          </select>
          <span class="field__hint">{{ $t('provider.presetHint') }}</span>
        </div>

        <div class="field">
          <label class="field__label">{{ $t('provider.id') }}</label>
          <input
            v-model="form.id"
            :disabled="isEdit"
            :placeholder="$t('provider.idPlaceholder')"
            required
            class="input input--mono"
            :class="{ 'is-invalid': submitted && !form.id }"
          />
        </div>

        <div class="field">
          <label class="field__label">{{ $t('provider.name') }}</label>
          <input
            v-model="form.name"
            :placeholder="$t('provider.namePlaceholder')"
            required
            class="input"
            :class="{ 'is-invalid': submitted && !form.name }"
          />
        </div>

        <div class="field">
          <label class="field__label">{{ $t('provider.type') }}</label>
          <select v-model="form.provider_type" class="select">
            <option value="openai_compatible">{{ $t('provider.typeOpenAI') }}</option>
            <option value="custom_template">{{ $t('provider.typeCustom') }}</option>
          </select>
          <span v-if="form.provider_type === 'custom_template'" class="field__hint" style="color: var(--warning)">
            {{ $t('provider.typeCustomHint') }}
          </span>
        </div>

        <div class="field">
          <label class="field__label">{{ $t('provider.baseUrl') }}</label>
          <div class="row">
            <input
              v-model="form.base_url"
              :placeholder="$t('provider.baseUrlPlaceholder')"
              required
              class="input input--mono grow"
              :class="{ 'is-invalid': submitted && !form.base_url }"
              @blur="form.base_url = normalizeBaseUrl(form.base_url)"
            />
            <button type="button" class="btn btn--ghost btn--icon" :title="$t('app.copy') + ' Base URL'" @click="copyBaseUrl">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            </button>
            <button v-if="!isEdit" type="button" class="btn btn--ghost btn--sm" @click="resetPreset">{{ $t('provider.reset') }}</button>
          </div>
        </div>

        <div class="field">
          <label class="field__label">{{ $t('provider.apiKey') }}</label>
          <div class="row">
            <input
              v-model="apiKey"
              :type="apiKeyVisible ? 'text' : 'password'"
              :placeholder="$t('provider.apiKeyPlaceholder')"
              class="input input--mono grow"
            />
            <button
              type="button"
              class="btn btn--ghost btn--icon"
              :title="apiKeyVisible ? $t('app.hidden') : $t('app.visible')"
              @click="apiKeyVisible = !apiKeyVisible"
            >
              <svg
                v-if="apiKeyVisible"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
                <circle cx="12" cy="12" r="3" />
                <line x1="4" y1="4" x2="20" y2="20" />
              </svg>
              <svg
                v-else
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7S2 12 2 12z" />
                <circle cx="12" cy="12" r="3" />
              </svg>
            </button>
            <button type="button" class="btn btn--ghost btn--icon" :title="$t('app.copy')" @click="copyApiKey">
              <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1.8"
                stroke-linecap="round"
                stroke-linejoin="round"
                width="16"
                height="16"
              >
                <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
              </svg>
            </button>
            <button type="button" class="btn btn--ghost btn--sm" :disabled="fetchingModels" @click="fetchModels">
              {{ fetchingModels ? $t('provider.fetchingModels') : $t('provider.fetchModels') }}
            </button>
            <button type="button" class="btn btn--ghost btn--sm" :disabled="testingKey" @click="testApiKey">
              {{ testingKey ? $t('app.testing') : $t('provider.testConnection') }}
            </button>
          </div>
          <span v-if="fetchError" class="field__hint" style="color: var(--error)">{{ fetchError }}</span>
          <span
            v-if="keyTestResult"
            class="badge mt-8"
            :class="keyTestResult.online ? 'badge--ok' : 'badge--err'"
          >
            {{ keyTestResult.online
              ? $t('provider.testSuccess', { latency: keyTestResult.latency_ms ?? '-' })
              : $t('provider.testFail', { error: keyTestResult.error ?? '' }) }}
          </span>
        </div>

        <div class="card card--pad models-card">
          <div class="row" style="justify-content: space-between; align-items: center;">
            <label class="field__label" style="margin: 0;">{{ $t('provider.defaultModels') }}</label>
            <span class="badge badge--info">{{ $t('provider.selected', { count: selectedModels.length }) }}</span>
          </div>

          <div ref="modelSelectRef" class="model-select" :class="{ 'is-open': modelDropdownOpen }">
            <button
              type="button"
              class="model-select__trigger input"
              :class="{ 'is-invalid': submitted && selectedModels.length === 0 }"
              @click="modelDropdownOpen = !modelDropdownOpen"
            >
              <span v-if="selectedModels.length === 0" class="muted">{{ $t('provider.selectOrAdd') }}</span>
              <div v-else class="model-select__pills">
                <span v-for="m in selectedModels" :key="m" class="badge badge--info">
                  {{ m }}
                  <button type="button" class="model-select__remove" @click.stop="removeSelectedModel(m)">×</button>
                </span>
              </div>
              <span class="model-select__arrow">▾</span>
            </button>

            <div v-if="modelDropdownOpen" class="model-select__dropdown card">
              <div class="row">
                <input
                  v-model="customModelInput"
                  type="text"
                  :placeholder="$t('provider.inputModelName')"
                  class="input input--mono grow"
                  @keydown.enter.prevent="addCustomModel"
                />
                <button type="button" class="btn btn--primary btn--sm" @click="addCustomModel">{{ $t('app.add') }}</button>
              </div>
              <div v-if="availableModels.length === 0" class="tc muted mt-12">
                {{ $t('provider.noModels') }}
              </div>
              <div v-else class="model-options">
                <label
                  v-for="m in availableModels"
                  :key="m"
                  class="model-option"
                  :class="{ 'is-selected': selectedModels.includes(m) }"
                >
                  <input v-model="selectedModels" type="checkbox" :value="m" />
                  <span class="model-option__check" />
                  <span class="mono">{{ modelInfoLabel(m) }}</span>
                </label>
              </div>
            </div>
          </div>

          <div v-if="Object.keys(fetchedModelInfo).length > 0" class="mt-12">
            <table class="table">
              <thead>
                <tr>
                  <th>{{ $t('provider.modelName') }}</th>
                  <th>{{ $t('provider.contextLength') }}</th>
                  <th>{{ $t('provider.maxOutput') }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="name in availableModels" :key="`meta-${name}`">
                  <td class="mono">{{ name }}</td>
                  <td>{{ formatTokens(fetchedModelInfo[name]?.context_length) }}</td>
                  <td>{{ formatTokens(fetchedModelInfo[name]?.max_tokens) }}</td>
                </tr>
              </tbody>
            </table>
          </div>

          <div v-if="fetchedRawJson" class="mt-12">
            <button type="button" class="btn btn--ghost btn--sm" @click="showRawJson = !showRawJson">
              {{ showRawJson ? $t('provider.hideRawJson') : $t('provider.viewRawJson') }}
            </button>
            <pre
              v-if="showRawJson"
              class="mono tag mt-8"
              style="max-height: 260px; overflow: auto; white-space: pre-wrap; line-height: 1.5;"
            >{{ fetchedRawJson }}</pre>
          </div>

          <div class="row mt-12">
            <button
              type="button"
              class="btn btn--ghost btn--sm"
              :disabled="fetchingModels"
              @click="fetchModels"
            >
              {{ fetchingModels ? $t('provider.fetchingModels') : $t('provider.fetchFromAPI') }}
            </button>
            <button type="button" class="btn btn--ghost btn--sm" @click="selectAllModels">{{ $t('provider.selectAll') }}</button>
            <button type="button" class="btn btn--ghost btn--sm" @click="clearModelSelection">{{ $t('provider.clearSelection') }}</button>
          </div>
        </div>

      </div>

      <div v-if="activeTab === 'advanced'" class="provider-form__tab">
        <div class="grid-cols cols-3">
          <div class="field">
            <label class="field__label">{{ $t('provider.timeout') }}</label>
            <input v-model.number="form.timeout_seconds" type="number" min="1" max="300" class="input" />
          </div>
          <div class="field">
            <label class="field__label">{{ $t('provider.qpsLimit') }}</label>
            <input v-model.number="form.qps_limit" type="number" min="0" class="input" />
          </div>
          <div class="field">
            <label class="field__label">{{ $t('provider.concurrencyLimit') }}</label>
            <input v-model.number="form.concurrency_limit" type="number" min="0" class="input" />
          </div>
          <div class="field">
            <label class="field__label">{{ $t('provider.tpmLimit') }}</label>
            <input v-model.number="form.tpm_limit" type="number" min="0" class="input" />
          </div>
        </div>

        <div class="field row" style="align-items: flex-start;">
          <label class="switch">
            <input v-model="form.disable_proxy" type="checkbox" />
            <span class="switch__track"><span class="switch__thumb" /></span>
          </label>
          <div class="col">
            <span class="field__label" style="margin: 0;">{{ $t('provider.disableProxy') }}</span>
            <span class="field__hint">{{ $t('provider.disableProxyHint') }}</span>
          </div>
        </div>

        <div class="field">
          <label class="field__label">{{ $t('provider.extraHeaders') }}</label>
          <div v-for="(entry, index) in extraHeadersList" :key="index" class="row">
            <input v-model="entry.key" :placeholder="$t('provider.headerName')" class="input input--mono grow" />
            <input v-model="entry.value" :placeholder="$t('provider.headerValue')" class="input input--mono grow" />
            <button type="button" class="btn btn--danger btn--sm" @click="removeHeader(index)">{{ $t('app.delete') }}</button>
          </div>
          <button type="button" class="btn btn--soft btn--sm mt-8" @click="addHeader">{{ $t('provider.addHeader') }}</button>
        </div>
      </div>
    </div>

    <div class="modal__foot">
      <button type="button" class="btn btn--ghost" @click="emit('cancel')">{{ $t('app.cancel') }}</button>
      <button type="submit" class="btn btn--primary">{{ $t('app.save') }}</button>
    </div>
  </form>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ProviderConfig, ModelInfo, ModelTestTargetResult } from '../../types';
import type { ProviderPreset } from '../../data/providerPresets';
import { providerPresets } from '../../data/providerPresets';
import * as tauri from '../../services/tauri';

const { t } = useI18n();

interface Props {
  provider?: ProviderConfig | null;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  (e: 'save', provider: ProviderConfig, apiKey: string): void;
  (e: 'cancel'): void;
  (e: 'show-message', text: string, type: 'success' | 'error' | 'warn' | 'info'): void;
}>();

const isEdit = computed(() => !!props.provider);

const defaultProvider = (): ProviderConfig => ({
  id: '',
  name: '',
  base_url: '',
  timeout_seconds: 60,
  qps_limit: 0,
  concurrency_limit: 0,
  tpm_limit: 0,
  enabled: true,
  default_models: [],
  extra_headers: {},
  disable_proxy: false,
  model_info: {},
  provider_type: 'openai_compatible',
});

const form = ref<ProviderConfig>(defaultProvider());
const apiKey = ref('');
const apiKeyVisible = ref(false);
const selectedPresetId = ref('');
const activeTab = ref('basic');
const fetchingModels = ref(false);
const fetchedModels = ref<string[]>([]);
const fetchedModelInfo = ref<Record<string, ModelInfo>>({});
const fetchedRawJson = ref('');
const showRawJson = ref(false);
const selectedModels = ref<string[]>([]);
const fetchError = ref('');
const modelDropdownOpen = ref(false);
const customModelInput = ref('');
const modelSelectRef = ref<HTMLElement | null>(null);
const testingKey = ref(false);
const keyTestResult = ref<ModelTestTargetResult | null>(null);
const submitted = ref(false);

function closeDropdownOnOutside(event: MouseEvent): void {
  if (modelSelectRef.value && !modelSelectRef.value.contains(event.target as Node)) {
    modelDropdownOpen.value = false;
  }
}

onMounted(() => document.addEventListener('click', closeDropdownOnOutside));
onUnmounted(() => document.removeEventListener('click', closeDropdownOnOutside));

const availableModels = computed(() => {
  const set = new Set([...fetchedModels.value, ...selectedModels.value]);
  return Array.from(set).sort();
});

interface PresetGroup {
  category: 'cn' | 'us' | 'aggregator';
  label: string;
  presets: typeof providerPresets;
}

const presetGroups = computed<PresetGroup[]>(() => {
  const groups: Record<'cn' | 'us' | 'aggregator', PresetGroup> = {
    cn: { category: 'cn', label: t('provider.categories.cn'), presets: [] },
    us: { category: 'us', label: t('provider.categories.us'), presets: [] },
    aggregator: { category: 'aggregator', label: t('provider.categories.aggregator'), presets: [] },
  };
  for (const p of providerPresets) {
    groups[p.category].presets.push(p);
  }
  return [groups.cn, groups.us, groups.aggregator];
});

const tabs = computed(() => [
  { key: 'basic', label: t('provider.baseInfo') },
  { key: 'advanced', label: t('provider.advanced') },
]);

interface HeaderEntry {
  key: string;
  value: string;
}
const extraHeadersList = ref<HeaderEntry[]>([]);

function applyPreset(): void {
  const preset: ProviderPreset | undefined = providerPresets.find(
    (p) => p.id === selectedPresetId.value
  );
  if (!preset) return;
  form.value.id = preset.id;
  form.value.name = preset.name;
  form.value.base_url = preset.base_url;
  form.value.provider_type = preset.provider_type;
  form.value.default_models = [];
  form.value.extra_headers = { ...preset.extra_headers };
  form.value.disable_proxy = preset.disable_proxy;
  form.value.timeout_seconds = preset.timeout_seconds;
  form.value.qps_limit = preset.qps_limit;
  form.value.concurrency_limit = preset.concurrency_limit;
  form.value.tpm_limit = preset.tpm_limit ?? 0;
  selectedModels.value = [];
  fetchedModels.value = [];
  fetchedModelInfo.value = {};
  fetchedRawJson.value = '';
  showRawJson.value = false;
  fetchError.value = '';
  extraHeadersList.value = Object.entries(preset.extra_headers).map(([key, value]) => ({
    key,
    value,
  }));
}

function resetPreset(): void {
  selectedPresetId.value = '';
  form.value = defaultProvider();
  selectedModels.value = [];
  extraHeadersList.value = [];
}

function normalizeBaseUrl(url: string): string {
  return url.trim().replace(/\/+$/, '');
}

async function copyApiKey(): Promise<void> {
  try {
    await navigator.clipboard?.writeText(apiKey.value);
    emit('show-message', t('app.copied'), 'success');
  } catch {
    emit('show-message', t('app.copyFailed'), 'error');
  }
}

async function copyBaseUrl(): Promise<void> {
  try {
    await navigator.clipboard?.writeText(form.value.base_url);
    emit('show-message', t('app.copied'), 'success');
  } catch {
    emit('show-message', t('app.copyFailed'), 'error');
  }
}

function addHeader(): void {
  extraHeadersList.value.push({ key: '', value: '' });
}

function removeHeader(index: number): void {
  extraHeadersList.value.splice(index, 1);
}

function formatTokens(n?: number): string {
  if (n === undefined || n === null) return '-';
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`;
  return String(n);
}

function modelInfoLabel(name: string): string {
  const info = fetchedModelInfo.value[name] ?? form.value.model_info?.[name];
  if (!info || (info.context_length === undefined && info.max_tokens === undefined)) {
    return name;
  }
  const ctx = info.context_length !== undefined ? formatTokens(info.context_length) : '?';
  const out = info.max_tokens !== undefined ? formatTokens(info.max_tokens) : '?';
  return `${name}（${t('provider.contextLength')} ${ctx} / ${t('provider.maxOutput')} ${out}）`;
}

async function fetchModels(): Promise<void> {
  fetchError.value = '';
  form.value.base_url = normalizeBaseUrl(form.value.base_url);
  if (!form.value.base_url) {
    fetchError.value = t('provider.fillBaseUrlFirst');
    return;
  }
  if (!isEdit.value && !apiKey.value) {
    fetchError.value = t('provider.fillApiKeyFirst');
    return;
  }

  fetchingModels.value = true;
  try {
    const result = await tauri.fetchProviderModels({ ...form.value }, apiKey.value);
    fetchedModels.value = result.models;
    fetchedModelInfo.value = result.info ?? {};
    fetchedRawJson.value = result.raw_json ?? '';
    form.value.model_info = { ...result.info };
    modelDropdownOpen.value = true;
    showRawJson.value = false;
  } catch (e) {
    fetchError.value = t('provider.fetchModelsFailed', { error: String(e) });
    fetchedModels.value = [];
    fetchedModelInfo.value = {};
    fetchedRawJson.value = '';
  } finally {
    fetchingModels.value = false;
  }
}

async function testApiKey(): Promise<void> {
  keyTestResult.value = null;
  form.value.base_url = normalizeBaseUrl(form.value.base_url);
  if (!form.value.base_url) {
    keyTestResult.value = {
      provider_id: form.value.id,
      model_name: '',
      online: false,
      latency_ms: null,
      error: t('provider.fillBaseUrlFirst'),
    };
    return;
  }
  const modelToTest = selectedModels.value[0] ?? fetchedModels.value[0] ?? '';
  if (!modelToTest) {
    keyTestResult.value = {
      provider_id: form.value.id,
      model_name: '',
      online: false,
      latency_ms: null,
      error: t('provider.getModelsFirst'),
    };
    return;
  }

  testingKey.value = true;
  try {
    const result = await tauri.testProviderTarget({ ...form.value }, apiKey.value, modelToTest);
    keyTestResult.value = result;
  } catch (e) {
    keyTestResult.value = {
      provider_id: form.value.id,
      model_name: modelToTest,
      online: false,
      latency_ms: null,
      error: String(e),
    };
  } finally {
    testingKey.value = false;
  }
}

function selectAllModels(): void {
  selectedModels.value = [...availableModels.value];
}

function clearModelSelection(): void {
  selectedModels.value = [];
}

function addCustomModel(): void {
  const name = customModelInput.value.trim();
  if (!name) return;
  if (!availableModels.value.includes(name)) {
    fetchedModels.value.push(name);
  }
  if (!selectedModels.value.includes(name)) {
    selectedModels.value.push(name);
  }
  customModelInput.value = '';
}

function removeSelectedModel(name: string): void {
  selectedModels.value = selectedModels.value.filter((m) => m !== name);
}

watch(
  () => props.provider,
  (p) => {
    submitted.value = false;
    if (p) {
      form.value = { ...defaultProvider(), ...p };
      apiKey.value = '';
      selectedPresetId.value = '';
      fetchError.value = '';
      fetchedModels.value = [];
      fetchedModelInfo.value = { ...(p.model_info || {}) };
      fetchedRawJson.value = '';
      showRawJson.value = false;
      selectedModels.value = [...p.default_models];
      modelDropdownOpen.value = false;
      customModelInput.value = '';
      extraHeadersList.value = Object.entries(p.extra_headers || {}).map(([key, value]) => ({
        key,
        value,
      }));
    } else {
      form.value = defaultProvider();
      apiKey.value = '';
      selectedPresetId.value = '';
      fetchError.value = '';
      fetchedModels.value = [];
      fetchedModelInfo.value = {};
      fetchedRawJson.value = '';
      showRawJson.value = false;
      selectedModels.value = [];
      modelDropdownOpen.value = false;
      customModelInput.value = '';
      extraHeadersList.value = [];
      activeTab.value = 'basic';
    }
  },
  { immediate: true }
);

function submit(): void {
  submitted.value = true;
  form.value.base_url = normalizeBaseUrl(form.value.base_url);
  if (!form.value.id || !form.value.name || !form.value.base_url) {
    return;
  }
  form.value.default_models = [...selectedModels.value];

  const headers: Record<string, string> = {};
  for (const entry of extraHeadersList.value) {
    if (entry.key.trim()) {
      headers[entry.key.trim()] = entry.value;
    }
  }
  form.value.extra_headers = headers;

  emit('save', { ...form.value }, apiKey.value);
}
</script>

<style scoped>
.provider-form {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.model-select {
  position: relative;
}

.model-select__trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  width: 100%;
  min-height: 38px;
  height: auto;
  padding: 6px 34px 6px 13px;
  text-align: left;
}

.model-select__trigger.is-open {
  border-color: var(--primary);
  box-shadow: 0 0 0 4px var(--primary-soft);
}

.model-select__pills {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.model-select__remove {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 14px;
  height: 14px;
  margin-left: 4px;
  border-radius: 50%;
  border: none;
  background: var(--primary);
  color: #fff;
  font-size: 11px;
  line-height: 1;
  cursor: pointer;
}

.model-select__arrow {
  position: absolute;
  right: 13px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-3);
  transition: transform 0.2s;
}

.model-select.is-open .model-select__arrow {
  transform: translateY(-50%) rotate(180deg);
}

.model-select__dropdown {
  position: absolute;
  top: calc(100% + 6px);
  left: 0;
  right: 0;
  max-height: 280px;
  overflow-y: auto;
  z-index: 10;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.model-options {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.model-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 8px;
  border-radius: var(--r-sm);
  cursor: pointer;
  transition: background 0.12s;
}

.model-option:hover {
  background: var(--surface-2);
}

.model-option input {
  position: absolute;
  opacity: 0;
  pointer-events: none;
}

.model-option__check {
  width: 18px;
  height: 18px;
  border-radius: 6px;
  border: 1.5px solid var(--border-strong);
  display: grid;
  place-items: center;
  flex: none;
  transition: all 0.14s;
}

.model-option.is-selected .model-option__check {
  background: var(--primary);
  border-color: var(--primary);
}

.model-option.is-selected .model-option__check::after {
  content: '✓';
  color: #fff;
  font-size: 10px;
  font-weight: 700;
}
</style>
