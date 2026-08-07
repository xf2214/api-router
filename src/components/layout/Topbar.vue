<template>
  <header class="topbar">
    <div class="topbar__spacer" />
    <div class="topbar__actions">
      <button class="btn btn--ghost btn--sm" style="gap: 8px; color: var(--text-3)" @click="$emit('openCmdk')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="16" height="16">
          <circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/>
        </svg>
        <span style="font-size: 12.5px; font-weight: 600">{{ searchText }}</span>
        <span class="kbd">⌘K</span>
      </button>

      <button class="btn btn--ghost btn--sm" @click="$emit('toggleServer')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="15" height="15">
          <path v-if="serverStatus.running" d="M18.36 6.64A9 9 0 0 1 20.77 15M5.63 6.64A9 9 0 1 1 12 21a9.72 9.72 0 0 1-6.74-2.74L12 12Z"/>
          <path v-else d="M13 2 3 14h9l-1 8 10-12h-9l1-8z"/>
        </svg>
        {{ serverStatus.running ? stopServiceText : startServiceText }}
      </button>

      <button class="btn btn--ghost btn--icon" :title="toggleThemeText" @click="$emit('toggleTheme')">
        <svg v-if="isDark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
          <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/>
        </svg>
        <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="18" height="18">
          <circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/>
        </svg>
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import type { ServerStatus } from '../../types';

defineProps<{
  searchText: string;
  startServiceText: string;
  stopServiceText: string;
  toggleThemeText: string;
  serverStatus: ServerStatus;
  isDark: boolean;
}>();

defineEmits<{
  (e: 'openCmdk'): void;
  (e: 'toggleServer'): void;
  (e: 'toggleTheme'): void;
}>();
</script>
