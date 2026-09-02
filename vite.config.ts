import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig(async () => ({
  plugins: [vue()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: "esnext",
    rollupOptions: {
      output: {
        manualChunks: {
          vue: ["vue", "vue-i18n"],
          tauri: ["@tauri-apps/api", "@tauri-apps/plugin-shell"],
        },
      },
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
}));
