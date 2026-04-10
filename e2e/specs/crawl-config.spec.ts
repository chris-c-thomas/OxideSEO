/**
 * E2E tests for the Crawl Configuration form.
 */

import { test, expect } from "@playwright/test";
import { TauriMockBuilder } from "../helpers/mock-builder";
import { AppHelper } from "../helpers/app";

test.describe("Crawl Config Form", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("New Crawl");
  });

  test("renders form with heading", async ({ page }) => {
    await expect(
      page.getByRole("heading", { name: "Configure Crawl" }),
    ).toBeVisible();
    await expect(
      page.getByText("Set the target URL and tune crawler parameters."),
    ).toBeVisible();
  });

  test("Start URL field has placeholder", async ({ page }) => {
    await expect(
      page.getByPlaceholder("https://example.com"),
    ).toBeVisible();
  });

  test("shows default values for numeric fields", async ({ page }) => {
    // Max Depth field is next to its label text
    const maxDepthSection = page.getByText("Max Depth").locator("..");
    const maxDepthInput = maxDepthSection.locator("input");
    await expect(maxDepthInput).toHaveValue("10");
  });

  test("Respect robots.txt toggle is checked by default", async ({ page }) => {
    // The Switch has an htmlFor="respect-robots" that links to the toggle
    const toggle = page.locator("#respect-robots");
    await expect(toggle).toBeChecked();
  });

  test("submitting with empty URL shows validation error", async ({
    page,
  }) => {
    await page.getByRole("button", { name: "Start Crawl" }).click();
    // Zod validation should show an error for startUrl
    await expect(page.getByText(/URL|url|required/i).first()).toBeVisible();
  });

  test("successful submit navigates to monitor", async ({ page }) => {
    await page.getByPlaceholder("https://example.com").fill("https://test.com");
    await page.getByRole("button", { name: "Start Crawl" }).click();
    // Should navigate to crawl monitor
    await expect(
      page.getByRole("heading", { name: "Crawl Monitor" }),
    ).toBeVisible();
  });

  test("Cancel button navigates back to dashboard", async ({ page }) => {
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(
      page.getByRole("heading", { name: "Dashboard" }),
    ).toBeVisible();
  });
});

test.describe("Crawl Config Advanced Sections", () => {
  let app: AppHelper;

  test.beforeEach(async ({ page }) => {
    app = new AppHelper(page);
    const mocks = new TauriMockBuilder().build();
    await app.setup(mocks);
    await app.navigateTo("New Crawl");
  });

  test("Custom Headers section expands and collapses", async ({ page }) => {
    const trigger = page.getByRole("button", { name: "Custom Headers" });
    await trigger.click();
    await expect(
      page.getByText("Inject custom HTTP headers"),
    ).toBeVisible();

    // Collapse
    await trigger.click();
    await expect(
      page.getByText("Inject custom HTTP headers"),
    ).toBeHidden();
  });

  test("Cookies section expands", async ({ page }) => {
    await page.getByRole("button", { name: "Cookies" }).click();
    await expect(
      page.getByText("Pre-seed cookies for authenticated crawling"),
    ).toBeVisible();
  });

  test("URL Include / Exclude Patterns section expands", async ({ page }) => {
    await page
      .getByRole("button", { name: /URL Include.*Exclude/i })
      .click();
    await expect(
      page.getByText("Regex patterns to control which URLs"),
    ).toBeVisible();
  });

  test("JavaScript Rendering section shows toggle", async ({ page }) => {
    await page
      .getByRole("button", { name: "JavaScript Rendering" })
      .click();
    await expect(
      page.getByText("Enable JS rendering for SPA detection"),
    ).toBeVisible();
  });

  test("enabling JS rendering reveals sub-options", async ({ page }) => {
    await page
      .getByRole("button", { name: "JavaScript Rendering" })
      .click();
    // Click the Switch toggle (linked via htmlFor="js-rendering")
    await page.locator("#js-rendering").click();
    await expect(page.getByText("Max Concurrent Webviews")).toBeVisible();
  });

  test("Sitemap & External Links section has toggles", async ({ page }) => {
    await page
      .getByRole("button", { name: /Sitemap.*External/i })
      .click();
    await expect(
      page.getByText("Enable sitemap auto-discovery"),
    ).toBeVisible();
    await expect(
      page.getByText(/Check external links/i),
    ).toBeVisible();
  });
});
