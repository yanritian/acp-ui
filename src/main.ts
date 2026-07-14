import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import { i18n, loadInitialLocale } from "./locales";
import { router } from "./router";

// Initialize MCP bridge for E2E testing (dev mode only)
// Disabled until tauri-plugin-mcp build issues are resolved
// if (import.meta.env.DEV) {
//   import('tauri-plugin-mcp').then(({ initMcpBridge }) => {
//     initMcpBridge().catch(err => {
//       console.warn('[MCP] Bridge initialization failed:', err);
//     });
//   });
// }

async function bootstrap() {
  await loadInitialLocale()

  const app = createApp(App);
  const pinia = createPinia();

  app.use(pinia);
  app.use(i18n);
  app.use(router);
  app.mount("#app");
}

void bootstrap();
