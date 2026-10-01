import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Tauri expects a fixed port and no clearing of the screen; harmless for plain
// browser dev too. See ARCHITECTURE.md §18.
export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: false,
    host: true,
  },
  build: {
    target: "es2021",
    outDir: "dist",
  },
});
