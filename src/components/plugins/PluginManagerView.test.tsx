import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { PluginManagerView } from "./PluginManagerView";
import type { PluginInfo, PluginDetail } from "@/types";

const mockPlugins: PluginInfo[] = [
  {
    name: "schema-validator",
    version: "0.1.0",
    description: "Validates JSON-LD structured data",
    kind: "rule",
    enabled: true,
    isNative: false,
    loadError: null,
  },
  {
    name: "markdown-exporter",
    version: "1.0.0",
    description: "Export reports as Markdown",
    kind: "exporter",
    enabled: false,
    isNative: true,
    loadError: null,
  },
];

const mockDetail: PluginDetail = {
  name: "schema-validator",
  version: "0.1.0",
  description: "Validates JSON-LD structured data",
  author: "OxideSEO Contributors",
  license: "MIT",
  kind: "rule",
  capabilities: ["log"],
  enabled: true,
  isNative: false,
  config: { module: "schema_validator.wasm" },
  installedAt: "2026-04-07 10:00:00",
};

function mockInvoke(overrides: Record<string, unknown> = {}) {
  vi.mocked(invoke).mockImplementation(async (cmd: string) => {
    if (cmd in overrides) return overrides[cmd];
    switch (cmd) {
      case "list_plugins":
        return mockPlugins;
      case "get_plugin_detail":
        return mockDetail;
      default:
        return undefined;
    }
  });
}

describe("PluginManagerView", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("shows loading spinner initially", () => {
    vi.mocked(invoke).mockReturnValue(new Promise(() => {}));
    render(<PluginManagerView />);
    expect(screen.getByRole("status", { name: "Loading" })).toBeTruthy();
  });

  it("renders plugin list after loading", async () => {
    mockInvoke();
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("schema-validator")).toBeTruthy();
      expect(screen.getByText("markdown-exporter")).toBeTruthy();
    });
  });

  it("shows kind badges for plugins", async () => {
    mockInvoke();
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("Rule")).toBeTruthy();
      expect(screen.getByText("Exporter")).toBeTruthy();
    });
  });

  it("shows native badge for native plugins", async () => {
    mockInvoke();
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("Native")).toBeTruthy();
    });
  });

  it("shows empty state when no plugins", async () => {
    mockInvoke({ list_plugins: [] });
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("No plugins installed")).toBeTruthy();
    });
  });

  it("calls enable_plugin when toggling off-to-on", async () => {
    mockInvoke();
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("markdown-exporter")).toBeTruthy();
    });

    // Find the disabled toggle (markdown-exporter is disabled)
    const switches = screen.getAllByRole("switch");
    const disabledSwitch = switches.find(
      (s) => s.getAttribute("aria-checked") === "false",
    );
    expect(disabledSwitch).toBeTruthy();

    fireEvent.click(disabledSwitch!);

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("enable_plugin", {
        name: "markdown-exporter",
      });
    });
  });

  it("calls disable_plugin when toggling on-to-off", async () => {
    mockInvoke();
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("schema-validator")).toBeTruthy();
    });

    const switches = screen.getAllByRole("switch");
    const enabledSwitch = switches.find((s) => s.getAttribute("aria-checked") === "true");
    expect(enabledSwitch).toBeTruthy();

    fireEvent.click(enabledSwitch!);

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("disable_plugin", {
        name: "schema-validator",
      });
    });
  });

  it("calls reload_plugins when Reload button clicked", async () => {
    mockInvoke({ reload_plugins: mockPlugins });
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("Reload")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Reload"));

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("reload_plugins");
    });
  });

  it("opens detail sheet when plugin card clicked", async () => {
    mockInvoke();
    render(<PluginManagerView />);

    await waitFor(() => {
      expect(screen.getByText("schema-validator")).toBeTruthy();
    });

    // Click the card (it's a button role)
    const cards = screen.getAllByRole("button");
    const pluginCard = cards.find(
      (c) =>
        c.textContent?.includes("schema-validator") && c.getAttribute("tabindex") === "0",
    );
    expect(pluginCard).toBeTruthy();

    fireEvent.click(pluginCard!);

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("get_plugin_detail", {
        name: "schema-validator",
      });
    });
  });
});
