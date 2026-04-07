import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { SettingsView } from "./SettingsView";
import type { AiProviderConfig, AppSettings } from "@/types";

const mockConfig: AiProviderConfig = {
  providerType: "open_ai",
  model: "gpt-4o",
  ollamaEndpoint: null,
  maxTokensPerCrawl: 100000,
  isConfigured: true,
};

const mockSettings: AppSettings = {
  defaultCrawlConfig: {},
  theme: "system",
  defaultExportFormat: "csv",
};

function mockInvoke(overrides: Record<string, unknown> = {}) {
  vi.mocked(invoke).mockImplementation(async (cmd: string) => {
    if (cmd in overrides) return overrides[cmd];
    switch (cmd) {
      case "get_settings":
        return mockSettings;
      case "get_ai_config":
        return mockConfig;
      case "has_api_key":
        return true;
      default:
        return undefined;
    }
  });
}

describe("AiProviderSection", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("renders provider dropdown with three options", async () => {
    mockInvoke();
    render(<SettingsView />);

    await waitFor(() => {
      expect(screen.getByText("AI Providers")).toBeTruthy();
    });

    const select = screen.getByDisplayValue("OpenAI") as HTMLSelectElement;
    expect(select.options).toHaveLength(3);
  });

  it("hides API key section when Ollama is selected", async () => {
    mockInvoke({
      get_ai_config: { ...mockConfig, providerType: "ollama", model: "llama3" },
      has_api_key: true,
    });
    render(<SettingsView />);

    await waitFor(() => {
      expect(screen.getByText("AI Providers")).toBeTruthy();
    });

    expect(screen.queryByText("API Key")).toBeNull();
    expect(screen.getByText("Ollama Endpoint")).toBeTruthy();
  });

  it("shows API key section for OpenAI", async () => {
    mockInvoke();
    render(<SettingsView />);

    await waitFor(() => {
      expect(screen.getByText(/API Key/)).toBeTruthy();
      expect(screen.getByText("(stored in OS keychain)")).toBeTruthy();
    });
  });

  it("calls testAiConnection when test button is clicked", async () => {
    mockInvoke({ test_ai_connection: "Connected to OpenAI (model: gpt-4o)" });
    render(<SettingsView />);

    await waitFor(() => {
      expect(screen.getByText("Test Connection")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Test Connection"));

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("test_ai_connection");
    });
  });

  it("shows key not set when provider has no key", async () => {
    mockInvoke({
      get_ai_config: { ...mockConfig, isConfigured: false },
      has_api_key: false,
    });
    render(<SettingsView />);

    await waitFor(() => {
      expect(screen.getByText("(not set)")).toBeTruthy();
    });
  });
});
