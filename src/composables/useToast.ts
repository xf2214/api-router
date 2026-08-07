import { ref } from 'vue';
import type { ToastMessage, ToastType } from '../types/ui';

const message = ref<ToastMessage | null>(null);
let toastTimer: ReturnType<typeof setTimeout> | null = null;

export function useToast() {
  function showMessage(text: string, type: ToastType) {
    message.value = { text, type };
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => {
      message.value = null;
    }, 3000);
  }
  return { message, showMessage };
}
