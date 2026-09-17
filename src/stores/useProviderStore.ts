import { reactive, toRefs, type Ref } from 'vue';
import type { ProviderConfig, ProviderHealth, AppConfig, ModelMapping } from '../types';
import * as tauri from '../services/tauri';

export function createProviderStore(
  t: (key: string, args?: Record<string, unknown>) => string,
  showMessageFn: (text: string, type: 'success' | 'error' | 'warn' | 'info') => void,
  helpers: {
    configRef: Ref<AppConfig>;
    pendingKeysRef: Ref<Record<string, string>>;
    persistConfig: () => Promise<void>;
  },
) {
  const state = reactive<{
    editingProvider: ProviderConfig | null | undefined;
    healthStatus: ProviderHealth[];
    checkingAll: boolean;
    checkError: string | null;
  }>({
    editingProvider: undefined,
    healthStatus: [],
    checkingAll: false,
    checkError: null,
  });

  function startAddProvider(): void {
    state.editingProvider = null;
  }

  function editProvider(p: ProviderConfig): void {
    state.editingProvider = { ...p };
  }

  async function saveProvider(p: ProviderConfig, apiKey: string): Promise<void> {
    try {
      const idx = helpers.configRef.value.providers.findIndex((x) => x.id === p.id);
      const isNew = idx < 0;
      if (idx >= 0) {
        helpers.configRef.value.providers[idx] = { ...p };
      } else {
        helpers.configRef.value.providers.push({ ...p });
      }
      if (apiKey) {
        helpers.pendingKeysRef.value[p.id] = apiKey;
      }

      // D1：新建 provider 时，对每个 default_models 条目自动生成模型映射
      //   - local_name = 上游模型名
      //   - 默认分组，单目标指向本 provider
      let createdCount = 0;
      let skippedCount = 0;
      if (isNew && Array.isArray(p.default_models) && p.default_models.length > 0) {
        const existingLocalNames = new Set(helpers.configRef.value.models.map((m) => m.local_name));
        const toAdd: ModelMapping[] = [];
        for (const modelName of p.default_models) {
          const nm = String(modelName || '').trim();
          if (!nm) continue;
          if (existingLocalNames.has(nm)) {
            skippedCount++;
            continue;
          }
          toAdd.push({
            local_name: nm,
            group: '默认',
            strategy: 'priority',
            fallback_enabled: true,
            max_retries: 2,
            targets: [
              {
                provider_id: p.id,
                model_name: nm,
                tier: 1,
                weight: 1,
              },
            ],
          });
          createdCount++;
        }
        if (toAdd.length > 0) {
          helpers.configRef.value.models.push(...toAdd);
        }
      }

      await helpers.persistConfig();
      showMessageFn(t('provider.saved'), 'success');
      if (createdCount > 0) {
        showMessageFn(t('routingTree.autoGenMappingsCreated', { count: createdCount }), 'info');
      }
      if (skippedCount > 0) {
        showMessageFn(t('routingTree.autoGenMappingsSkipped', { count: skippedCount }), 'info');
      }
      await checkProvider(p.id);
    } catch (e) {
      showMessageFn(t('provider.saveFailed', { error: String(e) }), 'error');
    }
  }

  async function removeProvider(id: string): Promise<void> {
    if (!confirm(t('provider.deleteConfirm'))) return;
    try {
      await tauri.deleteProvider(id);
      helpers.configRef.value.providers = helpers.configRef.value.providers.filter((p) => p.id !== id);
      helpers.configRef.value.models = helpers.configRef.value.models.filter((m) => !m.targets.some((t) => t.provider_id === id));
      await helpers.persistConfig();
      showMessageFn(t('provider.deleted'), 'success');
    } catch (e) {
      showMessageFn(t('provider.deleteFailed', { error: String(e) }), 'error');
    }
  }

  async function checkProvider(id: string): Promise<void> {
    try {
      const h = await tauri.checkProviderHealth(id);
      state.healthStatus = state.healthStatus.filter((x) => x.provider_id !== id).concat(h);
    } catch (e) {
      console.error('checkProviderHealth failed', e);
    }
  }

  async function checkAllProviders(): Promise<void> {
    state.checkingAll = true;
    state.checkError = null;
    try {
      const list = await tauri.checkAllProvidersHealth();
      state.healthStatus = list;
    } catch (e) {
      state.checkError = String(e);
      showMessageFn(t('provider.saveFailed', { error: String(e) }), 'error');
    } finally {
      state.checkingAll = false;
    }
  }

  async function loadHealthStatus(): Promise<void> {
    try {
      state.healthStatus = await tauri.getHealthStatus();
    } catch (e) {
      console.error('getHealthStatus failed', e);
    }
  }

  return {
    ...toRefs(state),
    startAddProvider,
    editProvider,
    saveProvider,
    removeProvider,
    checkProvider,
    checkAllProviders,
    loadHealthStatus,
  };
}

export type ProviderStore = ReturnType<typeof createProviderStore>;
