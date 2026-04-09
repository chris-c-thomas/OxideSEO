/**
 * Settings view with left sub-navigation and sectioned forms.
 *
 * Replaces src/components/settings/SettingsView.tsx. Preserves all
 * IPC wiring for settings and AI configuration.
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
  listOllamaModels,
} from "@/lib/commands";
import type { AppSettings, AiProviderConfig, AiProviderType } from "@/types";
import { useTheme, type Theme } from "@/hooks/useTheme";
import { useUiStore, type Density } from "@/stores/uiStore";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Card, CardContent } from "@/components/ui/card";
import { Loader2, Check, AlertCircle } from "lucide-react";

type SettingsSection = "general" | "appearance" | "ai" | "about";

const SECTIONS: { id: SettingsSection; label: string }[] = [
  { id: "general", label: "General" },
  { id: "appearance", label: "Appearance" },
  { id: "ai", label: "AI Providers" },
  { id: "about", label: "About" },
];

export function SettingsView() {
  const [activeSection, setActiveSection] = useState<SettingsSection>("general");
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
      setTimeout(() => setSaveMessage(null), 3000);
    } catch (err) {
      setSaveMessage(`Error: ${err}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="flex h-full">
      {/* Left sub-nav */}
      <div className="border-border-subtle flex w-48 shrink-0 flex-col gap-1 border-r p-3">
        <h2 className="text-fg-muted mb-2 px-2 text-xs font-medium">Settings</h2>
        {SECTIONS.map((section) => (
          <button
            key={section.id}
            onClick={() => setActiveSection(section.id)}
            className={cn(
              "rounded-[var(--radius-sm)] px-2 py-1.5 text-left text-sm transition-colors",
              activeSection === section.id
                ? "bg-bg-active text-fg-default font-medium"
                : "text-fg-muted hover:bg-bg-hover hover:text-fg-default",
            )}
          >
            {section.label}
          </button>
        ))}
      </div>

      {/* Right content */}
      <div className="flex-1 overflow-auto p-6">
        <div className="mx-auto max-w-2xl">
          {activeSection === "general" && (
            <GeneralSection settings={settings} onSettingsChange={setLocalSettings} />
          )}
          {activeSection === "appearance" && <AppearanceSection />}
          {activeSection === "ai" && <AiProviderSection />}
          {activeSection === "about" && <AboutSection />}

          {/* Save button (for General section) */}
          {activeSection === "general" && (
            <div className="mt-6 flex items-center gap-3">
              <Button size="sm" onClick={handleSave} disabled={isSaving}>
                {isSaving && <Loader2 className="size-3.5 animate-spin" />}
                {isSaving ? "Saving..." : "Save Settings"}
              </Button>
              {saveMessage && (
                <span className="text-fg-muted flex items-center gap-1 text-xs">
                  <Check className="text-success size-3" />
                  {saveMessage}
                </span>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// General
// ---------------------------------------------------------------------------

function GeneralSection({
  settings,
  onSettingsChange,
}: {
  settings: AppSettings | null;
  onSettingsChange: (s: AppSettings) => void;
}) {
  if (!settings) return null;

  return (
    <div>
      <h3 className="text-fg-default mb-4 text-base font-semibold">General</h3>
      <Card>
        <CardContent className="flex flex-col gap-4 pt-6">
          <div className="flex flex-col gap-1.5">
            <Label className="text-sm">Default Export Format</Label>
            <Select
              value={settings.defaultExportFormat}
              onValueChange={(v) =>
                onSettingsChange({
                  ...settings,
                  defaultExportFormat: v as AppSettings["defaultExportFormat"],
                })
              }
            >
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="csv">CSV</SelectItem>
                <SelectItem value="json">JSON (NDJSON)</SelectItem>
                <SelectItem value="html">HTML Report</SelectItem>
                <SelectItem value="xlsx">Excel (XLSX)</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Appearance
// ---------------------------------------------------------------------------

function AppearanceSection() {
  const { theme, setTheme } = useTheme();
  const density = useUiStore((s) => s.density);
  const setDensity = useUiStore((s) => s.setDensity);

  return (
    <div>
      <h3 className="text-fg-default mb-4 text-base font-semibold">Appearance</h3>
      <Card>
        <CardContent className="flex flex-col gap-4 pt-6">
          <div className="flex flex-col gap-1.5">
            <Label className="text-sm">Theme</Label>
            <Select value={theme} onValueChange={(v) => setTheme(v as Theme)}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="light">Light</SelectItem>
                <SelectItem value="dark">Dark</SelectItem>
                <SelectItem value="system">System</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <Separator />
          <div className="flex flex-col gap-1.5">
            <Label className="text-sm">Table Density</Label>
            <Select value={density} onValueChange={(v) => setDensity(v as Density)}>
              <SelectTrigger className="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="compact">Compact (24px)</SelectItem>
                <SelectItem value="default">Default (28px)</SelectItem>
                <SelectItem value="comfortable">Comfortable (32px)</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}

// ---------------------------------------------------------------------------
// AI Providers (preserves all existing IPC wiring)
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
  const [ollamaModels, setOllamaModels] = useState<string[] | null>(null);
  const [modelsFetchError, setModelsFetchError] = useState<string | null>(null);

  useEffect(() => {
    getAiConfig()
      .then((cfg) => {
        setConfig(cfg);
        setKeyStored(cfg.isConfigured);
        if (cfg.providerType === "ollama") {
          fetchOllamaModels(cfg.ollamaEndpoint ?? "http://localhost:11434");
        }
      })
      .catch((err) => setLoadError(String(err)));
  }, []);

  const fetchOllamaModels = async (endpoint: string) => {
    setModelsFetchError(null);
    try {
      const models = await listOllamaModels(endpoint);
      setOllamaModels(models);
    } catch (err) {
      setOllamaModels(null);
      setModelsFetchError(String(err));
    }
  };

  const refreshKeyStatus = async (provider: AiProviderType) => {
    try {
      const has = await hasApiKey(provider);
      setKeyStored(has);
    } catch {
      setKeyStored(false);
    }
  };

  const handleProviderChange = (provider: AiProviderType) => {
    if (!config) return;
    const endpoint = provider === "ollama" ? "http://localhost:11434" : null;
    setConfig({
      ...config,
      providerType: provider,
      model: DEFAULT_MODELS[provider],
      ollamaEndpoint: endpoint,
    });
    setApiKeyInput("");
    setOllamaModels(null);
    setModelsFetchError(null);
    refreshKeyStatus(provider);
    if (provider === "ollama" && endpoint) fetchOllamaModels(endpoint);
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
      <div>
        <h3 className="text-fg-default mb-4 text-base font-semibold">AI Providers</h3>
        <Alert variant="destructive">
          <AlertCircle className="size-4" />
          <AlertDescription>
            Failed to load AI configuration: {loadError}
          </AlertDescription>
        </Alert>
      </div>
    );
  }

  if (!config) return null;

  const isOllama = config.providerType === "ollama";

  return (
    <div>
      <h3 className="text-fg-default mb-4 text-base font-semibold">AI Providers</h3>
      <Card>
        <CardContent className="flex flex-col gap-4 pt-6">
          {/* Provider */}
          <div className="flex flex-col gap-1.5">
            <Label className="text-sm">Provider</Label>
            <Select
              value={config.providerType}
              onValueChange={(v) => handleProviderChange(v as AiProviderType)}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {PROVIDER_OPTIONS.map((opt) => (
                  <SelectItem key={opt.value} value={opt.value}>
                    {opt.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* API Key */}
          {!isOllama && (
            <div className="flex flex-col gap-1.5">
              <Label className="text-sm">
                API Key{" "}
                <span
                  className={cn("text-xs", keyStored ? "text-success" : "text-fg-muted")}
                >
                  {keyStored ? "(stored in OS keychain)" : "(not set)"}
                </span>
              </Label>
              <div className="flex gap-2">
                <Input
                  type="password"
                  value={apiKeyInput}
                  onChange={(e) => setApiKeyInput(e.target.value)}
                  placeholder={keyStored ? "Enter new key to replace" : "Enter API key"}
                  className="flex-1"
                />
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleSaveKey}
                  disabled={!apiKeyInput.trim() || isSaving}
                >
                  Save Key
                </Button>
                {keyStored && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="text-danger"
                    onClick={handleDeleteKey}
                  >
                    Delete
                  </Button>
                )}
              </div>
            </div>
          )}

          {/* Model */}
          <div className="flex flex-col gap-1.5">
            <Label className="text-sm">Model</Label>
            {isOllama && ollamaModels && ollamaModels.length > 0 ? (
              <div className="flex gap-2">
                <Select
                  value={config.model}
                  onValueChange={(v) => setConfig({ ...config, model: v })}
                >
                  <SelectTrigger className="flex-1">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {ollamaModels.map((m) => (
                      <SelectItem key={m} value={m}>
                        {m}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    fetchOllamaModels(config.ollamaEndpoint ?? "http://localhost:11434")
                  }
                >
                  Refresh
                </Button>
              </div>
            ) : (
              <Input
                value={config.model}
                onChange={(e) => setConfig({ ...config, model: e.target.value })}
              />
            )}
            {isOllama && modelsFetchError && (
              <p className="text-warning text-xs">
                Could not fetch models: {modelsFetchError}
              </p>
            )}
          </div>

          {/* Ollama endpoint */}
          {isOllama && (
            <div className="flex flex-col gap-1.5">
              <Label className="text-sm">Ollama Endpoint</Label>
              <Input
                value={config.ollamaEndpoint ?? "http://localhost:11434"}
                onChange={(e) => {
                  setConfig({ ...config, ollamaEndpoint: e.target.value });
                  setOllamaModels(null);
                  setModelsFetchError(null);
                }}
              />
            </div>
          )}

          {/* Token budget */}
          <div className="flex flex-col gap-1.5">
            <Label className="text-sm">Token Budget Per Crawl</Label>
            <Input
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
              className="tabular-nums"
            />
            <p className="text-fg-muted text-xs">
              Maximum tokens per analysis session. Ollama runs locally at no cost.
            </p>
          </div>

          <Separator />

          {/* Actions */}
          <div className="flex items-center gap-2">
            <Button size="sm" onClick={handleSaveConfig} disabled={isSaving}>
              {isSaving && <Loader2 className="size-3.5 animate-spin" />}
              Save AI Config
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleTestConnection}
              disabled={isTesting || (!keyStored && !isOllama)}
            >
              {isTesting ? "Testing..." : "Test Connection"}
            </Button>
          </div>

          {message && <p className="text-fg-muted text-xs">{message}</p>}
        </CardContent>
      </Card>
    </div>
  );
}

// ---------------------------------------------------------------------------
// About
// ---------------------------------------------------------------------------

function AboutSection() {
  return (
    <div>
      <h3 className="text-fg-default mb-4 text-base font-semibold">About</h3>
      <Card>
        <CardContent className="flex flex-col gap-2 pt-6">
          <p className="text-sm">
            <span className="font-medium">OxideSEO</span> v0.4.0
          </p>
          <p className="text-fg-muted text-xs">
            Open-source SEO crawler and audit platform. MIT / Apache 2.0 dual license.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
