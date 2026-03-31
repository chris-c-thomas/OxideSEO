/**
 * Settings view: application preferences, default crawl config, AI providers.
 */

import { useEffect, useState } from "react";
import { getSettings, setSettings } from "@/lib/commands";
import type { AppSettings } from "@/types";

export function SettingsView() {
  const [settings, setLocalSettings] = useState<AppSettings | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [saveMessage, setSaveMessage] = useState<string | null>(null);

  useEffect(() => {
    getSettings()
      .then(setLocalSettings)
      .catch(console.error);
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
        <div className="rounded-lg border p-4 space-y-4" style={{ borderColor: "var(--color-border)" }}>
          <div className="space-y-1.5">
            <label className="text-sm font-medium">Default Export Format</label>
            <select
              value={settings?.defaultExportFormat ?? "csv"}
              onChange={(e) =>
                setLocalSettings((prev) =>
                  prev ? { ...prev, defaultExportFormat: e.target.value as AppSettings["defaultExportFormat"] } : prev,
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

      {/* AI Providers (Phase 7 placeholder) */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">AI Providers</h2>
        <div
          className="rounded-lg border p-6 text-center"
          style={{ borderColor: "var(--color-border)" }}
        >
          <p className="text-sm" style={{ color: "var(--color-muted-foreground)" }}>
            AI provider configuration (OpenAI, Anthropic, Ollama) will be available in a future release.
          </p>
        </div>
      </section>

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
        <div className="rounded-lg border p-4 space-y-2" style={{ borderColor: "var(--color-border)" }}>
          <p className="text-sm"><span className="font-medium">OxideSEO</span> v0.1.0</p>
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
