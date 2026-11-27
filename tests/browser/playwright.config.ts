import { defineConfig, devices } from "@playwright/test";

const externalServer = process.env.CPM_BROWSER_SERVER === "external";

export default defineConfig({
  testDir: ".",
  testMatch: "journeys.spec.ts",
  fullyParallel: false,
  retries: 0,
  workers: 1,
  timeout: 90_000,
  expect: { timeout: 15_000 },
  reporter: [["json", { outputFile: "test-results/results.json" }], ["line"]],
  outputDir: "test-results/artifacts",
  use: {
    baseURL: process.env.CPM_BASE_URL ?? "http://127.0.0.1:4173",
    trace: "on",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
    colorScheme: "dark",
  },
  webServer: externalServer ? undefined : {
    command: "/usr/bin/npm run dev",
    cwd: "../../frontend",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: true,
    timeout: 60_000,
  },
  projects: [
    { name: "desktop-chromium", use: { ...devices["Desktop Chrome"], viewport: { width: 1440, height: 960 } } },
    { name: "mobile-chromium", use: { ...devices["Pixel 7"] } },
  ],
});
