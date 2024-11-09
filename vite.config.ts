import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    // Increase chunkSizeWarningLimit because it's not really applicable to slow
    // network load times. Jute is a desktop app.
    chunkSizeWarningLimit: 1024,
  },

  plugins: [react()],

  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
