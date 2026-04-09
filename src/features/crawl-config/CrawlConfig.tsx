/**
 * Crawl configuration form with Zod validation and shadcn/ui components.
 *
 * Replaces the original src/components/crawl/CrawlConfig.tsx with proper
 * design tokens and shadcn primitives. All IPC wiring preserved.
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
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Card, CardContent } from "@/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { ChevronDown, Loader2, Plus, X } from "lucide-react";

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
    setErrors((prev) => {
      const next = { ...prev };
      delete next[key];
      return next;
    });
  };

  const handleSubmit = async () => {
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
    <div className="mx-auto max-w-3xl p-6">
      <div className="mb-6">
        <h1 className="text-fg-default text-xl font-semibold tracking-tight">
          Configure Crawl
        </h1>
        <p className="text-fg-muted mt-0.5 text-xs">
          Set the target URL and tune crawler parameters.
        </p>
      </div>

      <Card>
        <CardContent className="flex flex-col gap-6 pt-6">
          {/* Start URL */}
          <Field label="Start URL" error={errors.startUrl}>
            <Input
              type="url"
              value={formData.startUrl}
              onChange={(e) => updateField("startUrl", e.target.value)}
              placeholder="https://example.com"
              aria-invalid={!!errors.startUrl}
            />
          </Field>

          {/* Crawl limits */}
          <div className="grid grid-cols-2 gap-4">
            <Field label="Max Depth" error={errors.maxDepth}>
              <Input
                type="number"
                value={formData.maxDepth}
                onChange={(e) => updateField("maxDepth", Number(e.target.value))}
                min={1}
                max={100}
                className="tabular-nums"
              />
            </Field>
            <Field label="Max Pages (0 = unlimited)" error={errors.maxPages}>
              <Input
                type="number"
                value={formData.maxPages}
                onChange={(e) => updateField("maxPages", Number(e.target.value))}
                min={0}
                max={10000000}
                className="tabular-nums"
              />
            </Field>
          </div>

          {/* Concurrency */}
          <div className="grid grid-cols-3 gap-4">
            <Field label="Concurrent Requests" error={errors.maxConcurrency}>
              <Input
                type="number"
                value={formData.maxConcurrency}
                onChange={(e) => updateField("maxConcurrency", Number(e.target.value))}
                min={1}
                max={200}
                className="tabular-nums"
              />
            </Field>
            <Field label="Fetch Workers" error={errors.fetchWorkers}>
              <Input
                type="number"
                value={formData.fetchWorkers}
                onChange={(e) => updateField("fetchWorkers", Number(e.target.value))}
                min={1}
                max={32}
                className="tabular-nums"
              />
            </Field>
            <Field label="Parse Threads" error={errors.parseThreads}>
              <Input
                type="number"
                value={formData.parseThreads}
                onChange={(e) => updateField("parseThreads", Number(e.target.value))}
                min={1}
                max={64}
                className="tabular-nums"
              />
            </Field>
          </div>

          {/* Politeness */}
          <div className="grid grid-cols-2 gap-4">
            <Field label="Crawl Delay (ms)" error={errors.crawlDelayMs}>
              <Input
                type="number"
                value={formData.crawlDelayMs}
                onChange={(e) => updateField("crawlDelayMs", Number(e.target.value))}
                min={0}
                max={10000}
                className="tabular-nums"
              />
            </Field>
            <Field label="Request Timeout (s)" error={errors.requestTimeoutSecs}>
              <Input
                type="number"
                value={formData.requestTimeoutSecs}
                onChange={(e) =>
                  updateField("requestTimeoutSecs", Number(e.target.value))
                }
                min={5}
                max={120}
                className="tabular-nums"
              />
            </Field>
          </div>

          {/* Toggles */}
          <div className="flex items-center gap-3">
            <Switch
              id="respect-robots"
              checked={formData.respectRobotsTxt}
              onCheckedChange={(v) => updateField("respectRobotsTxt", v)}
            />
            <Label htmlFor="respect-robots" className="text-sm">
              Respect robots.txt
            </Label>
          </div>

          {/* Advanced: Custom Headers */}
          <AdvancedSection title="Custom Headers">
            <p className="text-fg-muted mb-2 text-xs">
              Inject custom HTTP headers into every crawl request.
            </p>
            <KeyValueEditor
              pairs={formData.customHeaders}
              onChange={(v) => updateField("customHeaders", v)}
              keyPlaceholder="Header name"
              valuePlaceholder="Header value"
            />
          </AdvancedSection>

          {/* Advanced: Cookies */}
          <AdvancedSection title="Cookies">
            <p className="text-fg-muted mb-2 text-xs">
              Pre-seed cookies for authenticated crawling.
            </p>
            <KeyValueEditor
              pairs={formData.cookies}
              onChange={(v) => updateField("cookies", v)}
              keyPlaceholder="Cookie name"
              valuePlaceholder="Cookie value"
            />
          </AdvancedSection>

          {/* Advanced: URL Patterns */}
          <AdvancedSection title="URL Include / Exclude Patterns">
            <p className="text-fg-muted mb-2 text-xs">
              Regex patterns to control which URLs are crawled.
            </p>
            <div className="grid grid-cols-2 gap-4">
              <div className="flex flex-col gap-1.5">
                <Label className="text-xs">Include (must match at least one)</Label>
                <PatternListEditor
                  patterns={formData.includePatterns}
                  onChange={(v) => updateField("includePatterns", v)}
                  placeholder="/blog/.*"
                />
              </div>
              <div className="flex flex-col gap-1.5">
                <Label className="text-xs">Exclude (must not match any)</Label>
                <PatternListEditor
                  patterns={formData.excludePatterns}
                  onChange={(v) => updateField("excludePatterns", v)}
                  placeholder="/admin/.*"
                />
              </div>
            </div>
          </AdvancedSection>

          {/* Advanced: URL Rewrite Rules */}
          <AdvancedSection title="URL Rewrite Rules">
            <p className="text-fg-muted mb-2 text-xs">
              Regex find/replace applied to discovered URLs before crawling.
            </p>
            <KeyValueEditor
              pairs={formData.urlRewriteRules}
              onChange={(v) => updateField("urlRewriteRules", v)}
              keyPlaceholder="Pattern (regex)"
              valuePlaceholder="Replacement"
            />
          </AdvancedSection>

          {/* Advanced: Sitemap & External Links */}
          <AdvancedSection title="Sitemap & External Links">
            <div className="flex flex-col gap-3">
              <div className="flex items-center gap-3">
                <Switch
                  id="sitemap-discovery"
                  checked={formData.enableSitemapDiscovery}
                  onCheckedChange={(v) => updateField("enableSitemapDiscovery", v)}
                />
                <Label htmlFor="sitemap-discovery" className="text-sm">
                  Enable sitemap auto-discovery
                </Label>
              </div>
              <div className="flex items-center gap-3">
                <Switch
                  id="external-links"
                  checked={formData.enableExternalLinkCheck}
                  onCheckedChange={(v) => updateField("enableExternalLinkCheck", v)}
                />
                <Label htmlFor="external-links" className="text-sm">
                  Check external links (HEAD requests)
                </Label>
              </div>
              {formData.enableExternalLinkCheck && (
                <Field
                  label="External Link Concurrency"
                  error={errors.externalLinkConcurrency}
                >
                  <Input
                    type="number"
                    value={formData.externalLinkConcurrency}
                    onChange={(e) =>
                      updateField("externalLinkConcurrency", Number(e.target.value))
                    }
                    min={1}
                    max={50}
                    className="tabular-nums"
                  />
                </Field>
              )}
            </div>
          </AdvancedSection>

          {/* Advanced: JS Rendering */}
          <AdvancedSection title="JavaScript Rendering">
            <div className="flex flex-col gap-3">
              <div className="flex items-center gap-3">
                <Switch
                  id="js-rendering"
                  checked={formData.enableJsRendering}
                  onCheckedChange={(v) => updateField("enableJsRendering", v)}
                />
                <Label htmlFor="js-rendering" className="text-sm">
                  Enable JS rendering for SPA detection
                </Label>
              </div>
              {formData.enableJsRendering && (
                <>
                  <Field
                    label="Max Concurrent Webviews"
                    error={errors.jsRenderMaxConcurrent}
                  >
                    <Input
                      type="number"
                      value={formData.jsRenderMaxConcurrent}
                      onChange={(e) =>
                        updateField("jsRenderMaxConcurrent", Number(e.target.value))
                      }
                      min={1}
                      max={8}
                      className="tabular-nums"
                    />
                  </Field>
                  <div className="grid grid-cols-2 gap-4">
                    <div className="flex flex-col gap-1.5">
                      <Label className="text-xs">Always render patterns</Label>
                      <PatternListEditor
                        patterns={formData.jsRenderPatterns}
                        onChange={(v) => updateField("jsRenderPatterns", v)}
                        placeholder="/app/.*"
                      />
                    </div>
                    <div className="flex flex-col gap-1.5">
                      <Label className="text-xs">Never render patterns</Label>
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

          {/* Advanced: Custom CSS Selectors */}
          <AdvancedSection title="Custom CSS Selectors">
            <p className="text-fg-muted mb-2 text-xs">
              Extract data from pages using CSS selectors. Results stored per page.
            </p>
            <KeyValueEditor
              pairs={formData.customCssSelectors}
              onChange={(v) => updateField("customCssSelectors", v)}
              keyPlaceholder="Label"
              valuePlaceholder="CSS selector"
            />
          </AdvancedSection>
        </CardContent>
      </Card>

      {/* Sticky footer */}
      <div className="border-border-subtle bg-bg-app sticky bottom-0 mt-4 flex items-center justify-end gap-3 border-t py-4">
        <Button variant="ghost" size="sm">
          Cancel
        </Button>
        <Button size="sm" onClick={handleSubmit} disabled={isSubmitting}>
          {isSubmitting && <Loader2 className="size-3.5 animate-spin" />}
          {isSubmitting ? "Starting..." : "Start Crawl"}
        </Button>
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Local sub-components
// ---------------------------------------------------------------------------

function Field({
  label,
  error,
  children,
}: {
  label: string;
  error?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex flex-col gap-1.5">
      <Label className="text-sm">{label}</Label>
      {children}
      {error && <p className="text-danger text-xs">{error}</p>}
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
  return (
    <Collapsible>
      <CollapsibleTrigger className="border-border-default text-fg-default hover:bg-bg-hover flex w-full items-center justify-between rounded-[var(--radius-sm)] border px-4 py-2.5 text-sm font-medium transition-colors [&[data-state=open]>svg]:rotate-180">
        {title}
        <ChevronDown
          className="text-fg-subtle size-4 transition-transform duration-200"
          strokeWidth={1.75}
        />
      </CollapsibleTrigger>
      <CollapsibleContent className="border-border-default rounded-b-[var(--radius-sm)] border-x border-b px-4 pt-3 pb-4">
        {children}
      </CollapsibleContent>
    </Collapsible>
  );
}

function KeyValueEditor({
  pairs,
  onChange,
  keyPlaceholder,
  valuePlaceholder,
}: {
  pairs: [string, string][];
  onChange: (pairs: [string, string][]) => void;
  keyPlaceholder: string;
  valuePlaceholder: string;
}) {
  const updatePair = (index: number, field: 0 | 1, value: string) => {
    const next = pairs.map((p, i) => {
      if (i !== index) return p;
      const copy: [string, string] = [...p];
      copy[field] = value;
      return copy;
    });
    onChange(next);
  };

  return (
    <div className="flex flex-col gap-2">
      {pairs.map((pair, i) => (
        <div key={i} className="flex items-center gap-2">
          <Input
            value={pair[0]}
            onChange={(e) => updatePair(i, 0, e.target.value)}
            placeholder={keyPlaceholder}
            className="flex-1 text-xs"
          />
          <Input
            value={pair[1]}
            onChange={(e) => updatePair(i, 1, e.target.value)}
            placeholder={valuePlaceholder}
            className="flex-1 text-xs"
          />
          <Button
            variant="ghost"
            size="sm"
            className="text-danger hover:text-danger size-7 shrink-0 p-0"
            onClick={() => onChange(pairs.filter((_, j) => j !== i))}
          >
            <X className="size-3.5" strokeWidth={1.75} />
          </Button>
        </div>
      ))}
      <Button
        variant="ghost"
        size="sm"
        className="w-fit text-xs"
        onClick={() => onChange([...pairs, ["", ""]])}
      >
        <Plus className="size-3.5" strokeWidth={1.75} />
        Add
      </Button>
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
  return (
    <div className="flex flex-col gap-2">
      {patterns.map((pattern, i) => (
        <div key={i} className="flex items-center gap-2">
          <Input
            value={pattern}
            onChange={(e) => {
              const next = patterns.map((p, j) => (j === i ? e.target.value : p));
              onChange(next);
            }}
            placeholder={placeholder}
            className="flex-1 font-mono text-xs"
          />
          <Button
            variant="ghost"
            size="sm"
            className="text-danger hover:text-danger size-7 shrink-0 p-0"
            onClick={() => onChange(patterns.filter((_, j) => j !== i))}
          >
            <X className="size-3.5" strokeWidth={1.75} />
          </Button>
        </div>
      ))}
      <Button
        variant="ghost"
        size="sm"
        className="w-fit text-xs"
        onClick={() => onChange([...patterns, ""])}
      >
        <Plus className="size-3.5" strokeWidth={1.75} />
        Add
      </Button>
    </div>
  );
}
