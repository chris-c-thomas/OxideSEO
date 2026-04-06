/**
 * Crawl configuration form with Zod validation and resource allocation controls.
 */

import { useState } from "react";
import { startCrawl } from "@/lib/commands";
import {
  crawlConfigSchema,
  defaultCrawlConfig,
  type CrawlConfigFormValues,
} from "@/lib/validation";
import { useCrawlStore } from "@/stores/crawlStore";
import type { CrawlConfig as CrawlConfigType } from "@/types";

interface CrawlConfigProps {
  onCrawlStarted: (crawlId: string) => void;
}

export function CrawlConfig({ onCrawlStarted }: CrawlConfigProps) {
  const [formData, setFormData] = useState<CrawlConfigFormValues>(defaultCrawlConfig);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const setCrawlStarted = useCrawlStore((s) => s.setCrawlStarted);

  const updateField = <K extends keyof CrawlConfigFormValues>(
    key: K,
    value: CrawlConfigFormValues[K],
  ) => {
    setFormData((prev) => ({ ...prev, [key]: value }));
    // Clear field error on change.
    setErrors((prev) => {
      const next = { ...prev };
      delete next[key];
      return next;
    });
  };

  const handleSubmit = async () => {
    // Validate.
    const result = crawlConfigSchema.safeParse(formData);
    if (!result.success) {
      const fieldErrors: Record<string, string> = {};
      for (const issue of result.error.issues) {
        const key = issue.path[0]?.toString() ?? "unknown";
        fieldErrors[key] = issue.message;
      }
      setErrors(fieldErrors);
      return;
    }

    setIsSubmitting(true);
    try {
      const config = result.data as CrawlConfigType;
      const crawlId = await startCrawl(config);
      setCrawlStarted(crawlId, config);
      onCrawlStarted(crawlId);
    } catch (err) {
      setErrors({ startUrl: String(err) });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="mx-auto max-w-3xl space-y-8 p-8">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Configure Crawl</h1>
        <p className="mt-1 text-sm" style={{ color: "var(--color-muted-foreground)" }}>
          Set the target URL and tune crawler parameters.
        </p>
      </div>

      <div className="space-y-6">
        {/* Start URL */}
        <FieldGroup label="Start URL" error={errors.startUrl}>
          <input
            type="url"
            value={formData.startUrl}
            onChange={(e) => updateField("startUrl", e.target.value)}
            placeholder="https://example.com"
            className="w-full rounded-md border px-3 py-2 text-sm"
            style={{
              borderColor: errors.startUrl
                ? "var(--color-severity-error)"
                : "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          />
        </FieldGroup>

        {/* Crawl limits */}
        <div className="grid grid-cols-2 gap-4">
          <FieldGroup label="Max Depth" error={errors.maxDepth}>
            <NumberInput
              value={formData.maxDepth}
              onChange={(v) => updateField("maxDepth", v)}
              min={1}
              max={100}
            />
          </FieldGroup>
          <FieldGroup label="Max Pages (0 = unlimited)" error={errors.maxPages}>
            <NumberInput
              value={formData.maxPages}
              onChange={(v) => updateField("maxPages", v)}
              min={0}
              max={10000000}
            />
          </FieldGroup>
        </div>

        {/* Concurrency */}
        <div className="grid grid-cols-3 gap-4">
          <FieldGroup label="Concurrent Requests" error={errors.maxConcurrency}>
            <NumberInput
              value={formData.maxConcurrency}
              onChange={(v) => updateField("maxConcurrency", v)}
              min={1}
              max={200}
            />
          </FieldGroup>
          <FieldGroup label="Fetch Workers" error={errors.fetchWorkers}>
            <NumberInput
              value={formData.fetchWorkers}
              onChange={(v) => updateField("fetchWorkers", v)}
              min={1}
              max={32}
            />
          </FieldGroup>
          <FieldGroup label="Parse Threads" error={errors.parseThreads}>
            <NumberInput
              value={formData.parseThreads}
              onChange={(v) => updateField("parseThreads", v)}
              min={1}
              max={64}
            />
          </FieldGroup>
        </div>

        {/* Politeness */}
        <div className="grid grid-cols-2 gap-4">
          <FieldGroup label="Crawl Delay (ms)" error={errors.crawlDelayMs}>
            <NumberInput
              value={formData.crawlDelayMs}
              onChange={(v) => updateField("crawlDelayMs", v)}
              min={0}
              max={10000}
            />
          </FieldGroup>
          <FieldGroup label="Request Timeout (s)" error={errors.requestTimeoutSecs}>
            <NumberInput
              value={formData.requestTimeoutSecs}
              onChange={(v) => updateField("requestTimeoutSecs", v)}
              min={5}
              max={120}
            />
          </FieldGroup>
        </div>

        {/* Toggles */}
        <div className="flex items-center gap-6">
          <label className="flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              checked={formData.respectRobotsTxt}
              onChange={(e) => updateField("respectRobotsTxt", e.target.checked)}
              className="rounded"
            />
            Respect robots.txt
          </label>
        </div>

        {/* Advanced Settings */}
        <AdvancedSection title="Custom Headers">
          <p className="mb-2 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            Inject custom HTTP headers into every crawl request.
          </p>
          <KeyValueEditor
            pairs={formData.customHeaders}
            onChange={(v) => updateField("customHeaders", v)}
            keyPlaceholder="Header name"
            valuePlaceholder="Header value"
          />
        </AdvancedSection>

        <AdvancedSection title="Cookies">
          <p className="mb-2 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            Pre-seed cookies for authenticated crawling. One per row.
          </p>
          <KeyValueEditor
            pairs={formData.cookies}
            onChange={(v) => updateField("cookies", v)}
            keyPlaceholder="Cookie name"
            valuePlaceholder="Cookie value"
          />
        </AdvancedSection>

        <AdvancedSection title="URL Include / Exclude Patterns">
          <p className="mb-2 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            Regex patterns to control which URLs are crawled.
          </p>
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label className="text-xs font-medium">
                Include (must match at least one)
              </label>
              <PatternListEditor
                patterns={formData.includePatterns}
                onChange={(v) => updateField("includePatterns", v)}
                placeholder="/blog/.*"
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-xs font-medium">Exclude (must not match any)</label>
              <PatternListEditor
                patterns={formData.excludePatterns}
                onChange={(v) => updateField("excludePatterns", v)}
                placeholder="/admin/.*"
              />
            </div>
          </div>
        </AdvancedSection>

        <AdvancedSection title="URL Rewrite Rules">
          <p className="mb-2 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            Regex find/replace applied to discovered URLs before crawling.
          </p>
          <KeyValueEditor
            pairs={formData.urlRewriteRules}
            onChange={(v) => updateField("urlRewriteRules", v)}
            keyPlaceholder="Pattern (regex)"
            valuePlaceholder="Replacement"
          />
        </AdvancedSection>

        <AdvancedSection title="Sitemap & External Links">
          <div className="flex flex-col gap-3">
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={formData.enableSitemapDiscovery}
                onChange={(e) => updateField("enableSitemapDiscovery", e.target.checked)}
                className="rounded"
              />
              Enable sitemap auto-discovery
            </label>
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={formData.enableExternalLinkCheck}
                onChange={(e) => updateField("enableExternalLinkCheck", e.target.checked)}
                className="rounded"
              />
              Check external links (HEAD requests)
            </label>
            {formData.enableExternalLinkCheck && (
              <FieldGroup
                label="External Link Concurrency"
                error={errors.externalLinkConcurrency}
              >
                <NumberInput
                  value={formData.externalLinkConcurrency}
                  onChange={(v) => updateField("externalLinkConcurrency", v)}
                  min={1}
                  max={50}
                />
              </FieldGroup>
            )}
          </div>
        </AdvancedSection>

        <AdvancedSection title="JavaScript Rendering">
          <div className="flex flex-col gap-3">
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={formData.enableJsRendering}
                onChange={(e) => updateField("enableJsRendering", e.target.checked)}
                className="rounded"
              />
              Enable JS rendering for SPA detection
            </label>
            {formData.enableJsRendering && (
              <>
                <FieldGroup
                  label="Max Concurrent Webviews"
                  error={errors.jsRenderMaxConcurrent}
                >
                  <NumberInput
                    value={formData.jsRenderMaxConcurrent}
                    onChange={(v) => updateField("jsRenderMaxConcurrent", v)}
                    min={1}
                    max={8}
                  />
                </FieldGroup>
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-1.5">
                    <label className="text-xs font-medium">Always render patterns</label>
                    <PatternListEditor
                      patterns={formData.jsRenderPatterns}
                      onChange={(v) => updateField("jsRenderPatterns", v)}
                      placeholder="/app/.*"
                    />
                  </div>
                  <div className="space-y-1.5">
                    <label className="text-xs font-medium">Never render patterns</label>
                    <PatternListEditor
                      patterns={formData.jsNeverRenderPatterns}
                      onChange={(v) => updateField("jsNeverRenderPatterns", v)}
                      placeholder="/static/.*"
                    />
                  </div>
                </div>
              </>
            )}
          </div>
        </AdvancedSection>

        <AdvancedSection title="Custom CSS Selectors">
          <p className="mb-2 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            Extract data from pages using CSS selectors. Results stored per page.
          </p>
          <KeyValueEditor
            pairs={formData.customCssSelectors}
            onChange={(v) => updateField("customCssSelectors", v)}
            keyPlaceholder="Label"
            valuePlaceholder="CSS selector"
          />
        </AdvancedSection>

        {/* Submit */}
        <button
          onClick={handleSubmit}
          disabled={isSubmitting}
          className="rounded-md px-6 py-2.5 text-sm font-medium disabled:opacity-50"
          style={{
            backgroundColor: "var(--color-primary)",
            color: "var(--color-primary-foreground)",
          }}
        >
          {isSubmitting ? "Starting..." : "Start Crawl"}
        </button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Local sub-components
// ---------------------------------------------------------------------------

function FieldGroup({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <label className="text-sm font-medium">{label}</label>
      {children}
      {error && (
        <p className="text-xs" style={{ color: "var(--color-severity-error)" }}>
          {error}
        </p>
      )}
    </div>
  );
}

function AdvancedSection({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="rounded-md border" style={{ borderColor: "var(--color-border)" }}>
      <button
        type="button"
        onClick={() => setIsOpen((o) => !o)}
        className="flex w-full items-center justify-between px-4 py-3 text-sm font-medium"
      >
        {title}
        <span className="text-xs">{isOpen ? "\u25B2" : "\u25BC"}</span>
      </button>
      {isOpen && (
        <div
          className="border-t px-4 pt-3 pb-4"
          style={{ borderColor: "var(--color-border)" }}
        >
          {children}
        </div>
      )}
    </div>
  );
}

interface KeyValueEditorProps {
  pairs: [string, string][];
  onChange: (pairs: [string, string][]) => void;
  keyPlaceholder: string;
  valuePlaceholder: string;
}

function KeyValueEditor({
  pairs,
  onChange,
  keyPlaceholder,
  valuePlaceholder,
}: KeyValueEditorProps) {
  const updatePair = (index: number, field: 0 | 1, value: string) => {
    const next = pairs.map((p, i) => {
      if (i !== index) return p;
      const copy: [string, string] = [...p];
      copy[field] = value;
      return copy;
    });
    onChange(next);
  };

  const addPair = () => {
    onChange([...pairs, ["", ""]]);
  };

  const removePair = (index: number) => {
    onChange(pairs.filter((_, i) => i !== index));
  };

  return (
    <div className="space-y-2">
      {pairs.map((pair, i) => (
        <div key={i} className="flex items-center gap-2">
          <input
            type="text"
            value={pair[0]}
            onChange={(e) => updatePair(i, 0, e.target.value)}
            placeholder={keyPlaceholder}
            className="flex-1 rounded-md border px-2 py-1.5 text-sm"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          />
          <input
            type="text"
            value={pair[1]}
            onChange={(e) => updatePair(i, 1, e.target.value)}
            placeholder={valuePlaceholder}
            className="flex-1 rounded-md border px-2 py-1.5 text-sm"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          />
          <button
            type="button"
            onClick={() => removePair(i)}
            className="rounded px-2 py-1 text-xs"
            style={{ color: "var(--color-severity-error)" }}
          >
            Remove
          </button>
        </div>
      ))}
      <button
        type="button"
        onClick={addPair}
        className="text-xs font-medium"
        style={{ color: "var(--color-primary)" }}
      >
        + Add
      </button>
    </div>
  );
}

function PatternListEditor({
  patterns,
  onChange,
  placeholder,
}: {
  patterns: string[];
  onChange: (patterns: string[]) => void;
  placeholder: string;
}) {
  const updatePattern = (index: number, value: string) => {
    const next = patterns.map((p, i) => (i === index ? value : p));
    onChange(next);
  };

  const addPattern = () => {
    onChange([...patterns, ""]);
  };

  const removePattern = (index: number) => {
    onChange(patterns.filter((_, i) => i !== index));
  };

  return (
    <div className="space-y-2">
      {patterns.map((pattern, i) => (
        <div key={i} className="flex items-center gap-2">
          <input
            type="text"
            value={pattern}
            onChange={(e) => updatePattern(i, e.target.value)}
            placeholder={placeholder}
            className="flex-1 rounded-md border px-2 py-1.5 font-mono text-sm"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          />
          <button
            type="button"
            onClick={() => removePattern(i)}
            className="rounded px-2 py-1 text-xs"
            style={{ color: "var(--color-severity-error)" }}
          >
            Remove
          </button>
        </div>
      ))}
      <button
        type="button"
        onClick={addPattern}
        className="text-xs font-medium"
        style={{ color: "var(--color-primary)" }}
      >
        + Add
      </button>
    </div>
  );
}

function NumberInput({
  value,
  onChange,
  min,
  max,
}: {
  value: number;
  onChange: (v: number) => void;
  min: number;
  max: number;
}) {
  return (
    <input
      type="number"
      value={value}
      onChange={(e) => onChange(Number(e.target.value))}
      min={min}
      max={max}
      className="w-full rounded-md border px-3 py-2 text-sm tabular-nums"
      style={{
        borderColor: "var(--color-border)",
        backgroundColor: "var(--color-background)",
        color: "var(--color-foreground)",
      }}
    />
  );
}
