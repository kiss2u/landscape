import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "node:path";

import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    AutoImport({
      imports: [
        "vue",
        {
          "naive-ui": [
            "useDialog",
            "useMessage",
            "useNotification",
            "useLoadingBar",
          ],
        },
      ],
    }),
    Components({
      resolvers: [NaiveUiResolver()],
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      "*": path.resolve(""),
    },
  },
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:6300",
        changeOrigin: true,
        configure: (proxy: any, options: any) => {
          // proxy will be an instance of 'http-proxy'
        },
      },
      "/ws": {
        target: "ws://localhost:6300",
        changeOrigin: true,
        ws: true,
        rewrite: (path: any) => path.replace(/^\/ws/, ""),
      },
    },
  },
});
