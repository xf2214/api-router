import { reactive, toRefs, type Ref } from 'vue';
import type { ModelMapping, ModelGroup, ModelDefinition, AppConfig } from '../types';
import * as tauri from '../services/tauri';

export function createModelStore(
  t: (key: string, args?: Record<string, unknown>) => string,
  showMessageFn: (text: string, type: 'success' | 'error' | 'warn' | 'info') => void,
  helpers: {
    configRef: Ref<AppConfig>;
    persistConfig: () => Promise<void>;
  },
) {
  const state = reactive<{
    editingModel: ModelMapping | null | undefined;
    groups: ModelGroup[];
    modelDefinitions: ModelDefinition[];
  }>({
    editingModel: undefined,
    groups: [],
    modelDefinitions: [],
  });

  function startAddModel(): void {
    state.editingModel = null;
  }

  function editModel(m: ModelMapping): void {
    state.editingModel = { ...m, targets: m.targets.map((t) => ({ ...t })) };
  }

  async function saveModel(m: ModelMapping): Promise<void> {
    try {
      const idx = helpers.configRef.value.models.findIndex((x) => x.local_name === m.local_name);
      const old_group = idx >= 0 ? (helpers.configRef.value.models[idx].group ?? '默认') : '默认';
      const new_group = m.group || '默认';

      // 1. 先保存模型本身
      if (idx >= 0) {
        helpers.configRef.value.models[idx] = { ...m };
      } else {
        helpers.configRef.value.models.push({ ...m });
      }

      // 2. 分组变更：从旧分组移除，加到新分组（新分组不存在则自动创建）
      if (old_group !== new_group) {
        if (old_group !== '默认') {
          const gIdx = state.groups.findIndex((g) => g.name === old_group);
          if (gIdx >= 0) {
            const g = {
              ...state.groups[gIdx],
              members: state.groups[gIdx].members.filter((mm) => mm.local_name !== m.local_name),
            };
            await tauri.saveGroup(g);
          }
        }
        if (new_group !== '默认') {
          const gIdx = state.groups.findIndex((g) => g.name === new_group);
          if (gIdx < 0) {
            // 自动创建新分组（默认配置）
            const newG: ModelGroup = {
              name: new_group,
              members: [{ local_name: m.local_name, weight: 1 }],
              strategy: 'priority',
              fallback_enabled: true,
            };
            await tauri.saveGroup(newG);
          } else {
            const existing = state.groups[gIdx];
            const alreadyMember = existing.members.some((mm) => mm.local_name === m.local_name);
            if (!alreadyMember) {
              const g = {
                ...existing,
                members: [...existing.members, { local_name: m.local_name, weight: 1 }],
              };
              await tauri.saveGroup(g);
            }
          }
        }
      }

      await helpers.persistConfig();
      await loadGroups();

      // 3. 优化：把最后一个成员迁走后，自动删除空分组（仅对非默认 old_group 生效）
      if (old_group !== '默认' && old_group !== new_group) {
        const remainCount = helpers.configRef.value.models.filter(
          (mm) => mm.group === old_group
        ).length;
        if (remainCount === 0) {
          try {
            await tauri.deleteGroup(old_group);
            await loadGroups();
          } catch (_e) {
            // 删除空分组失败不影响主流程（可能是已被并发删除）
          }
        }
      }

      state.editingModel = undefined;
      showMessageFn(t('routing.saved'), 'success');
    } catch (e) {
      showMessageFn(t('routing.saveFailed', { error: String(e) }), 'error');
    }
  }

  async function removeModel(localName: string): Promise<void> {
    if (!confirm(t('routing.deleteConfirm', { name: localName }))) return;
    try {
      helpers.configRef.value.models = helpers.configRef.value.models.filter((m) => m.local_name !== localName);

      // 同步清理所有分组的 members 中的残留引用
      const dirtyGroups = state.groups.filter((g) =>
        g.members.some((mm) => mm.local_name === localName)
      );
      for (const g of dirtyGroups) {
        const cleaned = {
          ...g,
          members: g.members.filter((mm) => mm.local_name !== localName),
        };
        await tauri.saveGroup(cleaned);
      }

      await helpers.persistConfig();
      if (dirtyGroups.length > 0) await loadGroups();
      showMessageFn(t('routing.deleted'), 'success');
    } catch (e) {
      showMessageFn(t('routing.deleteFailed', { error: String(e) }), 'error');
    }
  }

  async function saveGroup(g: ModelGroup): Promise<void> {
    try {
      await tauri.saveGroup(g);
      await loadGroups();
      showMessageFn(t('routingTree.saved'), 'success');
    } catch (e) {
      showMessageFn(t('routingTree.saveFailed', { error: String(e) }), 'error');
    }
  }

  async function removeGroup(name: string): Promise<void> {
    const count = helpers.configRef.value.models.filter((m) => m.group === name).length;
    if (!confirm(t('routingTree.deleteConfirm', { name, count }))) return;
    try {
      // 先把该分组下的所有模型批量回流到"默认"分组
      let changed = false;
      for (const m of helpers.configRef.value.models) {
        if (m.group === name) {
          m.group = '默认';
          changed = true;
        }
      }
      if (changed) {
        await helpers.persistConfig();
      }

      await tauri.deleteGroup(name);
      await loadGroups();
      showMessageFn(t('routingTree.deleted'), 'success');
    } catch (e) {
      showMessageFn(t('routingTree.deleteFailed', { error: String(e) }), 'error');
    }
  }

  /**
   * C1：批量将多个模型迁移到指定目标分组（默认组/空字符串对应「默认」）
   *  - 逐模型更新 configRef.models[i].group
   *  - 同步旧/新分组 members（新分组不存在则自动创建）
   *  - 迁移完成后清理空分组并 persistConfig + reload groups
   */
  async function moveModelsToGroup(localNames: string[], targetGroup: string): Promise<void> {
    if (!localNames || localNames.length === 0) return;
    const normalizedTarget = targetGroup.trim() || '默认';

    try {
      const models = helpers.configRef.value.models;
      // 按 old_group 聚合影响过的组名，便于后续批量 saveGroup
      const touchedGroups = new Set<string>();
      const affected = new Map<string, string>(); // localName -> old_group

      for (const name of localNames) {
        const idx = models.findIndex((m) => m.local_name === name);
        if (idx < 0) continue;
        const old_group = models[idx].group || '默认';
        if (old_group === normalizedTarget) continue; // 已在目标组，跳过
        affected.set(name, old_group);
        touchedGroups.add(old_group);
        models[idx].group = normalizedTarget;
      }

      if (affected.size === 0) return; // 没实际变更
      touchedGroups.add(normalizedTarget);

      // 1) 移除旧分组里的这些成员
      for (const gName of touchedGroups) {
        if (gName === normalizedTarget) continue; // 目标组在后面处理
        if (gName === '默认') continue; // 默认组不存 group 记录
        const gIdx = state.groups.findIndex((g) => g.name === gName);
        if (gIdx < 0) continue;
        const g = state.groups[gIdx];
        const newMembers = g.members.filter((mm) => !affected.has(mm.local_name));
        if (newMembers.length !== g.members.length) {
          await tauri.saveGroup({ ...g, members: newMembers });
        }
      }

      // 2) 加入新分组（若非默认且不存在，则创建）
      if (normalizedTarget !== '默认') {
        const gIdx = state.groups.findIndex((g) => g.name === normalizedTarget);
        if (gIdx < 0) {
          // 新建分组，默认策略 priority + fallback
          const newMembers = Array.from(affected.entries())
            .filter(([, _old]) => true)
            .map(([n]) => ({ local_name: n, weight: 1 }));
          await tauri.saveGroup({
            name: normalizedTarget,
            members: newMembers,
            strategy: 'priority',
            fallback_enabled: true,
          });
        } else {
          const g = state.groups[gIdx];
          const addedNames = Array.from(affected.keys());
          const existingNames = new Set(g.members.map((m) => m.local_name));
          const toAdd = addedNames.filter((n) => !existingNames.has(n));
          if (toAdd.length > 0) {
            await tauri.saveGroup({
              ...g,
              members: [...g.members, ...toAdd.map((n) => ({ local_name: n, weight: 1 }))],
            });
          }
        }
      }

      // 3) 持久化 models
      await helpers.persistConfig();
      await loadGroups();

      // 4) 清理被移空的旧分组（非默认）
      for (const gName of touchedGroups) {
        if (gName === normalizedTarget) continue;
        if (gName === '默认') continue;
        const remainCount = helpers.configRef.value.models.filter((m) => m.group === gName).length;
        if (remainCount === 0) {
          try {
            await tauri.deleteGroup(gName);
            await loadGroups();
          } catch (_e) {
            // ignore
          }
        }
      }
    } catch (e) {
      showMessageFn(t('routing.saveFailed', { error: String(e) }), 'error');
    }
  }

  async function saveModelDefinition(def: ModelDefinition): Promise<void> {
    try {
      await tauri.saveModelDefinition(def);
      await loadModelDefinitions();
      // 同步更新 appStore.config 中的 model_definitions，保证前端状态一致
      helpers.configRef.value.model_definitions = [...state.modelDefinitions];
      showMessageFn(t('models.saved'), 'success');
    } catch (e) {
      showMessageFn(t('models.saveFailed', { error: String(e) }), 'error');
    }
  }

  async function removeModelDefinition(id: string): Promise<void> {
    if (!confirm(t('models.deleteConfirm', { name: id }))) return;
    try {
      await tauri.deleteModelDefinition(id);
      await loadModelDefinitions();
      // 同步更新 appStore.config 中的 model_definitions
      helpers.configRef.value.model_definitions = [...state.modelDefinitions];
      showMessageFn(t('models.deleted'), 'success');
    } catch (e) {
      showMessageFn(t('models.deleteFailed', { error: String(e) }), 'error');
    }
  }

  async function loadGroups(): Promise<void> {
    try {
      state.groups = await tauri.getGroups();
    } catch (e) {
      showMessageFn(t('common.loadFailed', { error: String(e) }), 'error');
    }
  }

  async function loadModelDefinitions(): Promise<void> {
    try {
      state.modelDefinitions = await tauri.getModelDefinitions();
    } catch (e) {
      showMessageFn(t('common.loadFailed', { error: String(e) }), 'error');
    }
  }

  return {
    ...toRefs(state),
    startAddModel,
    editModel,
    saveModel,
    removeModel,
    saveGroup,
    removeGroup,
    moveModelsToGroup,
    saveModelDefinition,
    removeModelDefinition,
    loadGroups,
    loadModelDefinitions,
  };
}

export type ModelStore = ReturnType<typeof createModelStore>;
