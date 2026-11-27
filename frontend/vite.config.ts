import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

const nodeEnv =
  (globalThis as { process?: { env?: Record<string, string | undefined> } })
    .process?.env ?? {};
const apiPort = nodeEnv.CPM_API_PORT ?? "8080";
const apiTarget =
  nodeEnv.CPM_VITE_API_TARGET ?? `http://127.0.0.1:${apiPort}`;

export default defineConfig({
  plugins: [react()],
  server: {
    host: "127.0.0.1",
    port: 4173,
    proxy: {
      "/api": apiTarget,
      "/healthz": apiTarget,
      "/readyz": apiTarget,
    },
  },
  preview: { host: "127.0.0.1", port: 4173 },
  test: { environment: "jsdom", setupFiles: "./src/test/setup.ts", css: true },
});
