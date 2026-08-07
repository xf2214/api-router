<template>
  <div class="overlay" :class="{ 'is-open': open }" @click="$emit('close')" />
  <div class="modal" :class="{ 'is-open': open }" :style="style">
    <div class="modal__head">
      <span class="modal__title">{{ title }}</span>
      <div class="spacer" />
      <button class="btn btn--ghost btn--icon" @click="$emit('close')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18"><path d="M18 6 6 18M6 6l12 12"/></svg>
      </button>
    </div>
    <slot />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    width?: string;
  }>(),
  {
    width: 'min(640px, 92vw)',
  },
);

defineEmits<{
  (e: 'close'): void;
}>();

const style = computed(() => ({
  width: props.width,
  maxHeight: '90vh',
}));
</script>
