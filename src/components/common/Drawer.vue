<template>
  <div class="overlay" :class="{ 'is-open': open }" @click="$emit('close')" />
  <div class="drawer drawer--wide" :class="{ 'is-open': open }">
    <div class="drawer__head">
      <span class="modal__title">{{ title }}</span>
      <div class="spacer" />
      <button class="btn btn--ghost btn--icon" @click="$emit('close')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="18" height="18"><path d="M18 6 6 18M6 6l12 12"/></svg>
      </button>
    </div>
    <div class="drawer__body">
      <span class="drawer__placeholder" aria-hidden="true"></span>
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  open: boolean;
  title: string;
}>();

defineEmits<{
  (e: 'close'): void;
}>();
</script>
