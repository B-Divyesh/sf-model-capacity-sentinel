import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { readFile } from "node:fs/promises";

test("@claim:demo-sandbox opens realistic sample data without using a project API", async ({
  page,
}) => {
  const apiRequests: string[] = [];
  const origins = new Set<string>();
  page.on("request", (request) => {
    origins.add(new URL(request.url()).origin);
    if (new URL(request.url()).pathname.startsWith("/api/"))
      apiRequests.push(request.url());
  });
  await page.goto("/demo");
  await expect(page).toHaveTitle("Demo — Capacity Sentinel");
  await expect(
    page.getByText("Demo — sample data, nothing is saved"),
  ).toBeVisible();
  await expect(page.locator(".specimen").first().locator(".expand")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Open alerts" }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Recovered alerts" }),
  ).toBeVisible();
  expect(apiRequests).toEqual([]);
  expect([...origins]).toEqual([new URL(page.url()).origin]);
  expect(
    await page.evaluate(() =>
      localStorage.getItem("demo:capacity-sentinel:summary"),
    ),
  ).toContain("North America chat availability");
});

test("@claim:browser-privacy-egress loads no analytics or third-party resources", async ({
  page,
}) => {
  const requests: Array<{ method: string; origin: string; path: string }> = [];
  page.on("request", (request) => {
    const url = new URL(request.url());
    requests.push({ method: request.method(), origin: url.origin, path: url.pathname });
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Try it with sample data" }).click();
  await expect(page).toHaveURL(/\/demo$/);
  await expect(
    page.getByText("Demo — sample data, nothing is saved"),
  ).toBeVisible();

  const ownOrigin = new URL(page.url()).origin;
  expect(requests.every((request) => request.origin === ownOrigin)).toBe(true);
  expect(requests.filter((request) => request.path.startsWith("/api/"))).toEqual([]);
  expect(
    requests.filter(
      (request) =>
        request.method !== "GET" ||
        !(
          ["/", "/demo", "/sw.js", "/favicon.svg"].includes(request.path) ||
          request.path.startsWith("/assets/")
        ),
    ),
  ).toEqual([]);
});

test("@claim:demo-reset restores the shipped sample and start-for-real does not carry it into a project", async ({
  page,
}) => {
  await page.goto("/demo");
  await page
    .getByRole("button", { name: "Edit Europe structured output" })
    .click();
  await expect(page.getByRole("dialog")).toBeVisible();
  await page.getByLabel("Probe name").fill("Changed only in sample");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.locator(".specimen").nth(1).locator(".expand")).toBeVisible();
  expect(
    await page.evaluate(() =>
      localStorage.getItem("demo:capacity-sentinel:summary"),
    ),
  ).toContain("Changed only in sample");
  await page.getByRole("button", { name: "Reset demo" }).click();
  await expect(page.locator(".specimen").nth(1).locator(".expand")).toBeVisible();
  await expect(page.getByText("Changed only in sample")).toHaveCount(0);
  await page.getByRole("button", { name: "Start for real" }).click();
  await expect(page).toHaveURL(/\/$/);
  await expect(
    page.getByRole("heading", { name: "Open this project" }),
  ).toBeVisible();
});

test("@claim:sample-monitoring-output shows attributed capacity evidence and recovery", async ({
  page,
}) => {
  await page.goto("/demo");
  await expect(page.getByText(/2 consecutive capacity failures/)).toBeVisible();
  await expect(
    page.getByText(/p95 latency recovered below the 1,800 ms objective/),
  ).toBeVisible();
  await page
    .getByRole("button", { name: "Observe Europe structured output now" })
    .click();
  await expect(
    page.getByText("Observation complete for Europe structured output"),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "Open alerts" })).toHaveCount(
    0,
  );
});

test("@claim:edit-probe updates an existing probe with pointer and keyboard operation", async ({
  page,
}) => {
  await page.goto("/demo");
  const edit = page.getByRole("button", {
    name: "Edit North America chat availability",
  });
  await edit.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect(page.getByLabel("Probe name")).toBeFocused();
  await page.getByRole("dialog").getByLabel("Model").fill("chat-pro-2026-revised");
  await page.getByRole("button", { name: "Save changes" }).click();
  await expect(page.locator(".specimens")).toContainText("chat-pro-2026-revised");
});

test("@claim:csv-export downloads every sample observation as CSV", async ({
  page,
}) => {
  await page.goto("/demo");
  const downloadPromise = page.waitForEvent("download");
  await page.getByRole("button", { name: "Export CSV" }).click();
  const download = await downloadPromise;
  const downloadPath = await download.path();
  const content = await readFile(downloadPath!, "utf8");
  expect(content).toContain("probe,provider,model,started_at,latency_ms");
  expect(content?.split("\n").length).toBeGreaterThanOrEqual(4);
  expect(content).toContain("North America chat availability");
});

test("@claim:atlas-comparison-view shows the licensed 365-day comparison on the sample data", async ({
  page,
}) => {
  await page.addInitScript(() => {
    localStorage.setItem("sb_license:model-capacity-sentinel", "fixture-license");
    localStorage.setItem(
      "sb_license:model-capacity-sentinel:verdict",
      JSON.stringify({ valid: true, at: Date.now(), token: "fixture-license" }),
    );
  });
  await page.goto("/demo");
  await expect(
    page.getByRole("heading", { name: "Compare providers over time" }),
  ).toBeVisible();
  const comparison = page.locator(".atlas-table");
  await expect(comparison).toContainText("Example AI");
  await expect(comparison).toContainText("chat-pro-2026");
  await expect(comparison).toContainText("eu-json-2026");
  await expect(comparison).toContainText("Availability");
  await expect(comparison).toContainText("p95 latency");
  await expect(page.getByText("$39", { exact: true })).toBeVisible();
  await expect(page.getByText("One-time", { exact: true })).toBeVisible();
});

test("@claim:accessible-mobile-dashboard is usable by keyboard and has no serious axe issues", async ({
  page,
}) => {
  await page.goto("/demo");
  await page.keyboard.press("Tab");
  await expect(page.getByText("Skip to main content")).toBeFocused();
  const scan = await new AxeBuilder({ page }).analyze();
  expect(
    scan.violations.filter(
      (violation) =>
        violation.impact === "serious" || violation.impact === "critical",
    ),
  ).toEqual([]);
  expect(
    await page
      .locator("body")
      .evaluate((body) => body.scrollWidth <= window.innerWidth),
  ).toBe(true);
  const targetSizes = await page
    .locator(".site-head nav a, footer nav a")
    .evaluateAll((targets) =>
      targets.map((target) => {
        const box = target.getBoundingClientRect();
        return { width: box.width, height: box.height };
      }).filter((target) => target.width > 0 && target.height > 0),
    );
  expect(
    targetSizes.every((target) => target.width >= 44 && target.height >= 44),
  ).toBe(true);
});

test("@claim:route-structure gives legal routes titles and a designed 404 page", async ({
  page,
}) => {
  await page.goto("/privacy");
  await expect(page).toHaveTitle("Privacy — Capacity Sentinel");
  await expect(
    page.getByRole("heading", { level: 1, name: "Read the privacy policy" }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Terms" }).click();
  await expect(page).toHaveTitle("Terms — Capacity Sentinel");
  await expect(
    page.getByRole("heading", { level: 1, name: "Read the terms of use" }),
  ).toBeFocused();
  const missing = await page.request.get("/a-page-that-does-not-exist");
  expect(missing.status()).toBe(404);
  expect(await missing.text()).toContain("This page was not found");
});

test("@claim:access-code-rate-limit rejects repeated invalid access codes with retry guidance", async ({
  page,
}, testInfo) => {
  await page.goto("/demo");
  const clientIp =
    testInfo.project.name === "mobile" ? "198.51.100.82" : "198.51.100.81";
  const responses = await Promise.all(
    Array.from({ length: 45 }, () =>
      page.request.get("/api/summary", {
        headers: {
          Authorization: "Bearer invalid-access-code",
          "X-Forwarded-For": clientIp,
        },
      }),
    ),
  );
  const limited = responses.filter((response) => response.status() === 429);
  expect(limited.length).toBeGreaterThan(0);
  expect(Number(limited[0].headers()["retry-after"])).toBeGreaterThanOrEqual(1);
  expect(
    responses.filter((response) => response.status() === 401).length,
  ).toBeGreaterThan(0);
});

test("the demo reloads offline after its first controlled visit and its service worker accepts an update check", async ({
  browser,
}) => {
  const context = await browser.newContext();
  const page = await context.newPage();
  await page.goto("/demo");
  await page.evaluate(async () => {
    const registration = await navigator.serviceWorker.ready;
    await registration.update();
  });
  await page.reload();
  await context.setOffline(true);
  await page.reload();
  await expect(page).toHaveTitle("Demo — Capacity Sentinel");
  await expect(
    page.getByText("Demo — sample data, nothing is saved"),
  ).toBeVisible();
  await context.close();
});
