<template>
  <aside class="sidebar">
    <div class="sidebar__brand">
      <div class="brand__mark">
        <svg viewBox="0 0 24 24" fill="none" stroke="#fff" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" width="16" height="16">
          <path d="M16 3h5v5"/><path d="M21 3l-7 7"/><path d="M8 21H3v-5"/><path d="m3 21 7-7"/><path d="M21 16v5h-5"/><path d="m14 14 7 7"/><path d="M3 8V3h5"/><path d="m3 3 7 7"/>
        </svg>
      </div>
      <div class="brand__name">{{ brandText }}</div>
    </div>

    <nav class="sidebar__nav">
      <div class="nav-group__label">{{ workspaceLabel }}</div>
      <button
        v-for="item in workspaceNav"
        :key="item.key"
        class="nav-item"
        :class="{ 'is-active': activeTab === item.key }"
        @click="$emit('update:activeTab', item.key)"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" v-html="item.icon" />
        <span>{{ item.label }}</span>
        <span v-if="item.badge !== undefined" class="nav-item__badge">{{ item.badge }}</span>
      </button>

      <div class="nav-group__label">{{ systemLabel }}</div>
      <button
        class="nav-item"
        :class="{ 'is-active': activeTab === 'settings' }"
        @click="$emit('update:activeTab', 'settings')"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>
        </svg>
        <span>{{ settingsLabel }}</span>
      </button>
    </nav>

    <div class="sidebar__foot">
      <button
        type="button"
        class="service-pill"
        :title="serverStatus.running ? clickToManageText : clickToStartText"
        @click="$emit('toggleService')"
      >
        <span class="dot" :class="serverStatus.running ? 'dot--ok' : 'dot--err'" />
        <div class="service-pill__txt">
          <div class="service-pill__t">{{ serverStatus.running ? serviceRunningText : serviceStoppedText }}</div>
          <div class="service-pill__s">{{ serverStatus.running ? `127.0.0.1:${serverStatus.port}` : startServiceText }}</div>
        </div>
      </button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import type { NavItem, ServerStatus } from '../../types';

defineProps<{
  brandText: string;
  workspaceLabel: string;
  systemLabel: string;
  settingsLabel: string;
  workspaceNav: NavItem[];
  activeTab: string;
  serverStatus: ServerStatus;
  serviceRunningText: string;
  serviceStoppedText: string;
  startServiceText: string;
  clickToManageText?: string;
  clickToStartText?: string;
}>();

defineEmits<{
  (e: 'update:activeTab', v: string): void;
  (e: 'toggleService'): void;
}>();
</script>
