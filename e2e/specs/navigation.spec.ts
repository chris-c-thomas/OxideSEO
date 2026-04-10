/**
 * E2E tests for app navigation: sidebar, keyboard shortcuts, and Command Palette.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";

test.describe("Sidebar Navigation", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
  });

  test("renders all navigation items in sidebar", async ({ page }) => {
    const navLabels = [
      "Dashboard",
      "New Crawl",
      "Monitor",
      "Results",
      "Issues",
      "Plugins",
      "Settings",
    ];
    const nav = page.getByRole("navigation");
    for (const label of navLabels) {
      await expect(nav.getByRole("button", { name: label })).toBeVisible();
    }
  });

  test("clicking Dashboard shows dashboard view", async () => {
    await app.navigateTo("Dashboard");
    await app.expectText("No crawls yet");
  });

  test("clicking New Crawl shows config form", async ({ page }) => {
    await app.navigateTo("New Crawl");
    await expect(
      page.getByRole("heading", { name: /Configure Crawl|New Crawl/i }).first(),
    ).toBeVisible();
  });

  test("clicking Settings shows settings view", async ({ page }) => {
    await app.navigateTo("Settings");
    await expect(page.getByRole("heading", { name: /Settings/i }).first()).toBeVisible();
  });

  test("clicking Plugins shows plugins view", async ({ page }) => {
    await app.navigateTo("Plugins");
    await expect(page.getByRole("heading", { name: /Plugins/i }).first()).toBeVisible();
  });

  test("sidebar collapse and expand toggles", async ({ page }) => {
    // Click collapse button
    await page.getByRole("button", { name: "Collapse sidebar" }).click();

    // After collapse, nav label text should be hidden
    await expect(page.getByRole("button", { name: "Expand sidebar" })).toBeVisible();

    // Expand again
    await page.getByRole("button", { name: "Expand sidebar" }).click();
    await expect(page.getByRole("button", { name: "Collapse sidebar" })).toBeVisible();
  });

  test("theme toggle switches between light and dark", async ({ page }) => {
    // Default is light, button says "Dark Mode"
    const themeButton = page.getByRole("button", { name: /Dark Mode/i });
    await expect(themeButton).toBeVisible();

    await themeButton.click();
    // After clicking, should now show "Light Mode"
    await expect(page.getByRole("button", { name: /Light Mode/i })).toBeVisible();
    // Check data-theme attribute changed
    const theme = await page.locator("html").getAttribute("data-theme");
    expect(theme).toBe("dark");
  });
});

test.describe("Keyboard Shortcuts", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
  });

  // Note: uses Control (not ControlOrMeta) because Playwright's headless
  // Chromium on macOS intercepts Meta+digit as browser tab-switching shortcuts
  // before they reach the page. Control works in both headless and CI contexts.

  test("Ctrl+1 navigates to Dashboard", async ({ page }) => {
    // First navigate away
    await app.navigateTo("Settings");
    // Then use shortcut to go back
    await page.keyboard.press("Control+1");
    await app.expectText("No crawls yet");
  });

  test("Ctrl+2 navigates to New Crawl", async ({ page }) => {
    await page.keyboard.press("Control+2");
    await expect(
      page.getByRole("heading", { name: /Configure Crawl|New Crawl/i }).first(),
    ).toBeVisible();
  });

  test("Ctrl+5 navigates to Settings", async ({ page }) => {
    await page.keyboard.press("Control+5");
    await expect(page.getByRole("heading", { name: /Settings/i }).first()).toBeVisible();
  });

  test("Ctrl+N navigates to New Crawl", async ({ page }) => {
    await page.keyboard.press("Control+n");
    await expect(
      page.getByRole("heading", { name: /Configure Crawl|New Crawl/i }).first(),
    ).toBeVisible();
  });

  test("Ctrl+, navigates to Settings", async ({ page }) => {
    await page.keyboard.press("Control+,");
    await expect(page.getByRole("heading", { name: /Settings/i }).first()).toBeVisible();
  });
});

test.describe("Command Palette", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
  });

  test("Ctrl+K opens the command palette", async ({ page }) => {
    await app.openCommandPalette();
    await expect(page.getByRole("dialog")).toBeVisible();
  });

  test("Escape closes the command palette", async ({ page }) => {
    await app.openCommandPalette();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog")).toBeHidden();
  });

  test("selecting 'Go to Dashboard' navigates to dashboard", async ({ page }) => {
    await app.openCommandPalette();
    await page.getByPlaceholder(/search|type/i).fill("Dashboard");
    await page.getByText("Go to Dashboard").click();
    await app.expectText("No crawls yet");
  });

  test("selecting 'Go to Settings' navigates to settings", async ({ page }) => {
    await app.openCommandPalette();
    await page.getByPlaceholder(/search|type/i).fill("Settings");
    await page.getByText("Go to Settings").click();
    await expect(page.getByRole("heading", { name: /Settings/i }).first()).toBeVisible();
  });

  test("selecting 'Toggle Theme' changes theme", async ({ page }) => {
    await app.openCommandPalette();
    // cmdk filters on the value prop (cmd.id = "theme:toggle") and keywords
    await page.getByPlaceholder(/search|type/i).fill("theme");
    await page.getByText("Toggle Theme").click();
    // Command execution is deferred via requestAnimationFrame
    await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  });

  test("typing filters commands", async ({ page }) => {
    await app.openCommandPalette();
    await page.getByPlaceholder(/search|type/i).fill("dashboard");
    await expect(page.getByText("Go to Dashboard")).toBeVisible();
    // Non-matching commands should be filtered out
    await expect(page.getByText("Go to Settings")).toBeHidden();
  });
});
