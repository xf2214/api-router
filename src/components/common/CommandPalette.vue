<template>
  <div class="cmdk-overlay" :class="{ 'is-open': cmdkOpen }" @click="$emit('close')" />
  <div class="cmdk" :class="{ 'is-open': cmdkOpen }">
    <div class="cmdk__input-wrap">
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="20" height="20"><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></svg>
      <input
        ref="inputRef"
        :value="cmdkQuery"
        @input="$emit('update:cmdkQuery', ($event.target as HTMLInputElement).value)"
        class="cmdk__input"
        :placeholder="placeholder"
        autocomplete="off"
        @keydown="onCmdkKeydown"
      >
      <span class="kbd">Esc</span>
    </div>
    <div class="cmdk__list">
      <template v-for="(group, gi) in cmdkGroups" :key="gi">
        <div class="cmdk__group">{{ group.name }}</div>
        <div
          v-for="(item, ii) in group.items"
          :key="ii"
          class="cmdk__item"
          :class="{ 'is-active': cmdkActiveIndex === item.globalIndex }"
          @click="runCmdk(item)"
          @mouseenter="$emit('update:cmdkActiveIndex', item.globalIndex)"
        >
          <svg v-if="item.icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" width="17" height="17" v-html="item.icon" />
          <span>{{ item.label }}</span>
          <span v-if="item.hint" class="cmdk__hint">{{ item.hint }}</span>
        </div>
      </template>
      <div v-if="cmdkFlatItems.length === 0" class="cmdk__group">{{ noMatchText }}</div>
    </div>
    <div class="cmdk__foot">
      <span><span class="kbd">↑↓</span> {{ cmdSelectText }}</span>
      <span><span class="kbd">↵</span> {{ cmdJumpText }}</span>
      <span><span class="kbd">Esc</span> {{ cmdCloseText }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import type { CmdItem } from '../../composables/useCommandPalette';

const props = defineProps<{
  cmdkOpen: boolean;
  cmdkQuery: string;
  cmdkActiveIndex: number;
  cmdkFlatItems: CmdItem[];
  cmdkGroups: { name: string; items: CmdItem[] }[];
  placeholder: string;
  noMatchText: string;
  cmdSelectText: string;
  cmdJumpText: string;
  cmdCloseText: string;
  onCmdkKeydown: (e: KeyboardEvent) => void;
}>();

const emit = defineEmits<{
  (e: 'update:cmdkQuery', v: string): void;
  (e: 'update:cmdkActiveIndex', v: number): void;
  (e: 'close'): void;
  (e: 'runCmdk', item: CmdItem): void;
}>();

const inputRef = ref<HTMLInputElement | null>(null);

watch(
  () => props.cmdkOpen,
  (open) => {
    if (open) {
      emit('update:cmdkQuery', '');
      nextTick(() => inputRef.value?.focus());
    }
  },
);

function runCmdk(item: CmdItem) {
  emit('runCmdk', item);
}
</script>
