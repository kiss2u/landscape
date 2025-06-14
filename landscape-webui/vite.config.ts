import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "node:path";
import { readFileSync } from "fs";

import basicSsl from "@vitejs/plugin-basic-ssl";
import AutoImport from "unplugin-auto-import/vite";
import Components from "unplugin-vue-components/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";

const HOST_PORT = "localhost:6443";

const pkg = JSON.parse(readFileSync("./package.json", "utf-8"));

// https://vitejs.dev/config/
export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(pkg.version),
  },
  plugins: [
    basicSsl(),
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
        target: `https://${HOST_PORT}`,
        changeOrigin: true,
        secure: false,
        configure: (proxy: any, options: any) => {
          // proxy will be an instance of 'http-proxy'
        },
      },
      "/ws": {
        target: `ws://${HOST_PORT}`,
        changeOrigin: true,
        ws: true,
        rewrite: (path: any) => path.replace(/^\/ws/, ""),
      },
    },
  },
});
