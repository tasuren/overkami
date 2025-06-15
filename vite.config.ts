import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [solid(), tailwindcss()],

  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**", "**/mock-env/**"],
    },
  },

  build: {
    rollupOptions: {
      input: {
        index: "./index.html",
        "builtin-wallpapers/picture": "./src/builtin-wallpapers/picture.html",
      },
    },
  },
}));
