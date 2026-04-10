/**
 * Page object for common app-level interactions in E2E tests.
 *
 * Encapsulates navigation, view assertions, and common UI patterns
 * so individual test specs stay focused on their specific scenarios.
 */

import { expect, type Page } from "@playwright/test";
import { setupTauriMocks, type MockCommandMap } from "../setup/tauri-mock";

export class AppHelper {
  constructor(readonly page: Page) {}

  /**
   * Initialize Tauri mocks and navigate to the app.
   * Must be called at the start of each test.
   */
  async setup(commands: MockCommandMap): Promise<void> {
    await setupTauriMocks(this.page, commands);
    await this.page.goto("/");
    // Wait for the app shell to render
    await this.page.waitForLoadState("networkidle");
  }

  /** Click a sidebar navigation item by its visible text. */
  async navigateTo(label: string): Promise<void> {
    await this.page
      .getByRole("navigation")
      .getByRole("button", { name: label })
      .click();
  }

  /** Assert that a heading with the given text is visible. */
  async expectHeading(text: string): Promise<void> {
    await expect(
      this.page.getByRole("heading", { name: text }).first(),
    ).toBeVisible();
  }

  /** Assert that specific text is visible on the page. */
  async expectText(text: string): Promise<void> {
    await expect(this.page.getByText(text).first()).toBeVisible();
  }

  /** Open the Command Palette with Ctrl+K / Cmd+K. */
  async openCommandPalette(): Promise<void> {
    await this.page.keyboard.press("Control+k");
    await this.page.getByRole("dialog").waitFor({ state: "visible" });
  }

  /** Click a button by its accessible name. */
  async clickButton(name: string): Promise<void> {
    await this.page.getByRole("button", { name }).click();
  }
}
