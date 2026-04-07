/**
 * AI Insights tab: crawl summary, batch analysis controls, and cost tracking.
 */

import { useCallback, useEffect, useState } from "react";
import {
  getAiConfig,
  getAiUsage,
  getCrawlAiSummary,
  generateCrawlSummary,
  batchAnalyzePages,
  estimateBatchCost,
} from "@/lib/commands";
import { useAiProgress } from "@/hooks/useAiProgress";
import type {
  AiProviderConfig,
  AiUsageRow,
  AiCrawlSummaryRow,
  BatchProgress,
  BatchAnalysisResult,
  BatchCostEstimate,
  CrawlSummaryResult,
} from "@/types";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Loader2 } from "lucide-react";

interface AiInsightsTabProps {
  crawlId: string;
}

export function AiInsightsTab({ crawlId }: AiInsightsTabProps) {
  const [config, setConfig] = useState<AiProviderConfig | null>(null);
  const [summary, setSummary] = useState<AiCrawlSummaryRow | null>(null);
  const [usage, setUsage] = useState<AiUsageRow[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    setIsLoading(true);
    setLoadError(null);
    Promise.all([getAiConfig(), getCrawlAiSummary(crawlId), getAiUsage(crawlId)])
      .then(([cfg, sum, usg]) => {
        setConfig(cfg);
        setSummary(sum);
        setUsage(usg);
      })
      .catch((err) => setLoadError(String(err)))
      .finally(() => setIsLoading(false));
  }, [crawlId]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2
          className="h-6 w-6 animate-spin"
          style={{ color: "var(--color-muted-foreground)" }}
        />
      </div>
    );
  }

  if (loadError) {
    return (
      <div className="flex flex-col items-center justify-center gap-2 py-12">
        <p className="text-sm" style={{ color: "var(--color-severity-error)" }}>
          Failed to load AI configuration
        </p>
        <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
          {loadError}
        </p>
      </div>
    );
  }

  if (!config?.isConfigured) {
    return (
      <div className="flex flex-col items-center justify-center gap-2 py-12">
        <p className="text-sm" style={{ color: "var(--color-muted-foreground)" }}>
          No AI provider configured.
        </p>
        <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
          Go to Settings to configure an AI provider (OpenAI, Anthropic, or Ollama).
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6">
      <CrawlSummarySection
        crawlId={crawlId}
        summary={summary}
        onSummaryGenerated={setSummary}
      />

      <Separator />

      <BatchAnalysisSection
        crawlId={crawlId}
        onComplete={() => {
          getAiUsage(crawlId).then(setUsage).catch(console.error);
        }}
      />

      <Separator />

      <CostTrackingSection usage={usage} config={config} />
    </div>
  );
}

// ---------------------------------------------------------------------------
// Crawl Summary
// ---------------------------------------------------------------------------

function CrawlSummarySection({
  crawlId,
  summary,
  onSummaryGenerated,
}: {
  crawlId: string;
  summary: AiCrawlSummaryRow | null;
  onSummaryGenerated: (s: AiCrawlSummaryRow) => void;
}) {
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGenerate = async (force?: boolean) => {
    setIsGenerating(true);
    setError(null);
    try {
      const result = await generateCrawlSummary(crawlId, force);
      onSummaryGenerated(result);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsGenerating(false);
    }
  };

  const parseSummary = (json: string): Partial<CrawlSummaryResult> | null => {
    try {
      return JSON.parse(json) as Partial<CrawlSummaryResult>;
    } catch {
      return null;
    }
  };

  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">Crawl Summary</h3>

      {!summary && (
        <div>
          <button
            onClick={() => handleGenerate()}
            disabled={isGenerating}
            className="rounded-md px-4 py-2 text-sm font-medium disabled:opacity-50"
            style={{
              backgroundColor: "var(--color-primary)",
              color: "var(--color-primary-foreground)",
            }}
          >
            {isGenerating ? (
              <span className="flex items-center gap-2">
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
                Generating Summary...
              </span>
            ) : (
              "Generate AI Summary"
            )}
          </button>
          {error && (
            <p className="mt-2 text-xs" style={{ color: "var(--color-severity-error)" }}>
              {error}
            </p>
          )}
        </div>
      )}

      {summary &&
        (() => {
          const parsed = parseSummary(summary.summaryJson);
          if (!parsed) {
            return (
              <pre
                className="max-h-60 overflow-auto rounded-md border p-3 text-xs whitespace-pre-wrap"
                style={{ borderColor: "var(--color-border)" }}
              >
                {summary.summaryJson}
              </pre>
            );
          }
          return (
            <div className="space-y-3">
              {parsed.overallHealth && (
                <Badge
                  variant={
                    parsed.overallHealth === "good"
                      ? "default"
                      : parsed.overallHealth === "fair"
                        ? "secondary"
                        : "destructive"
                  }
                >
                  Health: {parsed.overallHealth}
                </Badge>
              )}
              {parsed.summary && (
                <p className="text-sm leading-relaxed">{parsed.summary}</p>
              )}
              {parsed.topActions && parsed.topActions.length > 0 && (
                <div>
                  <p className="mb-1 text-xs font-medium">Top Actions:</p>
                  <ol className="list-inside list-decimal space-y-1">
                    {parsed.topActions.map((action, i) => (
                      <li key={i} className="text-sm">
                        {action}
                      </li>
                    ))}
                  </ol>
                </div>
              )}
              <div className="flex items-center gap-3">
                <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                  Generated by {summary.provider} ({summary.model}) — $
                  {summary.costUsd.toFixed(4)}
                </p>
                <button
                  onClick={() => handleGenerate(true)}
                  disabled={isGenerating}
                  className="text-xs font-medium disabled:opacity-50"
                  style={{ color: "var(--color-primary)" }}
                >
                  {isGenerating ? (
                    <span className="flex items-center gap-1">
                      <Loader2 className="h-3 w-3 animate-spin" />
                      Regenerating...
                    </span>
                  ) : (
                    "Regenerate"
                  )}
                </button>
              </div>
              {error && (
                <p className="text-xs" style={{ color: "var(--color-severity-error)" }}>
                  {error}
                </p>
              )}
            </div>
          );
        })()}
    </section>
  );
}

// ---------------------------------------------------------------------------
// Batch Analysis
// ---------------------------------------------------------------------------

function BatchAnalysisSection({
  crawlId,
  onComplete,
}: {
  crawlId: string;
  onComplete: () => void;
}) {
  const [isRunning, setIsRunning] = useState(false);
  const [progress, setProgress] = useState<BatchProgress | null>(null);
  const [result, setResult] = useState<BatchAnalysisResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [maxPages, setMaxPages] = useState(50);
  const [estimate, setEstimate] = useState<BatchCostEstimate | null>(null);
  const [pendingFilter, setPendingFilter] = useState<boolean | null>(null);
  const [isEstimating, setIsEstimating] = useState(false);

  useAiProgress(useCallback((p: BatchProgress) => setProgress(p), []));

  const analysisTypes: ("content_score" | "meta_desc" | "title_tag")[] = [
    "content_score",
    "meta_desc",
    "title_tag",
  ];

  const handleEstimate = async (onlyWithIssues: boolean) => {
    setIsEstimating(true);
    setEstimate(null);
    setError(null);
    setPendingFilter(onlyWithIssues);
    try {
      const est = await estimateBatchCost(
        crawlId,
        { onlyWithIssues, onlyMissingMeta: false, maxPages },
        analysisTypes,
      );
      setEstimate(est);
    } catch (err) {
      setError(String(err));
      setPendingFilter(null);
    } finally {
      setIsEstimating(false);
    }
  };

  const handleConfirm = async () => {
    if (pendingFilter === null) return;
    setIsRunning(true);
    setProgress(null);
    setResult(null);
    setError(null);
    setEstimate(null);
    try {
      const res = await batchAnalyzePages(
        crawlId,
        { onlyWithIssues: pendingFilter, onlyMissingMeta: false, maxPages },
        analysisTypes,
      );
      setResult(res);
      onComplete();
    } catch (err) {
      setError(String(err));
    } finally {
      setIsRunning(false);
      setProgress(null);
      setPendingFilter(null);
    }
  };

  const handleCancel = () => {
    setEstimate(null);
    setPendingFilter(null);
  };

  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">Batch Analysis</h3>

      <div className="space-y-3">
        <div className="flex items-center gap-3">
          <label className="text-sm">Max pages:</label>
          <input
            type="number"
            value={maxPages}
            onChange={(e) => setMaxPages(parseInt(e.target.value) || 50)}
            min={1}
            max={1000}
            className="w-20 rounded-md border px-2 py-1 text-sm"
            style={{
              borderColor: "var(--color-border)",
              backgroundColor: "var(--color-background)",
              color: "var(--color-foreground)",
            }}
            disabled={isRunning || estimate !== null}
          />
        </div>

        {!estimate && (
          <div className="flex gap-2">
            <button
              onClick={() => handleEstimate(false)}
              disabled={isRunning || isEstimating}
              className="rounded-md border px-4 py-2 text-sm font-medium disabled:opacity-50"
              style={{ borderColor: "var(--color-border)" }}
            >
              {isEstimating && pendingFilter === false
                ? "Estimating..."
                : "Analyze All Pages"}
            </button>
            <button
              onClick={() => handleEstimate(true)}
              disabled={isRunning || isEstimating}
              className="rounded-md border px-4 py-2 text-sm font-medium disabled:opacity-50"
              style={{ borderColor: "var(--color-border)" }}
            >
              {isEstimating && pendingFilter === true
                ? "Estimating..."
                : "Analyze Pages with Issues"}
            </button>
          </div>
        )}

        {/* Cost estimate confirmation */}
        {estimate && !isRunning && (
          <div
            className="rounded-md border p-3"
            style={{ borderColor: "var(--color-border)" }}
          >
            <p className="text-sm font-medium">Cost Estimate</p>
            <p
              className="mt-1 text-xs"
              style={{ color: "var(--color-muted-foreground)" }}
            >
              {estimate.estimatedCostUsd > 0
                ? `~${estimate.eligiblePages} pages, ~${(estimate.estimatedInputTokens + estimate.estimatedOutputTokens).toLocaleString()} tokens, ~$${estimate.estimatedCostUsd.toFixed(4)}`
                : `~${estimate.eligiblePages} pages (local inference, no cost)`}
            </p>
            <div className="mt-2 flex gap-2">
              <button
                onClick={handleConfirm}
                className="rounded-md px-4 py-1.5 text-sm font-medium"
                style={{
                  backgroundColor: "var(--color-primary)",
                  color: "var(--color-primary-foreground)",
                }}
              >
                Start Analysis
              </button>
              <button
                onClick={handleCancel}
                className="rounded-md border px-4 py-1.5 text-sm"
                style={{ borderColor: "var(--color-border)" }}
              >
                Cancel
              </button>
            </div>
          </div>
        )}

        {/* Progress bar */}
        {isRunning && progress && (
          <div className="space-y-1">
            <div
              className="h-2 overflow-hidden rounded-full"
              style={{ backgroundColor: "var(--color-muted)" }}
            >
              <div
                className="h-full rounded-full transition-all"
                style={{
                  backgroundColor: "var(--color-primary)",
                  width: `${progress.total > 0 ? (progress.completed / progress.total) * 100 : 0}%`,
                }}
              />
            </div>
            <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
              {progress.completed}/{progress.total} pages —{" "}
              {progress.tokensUsed.toLocaleString()} tokens used
            </p>
          </div>
        )}

        {isRunning && !progress && (
          <div className="flex items-center gap-2">
            <Loader2
              className="h-4 w-4 animate-spin"
              style={{ color: "var(--color-muted-foreground)" }}
            />
            <span className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
              Starting analysis...
            </span>
          </div>
        )}

        {/* Results */}
        {result && (
          <div
            className="rounded-md border p-3"
            style={{ borderColor: "var(--color-border)" }}
          >
            <p className="text-sm font-medium">Analysis Complete</p>
            <div className="mt-1 grid grid-cols-3 gap-4 text-center">
              <div>
                <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                  Pages
                </p>
                <p className="text-sm font-semibold tabular-nums">
                  {result.pagesAnalyzed}
                </p>
              </div>
              <div>
                <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                  Tokens
                </p>
                <p className="text-sm font-semibold tabular-nums">
                  {(result.totalInputTokens + result.totalOutputTokens).toLocaleString()}
                </p>
              </div>
              <div>
                <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                  Cost
                </p>
                <p className="text-sm font-semibold tabular-nums">
                  ${result.totalCostUsd.toFixed(4)}
                </p>
              </div>
            </div>
            {result.budgetExhausted && (
              <p
                className="mt-1 text-xs"
                style={{ color: "var(--color-severity-warning)" }}
              >
                Token budget exhausted — some pages were not analyzed.
              </p>
            )}
            {result.errors > 0 && (
              <p
                className="mt-1 text-xs"
                style={{ color: "var(--color-severity-error)" }}
              >
                {result.errors} page(s) failed during analysis
              </p>
            )}
          </div>
        )}

        {error && (
          <p className="text-xs" style={{ color: "var(--color-severity-error)" }}>
            {error}
          </p>
        )}
      </div>
    </section>
  );
}

// ---------------------------------------------------------------------------
// Cost Tracking
// ---------------------------------------------------------------------------

function CostTrackingSection({
  usage,
  config,
}: {
  usage: AiUsageRow[];
  config: AiProviderConfig;
}) {
  const totalCost = usage.reduce((sum, u) => sum + u.totalCostUsd, 0);
  const totalTokens = usage.reduce(
    (sum, u) => sum + u.totalInputTokens + u.totalOutputTokens,
    0,
  );
  const totalRequests = usage.reduce((sum, u) => sum + u.requestCount, 0);

  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">Cost Tracking</h3>

      {usage.length === 0 ? (
        <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
          No AI analysis has been run for this crawl yet.
        </p>
      ) : (
        <div className="space-y-3">
          {/* Summary row */}
          <div className="grid grid-cols-4 gap-4 text-center">
            <div>
              <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                Total Cost
              </p>
              <p className="text-sm font-semibold tabular-nums">
                ${totalCost.toFixed(4)}
              </p>
            </div>
            <div>
              <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                Total Tokens
              </p>
              <p className="text-sm font-semibold tabular-nums">
                {totalTokens.toLocaleString()}
              </p>
            </div>
            <div>
              <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                Requests
              </p>
              <p className="text-sm font-semibold tabular-nums">{totalRequests}</p>
            </div>
            <div>
              <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
                Budget Used
              </p>
              <p className="text-sm font-semibold tabular-nums">
                {config.maxTokensPerCrawl > 0
                  ? `${((totalTokens / config.maxTokensPerCrawl) * 100).toFixed(1)}%`
                  : "N/A"}
              </p>
            </div>
          </div>

          {/* Per-model breakdown */}
          <div
            className="overflow-hidden rounded-md border"
            style={{ borderColor: "var(--color-border)" }}
          >
            <table className="w-full text-sm">
              <thead>
                <tr style={{ backgroundColor: "var(--color-muted)" }}>
                  <th className="px-3 py-1.5 text-left text-xs font-medium">Provider</th>
                  <th className="px-3 py-1.5 text-left text-xs font-medium">Model</th>
                  <th className="px-3 py-1.5 text-right text-xs font-medium">Requests</th>
                  <th className="px-3 py-1.5 text-right text-xs font-medium">
                    Input Tokens
                  </th>
                  <th className="px-3 py-1.5 text-right text-xs font-medium">
                    Output Tokens
                  </th>
                  <th className="px-3 py-1.5 text-right text-xs font-medium">Cost</th>
                </tr>
              </thead>
              <tbody>
                {usage.map((u) => (
                  <tr
                    key={u.id}
                    className="border-t"
                    style={{ borderColor: "var(--color-border)" }}
                  >
                    <td className="px-3 py-1.5 text-xs">{u.provider}</td>
                    <td className="px-3 py-1.5 font-mono text-xs">{u.model}</td>
                    <td className="px-3 py-1.5 text-right text-xs tabular-nums">
                      {u.requestCount}
                    </td>
                    <td className="px-3 py-1.5 text-right text-xs tabular-nums">
                      {u.totalInputTokens.toLocaleString()}
                    </td>
                    <td className="px-3 py-1.5 text-right text-xs tabular-nums">
                      {u.totalOutputTokens.toLocaleString()}
                    </td>
                    <td className="px-3 py-1.5 text-right text-xs tabular-nums">
                      ${u.totalCostUsd.toFixed(4)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </section>
  );
}
