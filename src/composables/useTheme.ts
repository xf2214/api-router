import { ref, onMounted } from 'vue';

const isDark = ref(false);

function applyTheme() {
  document.body.setAttribute('data-theme', isDark.value ? 'dark' : 'light');
}

function toggleTheme() {
  isDark.value = !isDark.value;
  applyTheme();
  localStorage.setItem('api-router-theme', isDark.value ? 'dark' : 'light');
}

export function useTheme() {
  onMounted(() => {
    const saved = localStorage.getItem('api-router-theme');
    isDark.value = saved === 'dark';
    applyTheme();
  });
  return { isDark, toggleTheme, applyTheme };
}
