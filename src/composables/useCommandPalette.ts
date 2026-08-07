import { ref, computed, watch, nextTick, type Ref } from 'vue';
import type { AppConfig, ProviderConfig, ModelMapping, ModelGroup, ServerStatus } from '../types';

export interface CmdItem {
  label: string;
  icon?: string;
  hint?: string;
  run: () => void;
  globalIndex: number;
}

export function useCommandPalette(
  activeTab: Ref<string>,
  config: Ref<AppConfig>,
  groups: Ref<ModelGroup[]>,
  serverStatus: Ref<ServerStatus>,
  editingProvider: Ref<ProviderConfig | null | undefined>,
  editingModel: Ref<ModelMapping | null | undefined>,
  t: (key: string, args?: any) => string,
  callbacks: {
    editProvider: (p: ProviderConfig) => void;
    editModel: (m: ModelMapping) => void;
    toggleServer: () => Promise<void>;
    checkAllProviders: () => Promise<void>;
    toggleTheme: () => void;
  },
) {
  const cmdkOpen = ref(false);
  const cmdkQuery = ref('');
  const cmdkActiveIndex = ref(0);
  const cmdkInput = ref<HTMLInputElement | null>(null);

  const cmdkFlatItems = computed<CmdItem[]>(() => {
    const items: CmdItem[] = [];
    let idx = 0;
    const q = cmdkQuery.value.trim().toLowerCase();
    function add(label: string, run: () => void, icon?: string, hint?: string) {
      if (q && !label.toLowerCase().includes(q)) return;
      items.push({ label, run, icon, hint, globalIndex: idx++ });
    }
    add(t('nav.overview'), () => { activeTab.value = 'overview'; cmdkOpen.value = false; }, '<path d="M3 9.5 12 3l9 6.5V20a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z"/>', t('common.page'));
    add(t('nav.providers'), () => { activeTab.value = 'providers'; cmdkOpen.value = false; }, '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>', t('common.page'));
    add(t('nav.mappings'), () => { activeTab.value = 'mappings'; cmdkOpen.value = false; }, '<circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="6" r="2.5"/><circle cx="12" cy="18" r="2.5"/><path d="M8.2 7.2 16 16"/><path d="M7.5 8 11 15.5"/><path d="M15.8 7.5 13 15.5"/>', t('common.page'));
    add(t('nav.routingTree'), () => { activeTab.value = 'routingTree'; cmdkOpen.value = false; }, '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 8v8"/><path d="M8 12h8"/>', t('common.page'));
    add(t('nav.stats'), () => { activeTab.value = 'stats'; cmdkOpen.value = false; }, '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>', t('common.page'));
    add(t('nav.logs'), () => { activeTab.value = 'logs'; cmdkOpen.value = false; }, '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M9 13h6"/><path d="M9 17h4"/>', t('common.page'));
    add(t('nav.settings'), () => { activeTab.value = 'settings'; cmdkOpen.value = false; }, '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>', t('common.page'));
    for (const p of config.value.providers) {
      add(p.name, () => { activeTab.value = 'providers'; callbacks.editProvider(p); cmdkOpen.value = false; }, '<rect x="3" y="3" width="7" height="7" rx="1"/>', t('provider.title'));
    }
    for (const m of config.value.models) {
      add(m.local_name, () => { activeTab.value = 'mappings'; callbacks.editModel(m); cmdkOpen.value = false; }, '<circle cx="6" cy="6" r="2.5"/>', t('nav.mappings'));
    }
    for (const g of groups.value) {
      add(g.name, () => { activeTab.value = 'routingTree'; cmdkOpen.value = false; }, '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 8v8"/><path d="M8 12h8"/>', t('routingTree.title'));
    }
    add(serverStatus.value.running ? t('app.stopService') : t('app.startService'), () => { callbacks.toggleServer(); cmdkOpen.value = false; }, '<rect x="2" y="6" width="20" height="12" rx="2"/>', t('common.action'));
    add(t('provider.healthCheckAll'), () => { callbacks.checkAllProviders(); cmdkOpen.value = false; }, '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>', t('common.action'));
    add(t('app.toggleTheme'), () => { callbacks.toggleTheme(); cmdkOpen.value = false; }, '<circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/>', t('common.action'));
    return items;
  });

  const cmdkGroups = computed(() => {
    const groups: { name: string; items: CmdItem[] }[] = [
      { name: t('common.page'), items: [] },
      { name: t('provider.title'), items: [] },
      { name: t('nav.mappings'), items: [] },
      { name: t('routingTree.title'), items: [] },
      { name: t('common.action'), items: [] },
    ];
    for (const item of cmdkFlatItems.value) {
      const g = groups.find((x) => x.name === (item.hint ?? t('common.action')));
      if (g) g.items.push(item);
    }
    return groups.filter((g) => g.items.length > 0);
  });

  watch(cmdkFlatItems, () => {
    cmdkActiveIndex.value = cmdkFlatItems.value.length > 0 ? cmdkFlatItems.value[0].globalIndex : 0;
  });
  watch(cmdkOpen, (open) => {
    if (open) {
      cmdkQuery.value = '';
      nextTick(() => cmdkInput.value?.focus());
    }
  });

  function onCmdkKeydown(e: KeyboardEvent) {
    const items = cmdkFlatItems.value;
    if (items.length === 0) return;
    const idx = items.findIndex((i) => i.globalIndex === cmdkActiveIndex.value);
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      cmdkActiveIndex.value = items[(idx + 1) % items.length].globalIndex;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      cmdkActiveIndex.value = items[(idx - 1 + items.length) % items.length].globalIndex;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const active = items.find((i) => i.globalIndex === cmdkActiveIndex.value);
      if (active) active.run();
    }
  }
  function runCmdk(item: CmdItem) {
    item.run();
  }
  function onGlobalKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      cmdkOpen.value = !cmdkOpen.value;
    } else if (e.key === 'Escape') {
      cmdkOpen.value = false;
      editingProvider.value = undefined;
      editingModel.value = undefined;
    }
  }
  return {
    cmdkOpen,
    cmdkQuery,
    cmdkActiveIndex,
    cmdkInput,
    cmdkFlatItems,
    cmdkGroups,
    onCmdkKeydown,
    runCmdk,
    onGlobalKeydown,
  };
}
