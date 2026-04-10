/**
 * Core Tauri IPC mock for Playwright E2E tests.
 *
 * Injects a mock `window.__TAURI_INTERNALS__` via `page.addInitScript()`
 * before the React app loads. This intercepts all `invoke()` calls from
 * `@tauri-apps/api/core` and returns pre-configured mock responses.
 *
 * Based on the official `@tauri-apps/api/mocks` pattern, adapted for
 * Playwright's browser context injection.
 */

import type { Page } from "@playwright/test";

export type MockCommandMap = Record<string, unknown>;

/**
 * Set up Tauri IPC mocks for a Playwright page.
 *
 * Must be called BEFORE `page.goto()` so the init script runs before
 * the app's JavaScript modules load.
 *
 * @param page - Playwright page instance
 * @param commands - Map of command names to their mock return values
 */
export async function setupTauriMocks(
  page: Page,
  commands: MockCommandMap,
): Promise<void> {
  await page.addInitScript((commandMap: Record<string, unknown>) => {
    // Initialize Tauri internals objects
    const internals =
      (window as Record<string, unknown>).__TAURI_INTERNALS__ ??
      ({} as Record<string, unknown>);
    (window as Record<string, unknown>).__TAURI_INTERNALS__ = internals;
    (window as Record<string, unknown>).__TAURI_EVENT_PLUGIN_INTERNALS__ =
      (window as Record<string, unknown>).__TAURI_EVENT_PLUGIN_INTERNALS__ ??
      {};

    // Callback registry (mirrors the official mocks.js, with events always
    // enabled since E2E tests require event simulation)
    const callbacks = new Map<
      number,
      (data: unknown) => unknown | undefined
    >();

    function registerCallback(
      callback: ((data: unknown) => void) | null,
      once = false,
    ): number {
      const id = crypto.getRandomValues(new Uint32Array(1))[0]!;
      callbacks.set(id, (data: unknown) => {
        if (once) callbacks.delete(id);
        return callback?.(data);
      });
      return id;
    }

    function unregisterCallback(id: number): void {
      callbacks.delete(id);
    }

    function runCallback(id: number, data: unknown): void {
      const cb = callbacks.get(id);
      if (cb) {
        cb(data);
      }
    }

    // Event listener registry
    const eventListeners = new Map<string, number[]>();

    function handleListen(args: {
      event: string;
      handler: number;
    }): number {
      if (!eventListeners.has(args.event)) {
        eventListeners.set(args.event, []);
      }
      eventListeners.get(args.event)!.push(args.handler);
      return args.handler;
    }

    function handleEmit(args: { event: string; payload?: unknown }): null {
      const listeners = eventListeners.get(args.event) ?? [];
      for (const handlerId of listeners) {
        runCallback(handlerId, {
          event: args.event,
          id: handlerId,
          payload: args.payload,
        });
      }
      return null;
    }

    function handleUnlisten(args: { event: string; id: number }): void {
      const listeners = eventListeners.get(args.event);
      if (listeners) {
        const idx = listeners.indexOf(args.id);
        if (idx !== -1) listeners.splice(idx, 1);
      }
      unregisterCallback(args.id);
    }

    // Main invoke handler
    async function invoke(
      cmd: string,
      args: Record<string, unknown>,
      _options?: unknown,
    ): Promise<unknown> {
      // Handle event plugin commands
      if (cmd === "plugin:event|listen") {
        return handleListen(args as { event: string; handler: number });
      }
      if (cmd === "plugin:event|emit") {
        return handleEmit(args as { event: string; payload?: unknown });
      }
      if (cmd === "plugin:event|unlisten") {
        handleUnlisten(args as { event: string; id: number });
        return undefined;
      }

      // Look up mock response
      if (cmd in commandMap) {
        const response = commandMap[cmd];
        // Support error simulation: if the value is an object with
        // __mockError__, throw it as an error string (matching Tauri's
        // Result<T, String> IPC error format).
        if (
          response !== null &&
          typeof response === "object" &&
          "__mockError__" in (response as Record<string, unknown>)
        ) {
          throw (response as { __mockError__: string }).__mockError__;
        }
        // Return a deep copy to prevent cross-test mutation
        return response === undefined || response === null
          ? response
          : JSON.parse(JSON.stringify(response));
      }

      throw new Error(
        `[E2E Mock] Unhandled Tauri command: "${cmd}". ` +
        `Add it to TauriMockBuilder.withDefaults() or use .withCommand("${cmd}", ...) in your test.`,
      );
    }

    // Wire everything up
    const tauriInternals = internals as Record<string, unknown>;
    tauriInternals.invoke = invoke;
    tauriInternals.transformCallback = registerCallback;
    tauriInternals.unregisterCallback = unregisterCallback;
    tauriInternals.runCallback = runCallback;
    tauriInternals.callbacks = callbacks;

    // Mock window metadata
    tauriInternals.metadata = {
      currentWindow: { label: "main" },
      currentWebview: { windowLabel: "main", label: "main" },
    };

    // Expose event emitter for test code to call via page.evaluate()
    (window as Record<string, unknown>).__E2E_EMIT_EVENT__ = (
      event: string,
      payload: unknown,
    ): void => {
      const listeners = eventListeners.get(event) ?? [];
      for (const handlerId of listeners) {
        runCallback(handlerId, { event, id: handlerId, payload });
      }
    };

    // Mock the unregisterListener for event plugin internals
    const eventInternals = (window as Record<string, unknown>)
      .__TAURI_EVENT_PLUGIN_INTERNALS__ as Record<string, unknown>;
    eventInternals.unregisterListener = (
      _event: string,
      id: number,
    ): void => {
      unregisterCallback(id);
    };
  }, commands);
}
