<template>
  <div class="app" :data-theme="isDark ? 'dark' : 'light'">
    <Sidebar
      :brand-text="$t('app.brand')"
      :workspace-label="$t('nav.workspace')"
      :system-label="$t('nav.system')"
      :settings-label="$t('nav.settings')"
      :workspace-nav="workspaceNav"
      :active-tab="activeTab"
      :server-status="serverStatus"
      :service-running-text="$t('app.serviceRunning')"
      :service-stopped-text="$t('app.serviceStopped')"
      :start-service-text="$t('app.startService')"
      @update:active-tab="activeTab = $event"
    />

    <main class="main">
      <Topbar
        :search-text="$t('app.search')"
        :start-service-text="$t('app.startService')"
        :stop-service-text="$t('app.stopService')"
        :toggle-theme-text="$t('app.toggleTheme')"
        :server-status="serverStatus"
        :is-dark="isDark"
        @open-cmdk="cmdkOpen = true"
        @toggle-server="toggleServer"
        @toggle-theme="toggleTheme"
      />

      <div class="content">
        <OverviewPage
          v-if="activeTab === 'overview'"
          :config="config"
          :server-status="serverStatus"
          :health-status="healthStatus"
          :request-logs="requestLogs"
          :loading-stats="loadingStats"
          @set-tab="activeTab = $event"
          @check-all="checkAllProviders"
          @toggle-server="toggleServer"
          @show-message="showMessage"
        />
        <ProvidersPage
          v-if="activeTab === 'providers'"
          :config="config"
          :health-status="healthStatus"
          :checking-all="checkingAll"
          :loading="loadingStats"
          :check-error="checkError"
          @add-provider="startAddProvider"
          @edit-provider="editProvider"
          @delete-provider="removeProvider"
          @check-provider="checkProvider"
          @check-all="checkAllProviders"
          @persist-config="persistConfig"
        />
        <RoutingPage
          v-if="activeTab === 'mappings'"
          :config="config"
          :model-definitions="modelDefinitions"
          :loading="loadingStats"
          @add-model="startAddModel"
          @edit-model="editModel"
          @delete-model="removeModel"
          @show-message="showMessage"
        />
        <RoutingTreePage
          v-if="activeTab === 'routingTree'"
          :config="config"
          :model-definitions="modelDefinitions"
          :groups="groups"
          :loading="loadingStats"
          @save-group="saveGroup"
          @delete-group="removeGroup"
          @save-definition="saveModelDefinition"
          @delete-definition="removeModelDefinition"
          @show-message="showMessage"
          @update-model-group="onUpdateModelGroup"
          @move-models-to-group="moveModelsToGroup"
          @jump-and-edit-model="jumpAndEditModel"
        />
        <MonitoringPage
          v-if="activeTab === 'stats'"
          :config="config"
          :provider-stats="providerStats"
          :request-logs="requestLogs"
          :loading-stats="loadingStats"
          :stats-error="statsError"
          @set-tab="activeTab = $event"
          @clear-logs="clearLogs"
          @retry-stats="loadProviderStats"
          @show-message="showMessage"
        />
        <LogsPage
          v-if="activeTab === 'logs'"
          :config="config"
          :request-logs="requestLogs"
          :loading-logs="loadingLogs"
          :loading-stats="loadingStats"
          :last-error="lastError"
          :server-url="serverUrl"
          @set-tab="activeTab = $event"
          @refresh-logs="loadRequestLogs(); loadProviderStats()"
          @clear-logs="clearLogs"
          @export-logs="exportLogs"
          @show-message="showMessage"
        />
        <SettingsPage
          v-if="activeTab === 'settings'"
          :config="config"
          :server-status="serverStatus"
          :server-url="serverUrl"
          @save-settings="saveSettings"
          @show-message="showMessage"
        />
      </div>
    </main>

    <Modal
      :open="editingProvider !== undefined"
      :title="editingProvider ? $t('app.editProvider') : $t('app.addProvider')"
      @close="editingProvider = undefined"
    >
      <ProviderForm
        v-if="editingProvider !== undefined"
        :provider="editingProvider"
        @save="saveProvider"
        @cancel="editingProvider = undefined"
        @show-message="showMessage"
      />
    </Modal>

    <Drawer
      :open="editingModel !== undefined"
      :title="editingModel ? $t('app.editModel') : $t('app.newModel')"
      @close="editingModel = undefined"
    >
      <ModelForm
        v-if="modelFormReady && editingModel !== undefined"
        :model="editingModel"
        :providers="config.providers"
        :model-definitions="modelDefinitions"
        @save="saveModel"
        @cancel="editingModel = undefined"
      />
    </Drawer>

    <CommandPalette
      :cmdk-open="cmdkOpen"
      :cmdk-query="cmdkQuery"
      :cmdk-active-index="cmdkActiveIndex"
      :cmdk-flat-items="cmdkFlatItems"
      :cmdk-groups="cmdkGroups"
      :placeholder="$t('app.searchPlaceholder')"
      :no-match-text="$t('app.noMatch')"
      :cmd-select-text="$t('app.cmdSelect')"
      :cmd-jump-text="$t('app.cmdJump')"
      :cmd-close-text="$t('app.cmdClose')"
      :on-cmdk-keydown="onCmdkKeydown"
      @update:cmdk-query="cmdkQuery = $event"
      @update:cmdk-active-index="cmdkActiveIndex = $event"
      @close="cmdkOpen = false"
      @run-cmdk="runCmdk"
    />

    <Toast :message="message" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { useI18n } from 'vue-i18n';
import type { NavItem } from './types';
import { useTheme } from './composables/useTheme';
import { useToast } from './composables/useToast';
import { useAutoRefresh } from './composables/useAutoRefresh';
import { useCommandPalette } from './composables/useCommandPalette';
import { createAppStore } from './stores/useAppStore';
import { createProviderStore } from './stores/useProviderStore';
import { createModelStore } from './stores/useModelStore';
import { createObservabilityStore } from './stores/useObservabilityStore';
import Sidebar from './components/layout/Sidebar.vue';
import Topbar from './components/layout/Topbar.vue';
import Modal from './components/common/Modal.vue';
import Drawer from './components/common/Drawer.vue';
import CommandPalette from './components/common/CommandPalette.vue';
import Toast from './components/common/Toast.vue';
import ProviderForm from './components/forms/ProviderForm.vue';
import ModelForm from './components/forms/ModelForm.vue';
import OverviewPage from './pages/OverviewPage.vue';
import ProvidersPage from './pages/ProvidersPage.vue';
import RoutingTreePage from './pages/RoutingTreePage.vue';
import RoutingPage from './pages/RoutingPage.vue';
import MonitoringPage from './pages/MonitoringPage.vue';
import LogsPage from './pages/LogsPage.vue';
import SettingsPage from './pages/SettingsPage.vue';

const { t } = useI18n();
const { isDark, toggleTheme, applyTheme } = useTheme();
const { message, showMessage } = useToast();

const appStore = createAppStore(t, showMessage);
const providerStore = createProviderStore(t, showMessage, {
  configRef: appStore.config,
  pendingKeysRef: appStore.pendingKeys,
  persistConfig: appStore.persistConfig,
});
const modelStore = createModelStore(t, showMessage, {
  configRef: appStore.config,
  persistConfig: appStore.persistConfig,
});
const observabilityStore = createObservabilityStore(t, showMessage);

const { config, serverStatus, serverUrl, loadConfig, persistConfig, refreshStatus, toggleServer, saveSettings } = appStore;
const { editingProvider, healthStatus, checkingAll, checkError, startAddProvider, editProvider, saveProvider, removeProvider, checkProvider, checkAllProviders, loadHealthStatus } = providerStore;
const { editingModel, groups, modelDefinitions, startAddModel, editModel, saveModel, removeModel, saveGroup, removeGroup, moveModelsToGroup, saveModelDefinition, removeModelDefinition, loadGroups, loadModelDefinitions } = modelStore;

/**
 * 路由树中「删除节点」= 将模型改回默认分组，复用 saveModel 的分组同步逻辑。
 */
async function onUpdateModelGroup(localName: string, newGroup: string): Promise<void> {
  const original = config.value.models.find((m) => m.local_name === localName);
  if (!original) return;
  await saveModel({ ...original, group: newGroup });
}

/**
 * 路由树中「编辑节点」= 跳转到模型映射页并打开编辑抽屉。
 */
async function jumpAndEditModel(localName: string): Promise<void> {
  const target = config.value.models.find((m) => m.local_name === localName);
  if (!target) {
    showMessage(t('app.modelNotFound', { name: localName }), 'warn');
    return;
  }
  activeTab.value = 'mappings';
  await nextTick();
  editModel(target);
}

const { requestLogs, providerStats, loadingLogs, loadingStats, statsError, lastError, loadRequestLogs, loadProviderStats, clearLogs, exportLogs } = observabilityStore;

const activeTab = ref('overview');
const modelFormReady = ref(false);
let modelFormTimer: ReturnType<typeof setTimeout> | null = null;

watch(
  () => editingModel.value !== undefined,
  (shouldOpen) => {
    if (shouldOpen) {
      nextTick(() => {
        if (modelFormTimer) clearTimeout(modelFormTimer);
        modelFormTimer = setTimeout(() => {
          modelFormReady.value = true;
        }, 80);
      });
    } else {
      if (modelFormTimer) clearTimeout(modelFormTimer);
      modelFormTimer = null;
      modelFormReady.value = false;
    }
  },
  { flush: 'post' }
);

useAutoRefresh(activeTab, loadRequestLogs, loadProviderStats);

const workspaceNav = computed<NavItem[]>(() => {
  return [
    { key: 'overview', label: t('nav.overview'), icon: '<path d="M3 9.5 12 3l9 6.5V20a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z"/>' },
    { key: 'providers', label: t('nav.providers'), icon: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>', badge: config.value.providers.length },
    { key: 'mappings', label: t('nav.mappings'), icon: '<circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="6" r="2.5"/><circle cx="12" cy="18" r="2.5"/><path d="M8.2 7.2 16 16"/><path d="M7.5 8 11 15.5"/><path d="M15.8 7.5 13 15.5"/>', badge: config.value.models.length },
    { key: 'routingTree', label: t('nav.routingTree'), icon: '<rect x="3" y="3" width="18" height="18" rx="2"/><path d="M12 8v8"/><path d="M8 12h8"/>', badge: groups.value.length },
    { key: 'stats', label: t('nav.stats'), icon: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>' },
    { key: 'logs', label: t('nav.logs'), icon: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6"/><path d="M9 13h6"/><path d="M9 17h4"/>' },
  ];
});

const { cmdkOpen, cmdkQuery, cmdkActiveIndex, cmdkFlatItems, cmdkGroups, onCmdkKeydown, runCmdk, onGlobalKeydown } = useCommandPalette(
  activeTab,
  config,
  groups,
  serverStatus,
  editingProvider,
  editingModel,
  t,
  { editProvider, editModel, toggleServer, checkAllProviders, toggleTheme },
);

onMounted(async () => {
  const savedTheme = localStorage.getItem('api-router-theme');
  isDark.value = savedTheme === 'dark';
  applyTheme();

  await loadConfig();
  const bootResults = await Promise.allSettled([
    loadGroups(),
    loadModelDefinitions(),
    refreshStatus(),
    loadHealthStatus(),
    loadRequestLogs(),
    loadProviderStats(),
  ]);
  bootResults.forEach((r) => {
    if (r.status === 'rejected') {
      console.error('[boot] init failed:', r.reason);
      showMessage(String((r.reason as Error)?.message ?? r.reason ?? '初始化失败'), 'warn');
    }
  });

  document.addEventListener('keydown', onGlobalKeydown);
});

onBeforeUnmount(() => {
  document.removeEventListener('keydown', onGlobalKeydown);
});
</script>
