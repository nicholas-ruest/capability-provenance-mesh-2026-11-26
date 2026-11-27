import { execFileSync } from "node:child_process";
import { expect, Page, test, TestInfo } from "@playwright/test";

const upstreams = [
  "ruvnet/autogenous", "ruvnet/ruGraph", "ruvnet/agenticow",
  "ruvnet/ai-text-watermark", "ruvnet/LatentMesh",
  "NVIDIA/SkillSpector", "sigstore/cosign",
];

function identity(prefix: string, project: string) {
  const suffix = String(Date.now()) + "-" + project.replace(/[^a-z0-9]/gi, "-");
  return { plugin: prefix + "-" + suffix, version: "1.0." + String(Date.now() % 100000) };
}

async function prepare(page: Page, plugin: string, version: string) {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Governed Admission" })).toBeVisible();
  await page.getByLabel("Plugin name").fill(plugin);
  await page.getByLabel("Version").fill(version);
}

async function admit(page: Page) {
  await page.getByRole("button", { name: "Run governed admission" }).click();
  await expect(page.getByTestId("decision-receipt")).toContainText("ADMIT");
}

async function saveViewport(page: Page, name: string, project: string, testInfo: TestInfo) {
  const path = testInfo.outputPath(name + "-" + project + ".png");
  await page.screenshot({ path, fullPage: true });
  await testInfo.attach(name + "-" + project, { path, contentType: "image/png" });
}

test("primary outcome correlates all seven executed upstreams", async ({ page }, testInfo) => {
  const id = identity("primary", testInfo.project.name);
  await prepare(page, id.plugin, id.version);
  await admit(page);
  await page.getByRole("button", { name: /Evidence Mesh/ }).click();
  const mesh = page.getByTestId("upstream-evidence");
  await expect(mesh.locator("article")).toHaveCount(7);
  for (const upstream of upstreams) await expect(mesh).toContainText(upstream);
  await saveViewport(page, "primary", testInfo.project.name, testInfo);
});

test("policy denial proves no effect and recovers", async ({ page }, testInfo) => {
  const id = identity("denial", testInfo.project.name);
  await prepare(page, id.plugin, id.version);
  await page.getByRole("button", { name: "Run policy-denial probe" }).click();
  const proof = page.getByTestId("denial-proof");
  await expect(proof).toContainText("No forbidden downstream effect");
  await expect(proof).toContainText("403 policy denial followed by a 404 registry lookup");
  await saveViewport(page, "denial-proof", testInfo.project.name, testInfo);
  await page.getByRole("button", { name: "Recover and continue" }).click();
  await expect(page.getByRole("status")).toContainText("Recovered");
  await admit(page);
  await expect(page.getByTestId("decision-receipt")).toContainText("Protected effects1");
});

test("saved result and audit survive refresh and backend restart", async ({ page }, testInfo) => {
  const restart = process.env.CPM_BACKEND_RESTART_SCRIPT;
  if (!restart) throw new Error("CPM_BACKEND_RESTART_SCRIPT is required for durable journey evidence");
  const id = identity("durable", testInfo.project.name);
  await prepare(page, id.plugin, id.version);
  await admit(page);
  const receipt = page.getByTestId("decision-receipt");
  const correlation = await receipt.locator("dt", { hasText: "Correlation" }).locator("..").locator("dd").innerText();

  await page.reload();
  await expect(page.getByTestId("decision-receipt")).toContainText(correlation);
  execFileSync(restart, [], { stdio: "inherit", timeout: 60_000 });

  await page.getByRole("button", { name: /Durable Registry/ }).click();
  await page.getByLabel("Plugin").fill(id.plugin);
  await page.getByLabel("Version").fill(id.version);
  await page.getByRole("button", { name: "Retrieve from registry" }).click();
  await expect(page.getByTestId("decision-receipt")).toContainText(correlation);

  await page.getByRole("button", { name: /Audit Replay/ }).click();
  await page.getByLabel("Correlation ID").fill(correlation);
  await page.getByRole("button", { name: "Replay audit" }).click();
  await expect(page.getByTestId("decision-receipt")).toContainText(correlation);
  await saveViewport(page, "durable-replay", testInfo.project.name, testInfo);
});
