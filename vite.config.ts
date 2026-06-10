import { defineConfig } from "vite";
/// <reference types="vitest" />
import vue from "@vitejs/plugin-vue";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

// Read the package version once at config-evaluation time so we can inject
// it into the web build (the Tauri build reads it from `tauri.conf.json`
// instead via `@tauri-apps/api/app#getVersion`).
const pkg = JSON.parse(
  readFileSync(fileURLToPath(new URL("./package.json", import.meta.url)), "utf-8")
) as { version: string };

// https://vite.dev/config/
export default defineConfig(async ({ mode }) => {
  const isWeb = mode === "web";

  return {
    plugins: [vue()],

    resolve: {
      alias: {
        '@': path.resolve(__dirname, './src')
      }
    },

    test: {
      environment: 'jsdom',
      setupFiles: ['src/test/setup.ts'],
      globals: true,
      include: ['**/*.test.?(c|m)[jt]s?(x)'],
      exclude: ['**/tests/functional/**', '**/node_modules/**', '**/dist/**'],
    },

    define: {
      // Exposed to the frontend as `import.meta.env.VITE_APP_VERSION`. The
      // host abstraction (`src/lib/host/index.ts`) reads this on the web
      // build and falls back to it when `@tauri-apps/api/app` is unavailable.
      "import.meta.env.VITE_APP_VERSION": JSON.stringify(pkg.version),
    },

    // 代码分割优化 - 解决大文件警告
    build: {
      // Web builds emit to `dist-web/` so the Tauri build pipeline (which
      // expects `frontendDist: ../dist`) is unaffected.
      ...(isWeb ? { outDir: "dist-web", emptyOutDir: true } : {}),
      rollupOptions: {
        output: {
          // 使用函数形式的manualChunks进行动态代码分割
          manualChunks(id) {
            // Vue核心（包含vue和vue-router）
            if (id.includes('node_modules/vue/') || id.includes('node_modules/@vue/') || id.includes('node_modules/vue-router/')) {
              return 'vue-vendor';
            }
            // Pinia状态管理
            if (id.includes('node_modules/pinia/')) {
              return 'pinia';
            }
            // Vue Flow图形库
            if (id.includes('node_modules/@vue-flow/')) {
              return 'vue-flow';
            }
            // Tauri API
            if (id.includes('node_modules/@tauri-apps/')) {
              return 'tauri';
            }
            // 常用大型库单独分组
            if (id.includes('node_modules/marked/')) {
              return 'vendor-marked';
            }
            if (id.includes('node_modules/dompurify/')) {
              return 'vendor-dompurify';
            }
            // 协作网络 + Hermes 模块合并（避免循环依赖）
            if (id.includes('/lib/collaboration/') ||
                id.includes('/stores/collaboration') ||
                id.includes('/components/collaboration/') ||
                id.includes('/components/HermesDashboard') ||
                id.includes('/components/EnhancedHermesDashboard') ||
                id.includes('/components/TaskGraphView')) {
              return 'collaboration';
            }
            // Agent相关模块合并
            if (id.includes('/components/agent-progress/') ||
                id.includes('/components/agent-pet/') ||
                id.includes('/stores/agent-realtime') ||
                id.includes('/stores/agent-pet') ||
                id.includes('/lib/agent-runtime/realtime-progress-types') ||
                id.includes('/views/AgentTeamsDashboard')) {
              return 'agent-teams';
            }
            // WebSocket同步模块
            if (id.includes('/lib/sync/')) {
              return 'sync';
            }
          },
        },
      },
      chunkSizeWarningLimit: 400,
    },

    // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
    //
    // 1. prevent Vite from obscuring rust errors
    clearScreen: false,
    // 2. tauri expects a fixed port, fail if that port is not available
    server: isWeb
      ? {
          // Browser dev server: use Vite defaults so it works behind common
          // proxies / Dev Tunnels without the strict-port behaviour Tauri
          // requires.
          port: 5173,
        }
      : {
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
            // 3. tell Vite to ignore watching `src-tauri`
            ignored: ["**/src-tauri/**"],
          },
        },
  };
});