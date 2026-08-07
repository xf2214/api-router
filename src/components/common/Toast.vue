<template>
  <div v-if="message" class="toast-wrap">
    <div class="toast" :class="`toast--${message.type}`">{{ message.text }}</div>
  </div>
</template>

<script setup lang="ts">
import type { ToastMessage } from '../../types/ui';

defineProps<{
  message: ToastMessage | null;
}>();
</script>
