/**
 * Crawl configuration form with Zod validation and resource allocation controls.
 */

import { useState } from "react";
import { startCrawl } from "@/lib/commands";
import { crawlConfigSchema, defaultCrawlConfig, type CrawlConfigFormValues } from "@/lib/validation";
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
              borderColor: errors.startUrl ? "var(--color-severity-error)" : "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
          />
        </FieldGroup>

        {/* Crawl limits */}
        <div className="grid grid-cols-2 gap-4">
          <FieldGroup label="Max Depth" error={errors.maxDepth}>
            <NumberInput value={formData.maxDepth} onChange={(v) => updateField("maxDepth", v)} min={1} max={100} />
          </FieldGroup>
          <FieldGroup label="Max Pages (0 = unlimited)" error={errors.maxPages}>
            <NumberInput value={formData.maxPages} onChange={(v) => updateField("maxPages", v)} min={0} max={10000000} />
          </FieldGroup>
        </div>

        {/* Concurrency */}
        <div className="grid grid-cols-3 gap-4">
          <FieldGroup label="Concurrent Requests" error={errors.maxConcurrency}>
            <NumberInput value={formData.maxConcurrency} onChange={(v) => updateField("maxConcurrency", v)} min={1} max={200} />
          </FieldGroup>
          <FieldGroup label="Fetch Workers" error={errors.fetchWorkers}>
            <NumberInput value={formData.fetchWorkers} onChange={(v) => updateField("fetchWorkers", v)} min={1} max={32} />
          </FieldGroup>
          <FieldGroup label="Parse Threads" error={errors.parseThreads}>
            <NumberInput value={formData.parseThreads} onChange={(v) => updateField("parseThreads", v)} min={1} max={64} />
          </FieldGroup>
        </div>

        {/* Politeness */}
        <div className="grid grid-cols-2 gap-4">
          <FieldGroup label="Crawl Delay (ms)" error={errors.crawlDelayMs}>
            <NumberInput value={formData.crawlDelayMs} onChange={(v) => updateField("crawlDelayMs", v)} min={0} max={10000} />
          </FieldGroup>
          <FieldGroup label="Request Timeout (s)" error={errors.requestTimeoutSecs}>
            <NumberInput value={formData.requestTimeoutSecs} onChange={(v) => updateField("requestTimeoutSecs", v)} min={5} max={120} />
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
