/**
 * Settings view: application preferences, default crawl config, AI providers.
 */

import { useEffect, useState } from "react";
import {
  getSettings,
  setSettings,
  getAiConfig,
  setAiConfig,
  setApiKey,
  deleteApiKey,
  hasApiKey,
  testAiConnection,
} from "@/lib/commands";
import type { AppSettings, AiProviderConfig, AiProviderType } from "@/types";

export function SettingsView() {
  const [settings, setLocalSettings] = useState<AppSettings | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);

  useEffect(() => {
    getSettings().then(setLocalSettings).catch(console.error);
  }, []);

  const handleSave = async () => {
    if (!settings) return;
    setIsSaving(true);
    setSaveMessage(null);
    try {
      await setSettings(settings);
      setSaveMessage("Settings saved.");
    } catch (err) {
      setSaveMessage(`Error: ${err}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="mx-auto max-w-3xl space-y-8 p-8">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted-foreground)" }}>
          Application preferences and default configurations.
        </p>
      </div>

      {/* General */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">General</h2>
        <div
          className="space-y-4 rounded-lg border p-4"
          style={{ borderColor: "var(--color-border)" }}
        >
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Default Export Format</label>
            <select
              value={settings?.defaultExportFormat ?? "csv"}
              onChange={(e) =>
                setLocalSettings((prev) =>
                  prev
                    ? {
                        ...prev,
                        defaultExportFormat: e.target
                          .value as AppSettings["defaultExportFormat"],
                      }
                    : prev,
                )
              }
              className="w-full rounded-md border px-3 py-2 text-sm"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-background)",
                color: "var(--color-foreground)",
              }}
            >
              <option value="csv">CSV</option>
              <option value="json">JSON (NDJSON)</option>
              <option value="html">HTML Report</option>
              <option value="xlsx">Excel (XLSX)</option>
            </select>
          </div>
        </div>
      </section>

      {/* AI Providers */}
      <AiProviderSection />

      {/* Plugins (Phase 8 placeholder) */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">Plugins</h2>
        <div
          className="rounded-lg border p-6 text-center"
          style={{ borderColor: "var(--color-border)" }}
        >
          <p className="text-sm" style={{ color: "var(--color-muted-foreground)" }}>
            Plugin management will be available in a future release.
          </p>
        </div>
      </section>

      {/* About */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">About</h2>
        <div
          className="space-y-2 rounded-lg border p-4"
          style={{ borderColor: "var(--color-border)" }}
        >
          <p className="text-sm">
            <span className="font-medium">OxideSEO</span> v0.1.0
          </p>
          <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            Open-source SEO crawler and audit platform. MIT / Apache 2.0 dual license.
          </p>
        </div>
      </section>

      {/* Save */}
      <div className="flex items-center gap-3">
        <button
          onClick={handleSave}
          disabled={isSaving}
          className="rounded-md px-4 py-2 text-sm font-medium disabled:opacity-50"
          style={{
            backgroundColor: "var(--color-primary)",
            color: "var(--color-primary-foreground)",
          }}
        >
          {isSaving ? "Saving..." : "Save Settings"}
        </button>
        {saveMessage && (
          <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            {saveMessage}
          </p>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// AI Provider configuration section
// ---------------------------------------------------------------------------

const PROVIDER_OPTIONS: { value: AiProviderType; label: string }[] = [
  { value: "open_ai", label: "OpenAI" },
  { value: "anthropic", label: "Anthropic" },
  { value: "ollama", label: "Ollama (Local)" },
];

const DEFAULT_MODELS: Record<AiProviderType, string> = {
  open_ai: "gpt-4o",
  anthropic: "claude-sonnet-4-20250514",
  ollama: "llama3",
};

function AiProviderSection() {
  const [config, setConfig] = useState<AiProviderConfig | null>(null);
  const [apiKeyInput, setApiKeyInput] = useState("");
  const [keyStored, setKeyStored] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [message, setMessage] = useState<string | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    getAiConfig()
      .then((cfg) => {
        setConfig(cfg);
        setKeyStored(cfg.isConfigured);
      })
      .catch((err) => setLoadError(String(err)));
  }, []);

  const refreshKeyStatus = async (provider: AiProviderType) => {
    try {
      const has = await hasApiKey(provider);
      setKeyStored(has);
    } catch (err) {
      setKeyStored(false);
      setMessage(`Warning: could not check key status: ${err}`);
    }
  };

  const handleProviderChange = (provider: AiProviderType) => {
    if (!config) return;
    const updated = {
      ...config,
      providerType: provider,
      model: DEFAULT_MODELS[provider],
      ollamaEndpoint: provider === "ollama" ? "http://localhost:11434" : null,
    };
    setConfig(updated);
    setApiKeyInput("");
    refreshKeyStatus(provider);
  };

  const handleSaveConfig = async () => {
    if (!config) return;
    setIsSaving(true);
    setMessage(null);
    try {
      await setAiConfig(config);
      setMessage("AI configuration saved.");
    } catch (err) {
      setMessage(`Error: ${err}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleSaveKey = async () => {
    if (!config || !apiKeyInput.trim()) return;
    setIsSaving(true);
    setMessage(null);
    try {
      await setApiKey(config.providerType, apiKeyInput.trim());
      setApiKeyInput("");
      setKeyStored(true);
      setMessage("API key saved to OS keychain.");
    } catch (err) {
      setMessage(`Error saving key: ${err}`);
    } finally {
      setIsSaving(false);
    }
  };

  const handleDeleteKey = async () => {
    if (!config) return;
    try {
      await deleteApiKey(config.providerType);
      setKeyStored(false);
      setMessage("API key deleted.");
    } catch (err) {
      setMessage(`Error deleting key: ${err}`);
    }
  };

  const handleTestConnection = async () => {
    setIsTesting(true);
    setMessage(null);
    try {
      const result = await testAiConnection();
      setMessage(result);
    } catch (err) {
      setMessage(`Connection failed: ${err}`);
    } finally {
      setIsTesting(false);
    }
  };

  if (loadError) {
    return (
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">AI Providers</h2>
        <div
          className="rounded-lg border p-4"
          style={{ borderColor: "var(--color-border)" }}
        >
          <p className="text-sm" style={{ color: "var(--color-severity-error, red)" }}>
            Failed to load AI configuration: {loadError}
          </p>
        </div>
      </section>
    );
  }

  if (!config) return null;

  const isOllama = config.providerType === "ollama";

  return (
    <section className="space-y-4">
      <h2 className="text-lg font-semibold">AI Providers</h2>
      <div
        className="space-y-4 rounded-lg border p-4"
        style={{ borderColor: "var(--color-border)" }}
      >
        {/* Provider selector */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium">Provider</label>
          <select
            value={config.providerType}
            onChange={(e) => handleProviderChange(e.target.value as AiProviderType)}
            className="w-full rounded-md border px-3 py-2 text-sm"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          >
            {PROVIDER_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {/* API key */}
        {!isOllama && (
          <div className="space-y-1.5">
            <label className="text-sm font-medium">
              API Key{" "}
              <span
                className="text-xs"
                style={{
                  color: keyStored
                    ? "var(--color-success, green)"
                    : "var(--color-muted-foreground)",
                }}
              >
                {keyStored ? "(stored in OS keychain)" : "(not set)"}
              </span>
            </label>
            <div className="flex gap-2">
              <input
                type="password"
                value={apiKeyInput}
                onChange={(e) => setApiKeyInput(e.target.value)}
                placeholder={keyStored ? "Enter new key to replace" : "Enter API key"}
                className="flex-1 rounded-md border px-3 py-2 text-sm"
                style={{
                  borderColor: "var(--color-border)",
                  backgroundColor: "var(--color-background)",
                  color: "var(--color-foreground)",
                }}
              />
              <button
                onClick={handleSaveKey}
                disabled={!apiKeyInput.trim() || isSaving}
                className="rounded-md border px-3 py-2 text-sm font-medium disabled:opacity-50"
                style={{ borderColor: "var(--color-border)" }}
              >
                Save Key
              </button>
              {keyStored && (
                <button
                  onClick={handleDeleteKey}
                  className="rounded-md border px-3 py-2 text-sm"
                  style={{
                    borderColor: "var(--color-border)",
                    color: "var(--color-destructive, red)",
                  }}
                >
                  Delete
                </button>
              )}
            </div>
          </div>
        )}

        {/* Model */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium">Model</label>
          <input
            type="text"
            value={config.model}
            onChange={(e) => setConfig({ ...config, model: e.target.value })}
            className="w-full rounded-md border px-3 py-2 text-sm"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          />
        </div>

        {/* Ollama endpoint */}
        {isOllama && (
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Ollama Endpoint</label>
            <input
              type="text"
              value={config.ollamaEndpoint ?? "http://localhost:11434"}
              onChange={(e) => setConfig({ ...config, ollamaEndpoint: e.target.value })}
              className="w-full rounded-md border px-3 py-2 text-sm"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-background)",
                color: "var(--color-foreground)",
              }}
            />
          </div>
        )}

        {/* Token budget */}
        <div className="space-y-1.5">
          <label className="text-sm font-medium">Token Budget Per Crawl</label>
          <input
            type="number"
            value={config.maxTokensPerCrawl}
            onChange={(e) =>
              setConfig({
                ...config,
                maxTokensPerCrawl: parseInt(e.target.value) || 100000,
              })
            }
            min={1000}
            step={10000}
            className="w-full rounded-md border px-3 py-2 text-sm"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          />
          <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            Maximum tokens to spend per analysis session. Ollama runs locally at no cost.
          </p>
        </div>

        {/* Actions */}
        <div className="flex items-center gap-2 pt-2">
          <button
            onClick={handleSaveConfig}
            disabled={isSaving}
            className="rounded-md px-4 py-2 text-sm font-medium disabled:opacity-50"
            style={{
              backgroundColor: "var(--color-primary)",
              color: "var(--color-primary-foreground)",
            }}
          >
            {isSaving ? "Saving..." : "Save AI Config"}
          </button>
          <button
            onClick={handleTestConnection}
            disabled={isTesting || (!keyStored && !isOllama)}
            className="rounded-md border px-4 py-2 text-sm font-medium disabled:opacity-50"
            style={{ borderColor: "var(--color-border)" }}
          >
            {isTesting ? "Testing..." : "Test Connection"}
          </button>
        </div>

        {message && (
          <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            {message}
          </p>
        )}
      </div>
    </section>
  );
}
