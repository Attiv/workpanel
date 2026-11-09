import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    // The app only ever runs inside Tauri's bundled WebView (WKWebView on macOS,
    // WebView2 on Windows), both of which are modern - no legacy downleveling.
    target: "baseline-widely-available",
    sourcemap: false,
  },
});
