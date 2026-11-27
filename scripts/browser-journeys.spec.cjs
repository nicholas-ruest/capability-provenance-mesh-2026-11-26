const fs = require("fs");
const cp = require("child_process");
const { test, expect } = require(
  `${process.env.DREAM_MACHINE_PLAYWRIGHT_ROOT}/node_modules/@playwright/test`,
);

const evidenceDir = process.env.DREAM_MACHINE_EVIDENCE_DIR;
const correlationTrace = `${evidenceDir}/correlation-trace.ndjson`;
const creditedUpstreams = [
  ["ruvnet/autogenous", "3155111ae78039c3993a544044a9cac041090ea6"],
  ["ruvnet/ruGraph", "d66d976c0ba8fa792fcf024c2a07b51480e4fbf1"],
  ["ruvnet/agenticow", "a6467863e250371562c2e0c83f5f993459607746"],
  ["ruvnet/ai-text-watermark", "5871cf5fef6547c97626ba72d976f167562dab2a"],
  ["ruvnet/LatentMesh", "b04a70523196181aadc6d694e02de1f5722fae6b"],
  ["NVIDIA/SkillSpector", "48e0f942ea1f5fdb7aa096a6f65a0adeda2ecc4e"],
  ["sigstore/cosign", "252f10a80ba871152a12e5ee324ad1e02c85d079"],
];

fs.mkdirSync(evidenceDir, { recursive: true });
function appendEvidence(entry) {
  fs.appendFileSync(
    correlationTrace,
    `${JSON.stringify({ observed_at: new Date().toISOString(), ...entry })}\n`,
  );
}
function responsePath(response) {
  return new URL(response.url()).pathname;
}
function observe(page, journey) {
  const pending = new Set();
  page.on("response", (response) => {
    const pathname = responsePath(response);
    if (!pathname.startsWith("/api/") && pathname !== "/readyz") return;
    const task = (async () => {
      let body = "";
      try {
        body = (await response.text()).slice(0, 16_384);
      } catch {
        body = "<response body unavailable>";
      }
      appendEvidence({
        kind: "http_response",
        journey,
        method: response.request().method(),
        path: pathname,
        status: response.status(),
        request_correlation_id:
          response.request().headers()["x-correlation-id"] || null,
        response_correlation_id:
          response.headers()["x-correlation-id"] || null,
        body,
      });
    })();
    pending.add(task);
    void task.finally(() => pending.delete(task));
  });
  return {
    async flush() {
      await Promise.allSettled([...pending]);
    },
  };
}
async function expectServiceReady(page) {
  await expect
    .poll(
      () =>
        page.evaluate(async () => {
          try {
            return (await fetch("/readyz", { cache: "no-store" })).ok;
          } catch {
            return false;
          }
        }),
      { timeout: 90_000 },
    )
    .toBe(true);
  if ((page.viewportSize()?.width ?? 1280) >= 1024) {
    await expect(page.getByText("Service ready", { exact: true })).toBeVisible();
  }
}
async function openSection(page, label) {
  const navigation = page.getByRole("navigation", { name: "Product sections" });
  const button = navigation.getByRole("button", {
    name: new RegExp(`^${label}\\b`, "i"),
  });
  await button.scrollIntoViewIfNeeded();
  await button.click();
  await expect(
    page.getByRole("heading", { level: 1, name: label, exact: true }),
  ).toBeVisible();
}
async function submitAdmission(page) {
  const responsePromise = page.waitForResponse(
    (response) =>
      responsePath(response) === "/api/v1/intakes" &&
      response.request().method() === "POST",
  );
  await page.getByRole("button", { name: "Run governed admission" }).click();
  const response = await responsePromise;
  expect(response.status()).toBe(200);
  const record = await response.json();
  expect(record.admitted).toBe(true);
  expect(record.upstream_receipts).toHaveLength(creditedUpstreams.length);
  return record;
}
function expectCreditedReceipts(record) {
  for (const [project, revision] of creditedUpstreams) {
    expect(
      record.upstream_receipts.some((receipt) =>
        receipt.startsWith(`${project}@${revision}:`),
      ),
    ).toBe(true);
  }
}
test.describe.configure({ mode: "serial" });
test("[journey:primary] admits through real product stack", async ({ page }) => {
  const recorder = observe(page, "primary");
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/");
  await expectServiceReady(page);

  const version = `${Math.floor(Date.now() / 1_000)}.1`;
  await page.getByLabel("Version", { exact: true }).fill(version);
  const record = await submitAdmission(page);
  expectCreditedReceipts(record);

  await expect(page.getByTestId("decision-receipt")).toBeVisible();
  await expect(page.getByText("7/7", { exact: true })).toBeVisible();
  await openSection(page, "Evidence mesh");

  const evidenceGrid = page.getByTestId("upstream-evidence");
  await expect(evidenceGrid.locator("article")).toHaveCount(7);
  for (const [project, revision] of creditedUpstreams) {
    const card = evidenceGrid.locator("article").filter({ hasText: project });
    await expect(card).toHaveCount(1);
    await expect(card).toContainText(revision);
    await expect(card).toContainText("verified");
  }

  await page.screenshot({ path: `${evidenceDir}/desktop.png`, fullPage: true });
  await recorder.flush();
});
test("[journey:denial-recovery] prevents forbidden effects and recovers", async ({ page }) => {
  const recorder = observe(page, "denial-recovery");
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/");
  await expectServiceReady(page);

  const deniedVersion = `${Math.floor(Date.now() / 1_000)}.2`;
  await page.getByLabel("Version", { exact: true }).fill(deniedVersion);
  const denialResponsePromise = page.waitForResponse(
    (response) =>
      responsePath(response) === "/api/v1/intakes" &&
      response.request().method() === "POST",
  );
  const absentEffectPromise = page.waitForResponse(
    (response) =>
      responsePath(response) ===
        `/api/v1/registry/image-transform/${deniedVersion}` &&
      response.request().method() === "GET",
  );
  await page.getByRole("button", { name: "Run policy-denial probe" }).click();

  const denialResponse = await denialResponsePromise;
  expect(denialResponse.status()).toBe(200);
  expect((await denialResponse.json()).admitted).toBe(false);
  const absentEffectResponse = await absentEffectPromise;
  expect(absentEffectResponse.status()).toBe(404);
  await expect(page.getByTestId("denial-proof")).toBeVisible();
  await expect(page.getByText(/404 confirms no downstream mutation/)).toBeVisible();

  await page.getByRole("button", { name: "Recover and continue" }).click();
  await expect(page.getByText(
    "Recovered. Submit the corrected capability.", { exact: true },
  )).toBeVisible();

  const recoveredVersion = `${Math.floor(Date.now() / 1_000)}.3`;
  await page.getByLabel("Version", { exact: true }).fill(recoveredVersion);
  const recoveredRecord = await submitAdmission(page);
  expectCreditedReceipts(recoveredRecord);
  await expect(page.getByTestId("decision-receipt")).toContainText("ADMIT");
  await expect(page.getByText(
    "Admitted with a source-bound receipt and durable decision.", { exact: true },
  )).toBeVisible();
  await recorder.flush();
});
test("[journey:durable-retrieval] survives refresh and backend restart with audit replay", async ({
  page,
}) => {
  test.setTimeout(240_000);
  const recorder = observe(page, "durable-retrieval");
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");
  await expectServiceReady(page);

  const plugin = "durable-browser-capability";
  const version = `${Math.floor(Date.now() / 1_000)}.4`;
  await page.getByLabel("Plugin name", { exact: true }).fill(plugin);
  await page.getByLabel("Version", { exact: true }).fill(version);
  const original = await submitAdmission(page);
  expectCreditedReceipts(original);

  await page.reload();
  await expectServiceReady(page);
  await expect(page.getByTestId("decision-receipt")).toContainText(
    original.correlation_id,
  );

  const restartOutput = cp.execFileSync("scripts/restart-api.sh", {
    encoding: "utf8",
    env: process.env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  appendEvidence({
    kind: "backend_restart",
    journey: "durable-retrieval",
    output: restartOutput.trim(),
  });

  await page.reload();
  await expectServiceReady(page);
  await expect(page.getByTestId("decision-receipt")).toContainText(
    original.correlation_id,
  );

  await openSection(page, "Registry");
  await page.getByLabel("Plugin", { exact: true }).fill(plugin);
  await page.getByLabel("Version", { exact: true }).fill(version);
  const retrievalResponsePromise = page.waitForResponse(
    (response) =>
      responsePath(response) ===
        `/api/v1/registry/${encodeURIComponent(plugin)}/${encodeURIComponent(version)}` &&
      response.request().method() === "GET",
  );
  await page.getByRole("button", { name: "Retrieve from registry" }).click();
  const retrievalResponse = await retrievalResponsePromise;
  expect(retrievalResponse.status()).toBe(200);
  const retrieved = await retrievalResponse.json();
  expect(retrieved.correlation_id).toBe(original.correlation_id);
  expect(retrieved.receipt_checksum).toBe(original.receipt_checksum);
  await expect(page.getByText(
    "Durable decision retrieved from shared storage.", { exact: true },
  )).toBeVisible();

  await openSection(page, "Audit replay");
  await page
    .getByLabel("Correlation ID", { exact: true })
    .fill(original.correlation_id);
  const auditResponsePromise = page.waitForResponse(
    (response) =>
      responsePath(response) ===
        `/api/v1/audit/${encodeURIComponent(original.correlation_id)}` &&
      response.request().method() === "GET",
  );
  await page.getByRole("button", { name: "Replay audit" }).click();
  const auditResponse = await auditResponsePromise;
  expect(auditResponse.status()).toBe(200);
  const replayed = await auditResponse.json();
  expect(replayed.correlation_id).toBe(original.correlation_id);
  expect(replayed.receipt_checksum).toBe(original.receipt_checksum);
  await expect(page.getByText(
    "Audit replay matches the durable receipt.", { exact: true },
  )).toBeVisible();
  await expect(page.getByTestId("decision-receipt")).toContainText(
    original.correlation_id,
  );

  const mobileLayout = await page.evaluate(() => ({
    viewportWidth: document.documentElement.clientWidth,
    documentScrollWidth: document.documentElement.scrollWidth,
    bodyScrollWidth: document.body.scrollWidth,
  }));
  expect(
    Math.max(mobileLayout.documentScrollWidth, mobileLayout.bodyScrollWidth),
    `mobile layout overflowed: ${JSON.stringify(mobileLayout)}`,
  ).toBeLessThanOrEqual(mobileLayout.viewportWidth);
  await page.screenshot({ path: `${evidenceDir}/mobile.png`, fullPage: true });
  await recorder.flush();
});
