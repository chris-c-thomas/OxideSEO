/**
 * Test fixture factories for settings-related types.
 */

import type { AppSettings, AiProviderConfig } from "@/types";

export function makeSettings(overrides?: Partial<AppSettings>): AppSettings {
  return {
    defaultCrawlConfig: {},
    theme: "light",
    defaultExportFormat: "csv",
    ...overrides,
  };
}

export function makeAiConfig(overrides?: Partial<AiProviderConfig>): AiProviderConfig {
  return {
    providerType: "open_ai",
    model: "gpt-4o",
    ollamaEndpoint: null,
    maxTokensPerCrawl: 100000,
    isConfigured: false,
    ...overrides,
  };
}
