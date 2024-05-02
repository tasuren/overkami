import { defineConfig } from "@rsbuild/core";
import { pluginSvelte } from "@rsbuild/plugin-svelte";
import { pluginYaml } from "@rsbuild/plugin-yaml";

export default defineConfig({
  plugins: [pluginSvelte(), pluginYaml()],
  source: {
    entry: {
      index: "./src/main.ts",
    },
    exclude: ["**/src-tauri/**"],
    alias: {
      $lib: "./src/lib",
    },
  },
  html: {
    template: "./index.html",
  },
  server: {
    port: 1420,
    strictPort: true,
  },
});
