/**
 * E2E tests for the Settings view.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";
import { makeAiConfig } from "../fixtures/settings-data";

test.describe("Settings Sub-Navigation", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("Settings");
  });

  test("renders settings sub-navigation sections", async ({ page }) => {
    // The settings sub-nav buttons are within the main content area
    const main = page.getByRole("main");
    await expect(main.getByRole("button", { name: "General" })).toBeVisible();
    await expect(main.getByRole("button", { name: "Appearance" })).toBeVisible();
    await expect(main.getByRole("button", { name: "AI Providers" })).toBeVisible();
    await expect(main.getByRole("button", { name: "About" })).toBeVisible();
  });

  test("clicking General shows general settings", async ({ page }) => {
    const main = page.getByRole("main");
    await main.getByRole("button", { name: "General" }).click();
    await expect(page.getByText("Default Export Format")).toBeVisible();
  });

  test("clicking Appearance shows appearance settings", async ({ page }) => {
    const main = page.getByRole("main");
    await main.getByRole("button", { name: "Appearance" }).click();
    await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
  });

  test("clicking AI Providers shows AI settings", async ({ page }) => {
    const main = page.getByRole("main");
    await main.getByRole("button", { name: "AI Providers" }).click();
    await expect(page.getByRole("heading", { name: "AI Providers" })).toBeVisible();
  });

  test("clicking About shows version info", async ({ page }) => {
    const main = page.getByRole("main");
    await main.getByRole("button", { name: "About" }).click();
    await expect(main.getByText("OxideSEO")).toBeVisible();
  });
});

test.describe("Settings Appearance", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("Settings");
    await page.getByRole("main").getByRole("button", { name: "Appearance" }).click();
  });

  test("theme selector shows current theme", async ({ page }) => {
    await expect(page.getByText("Theme")).toBeVisible();
  });

  test("changing to dark theme applies data-theme attribute", async ({ page }) => {
    // Find and click the dark theme option
    const darkButton = page.getByRole("button", { name: /Dark/i }).first();
    await darkButton.click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  });

  test("table density section is visible", async ({ page }) => {
    await expect(page.getByText("Table Density")).toBeVisible();
  });
});

test.describe("Settings General", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("Settings");
  });

  test("displays Default Export Format", async ({ page }) => {
    await expect(page.getByText("Default Export Format")).toBeVisible();
  });

  test("Save Settings button is visible", async ({ page }) => {
    await expect(page.getByRole("button", { name: /Save Settings/i })).toBeVisible();
  });
});

test.describe("Settings AI Providers", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder()
      .withAiConfig(makeAiConfig({ providerType: "open_ai", isConfigured: false }))
      .build();
    await app.setup(mocks);
    await app.navigateTo("Settings");
    await page.getByRole("main").getByRole("button", { name: "AI Providers" }).click();
  });

  test("shows provider selector", async ({ page }) => {
    await expect(page.getByText("Provider", { exact: true })).toBeVisible();
  });

  test("shows API key section", async ({ page }) => {
    await expect(page.getByText(/API Key/i).first()).toBeVisible();
  });

  test("Test Connection button is visible", async ({ page }) => {
    await expect(page.getByRole("button", { name: /Test Connection/i })).toBeVisible();
  });
});
