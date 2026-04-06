/**
 * Page detail slide-out sheet showing SEO metadata, performance stats,
 * issues, and link tables for a single crawled page.
 */

import { useCallback, useEffect, useState } from "react";
import { getPageDetail, getPageAnalyses, analyzePage, getAiConfig } from "@/lib/commands";
import type {
  PageDetail as PageDetailType,
  IssueRow,
  LinkRow,
  Severity,
  AiAnalysisRow,
  ContentScoreResult,
  MetaDescResult,
  TitleTagResult,
} from "@/types";
import { formatBytes } from "@/lib/utils";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Copy, ExternalLink, Loader2 } from "lucide-react";

interface PageDetailProps {
  crawlId: string;
  pageId: number;
  open: boolean;
  onClose: () => void;
}

function severityVariant(s: Severity): "default" | "destructive" | "secondary" {
  switch (s) {
    case "error":
      return "destructive";
    case "warning":
      return "default";
    case "info":
      return "secondary";
  }
}

function statusCodeVariant(
  code: number | null,
): "default" | "secondary" | "destructive" | "outline" {
  if (!code) return "outline";
  if (code >= 400) return "destructive";
  if (code >= 300) return "secondary";
  return "default";
}

export function PageDetail({ crawlId, pageId, open, onClose }: PageDetailProps) {
  const [data, setData] = useState<PageDetailType | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!open) return;
    setData(null);
    setIsLoading(true);
    setError(null);

    getPageDetail(crawlId, pageId)
      .then(setData)
      .catch((err) => setError(String(err)))
      .finally(() => setIsLoading(false));
  }, [crawlId, pageId, open]);

  const copyUrl = () => {
    if (data?.page.url) {
      navigator.clipboard.writeText(data.page.url);
    }
  };

  return (
    <Sheet open={open} onOpenChange={(isOpen) => !isOpen && onClose()}>
      <SheetContent
        side="right"
        className="w-full max-w-2xl overflow-y-auto sm:max-w-2xl"
      >
        <SheetHeader>
          <SheetTitle>Page Detail</SheetTitle>
        </SheetHeader>

        {isLoading && (
          <div className="flex items-center justify-center py-12">
            <Loader2
              className="h-6 w-6 animate-spin"
              style={{ color: "var(--color-muted-foreground)" }}
            />
          </div>
        )}

        {error && (
          <div className="py-8 text-center">
            <p className="text-sm" style={{ color: "var(--color-severity-error)" }}>
              {error}
            </p>
          </div>
        )}

        {data && !isLoading && (
          <div className="space-y-6 pt-4">
            {/* URL Header */}
            <div>
              <div className="flex items-center gap-2">
                <span className="font-mono text-sm break-all">{data.page.url}</span>
                <button
                  onClick={copyUrl}
                  className="shrink-0 opacity-50 hover:opacity-100"
                  title="Copy URL"
                >
                  <Copy className="h-3.5 w-3.5" />
                </button>
                <a
                  href={data.page.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="shrink-0 opacity-50 hover:opacity-100"
                  title="Open in browser"
                >
                  <ExternalLink className="h-3.5 w-3.5" />
                </a>
              </div>
              <div className="mt-2 flex items-center gap-2">
                <Badge variant={statusCodeVariant(data.page.statusCode)}>
                  {data.page.statusCode ?? "N/A"}
                </Badge>
                {data.page.contentType && (
                  <span
                    className="text-xs"
                    style={{ color: "var(--color-muted-foreground)" }}
                  >
                    {data.page.contentType.split(";")[0]}
                  </span>
                )}
              </div>
            </div>

            <Separator />

            {/* SEO Metadata */}
            <section>
              <h3 className="mb-3 text-sm font-semibold">SEO Metadata</h3>
              <MetadataField label="Title" value={data.page.title} charCount />
              <MetadataField
                label="Meta Description"
                value={data.page.metaDesc}
                charCount
              />
              <MetadataField label="H1" value={data.page.h1} />
              <MetadataField label="Canonical" value={data.page.canonical} />
              <MetadataField label="Robots" value={data.page.robotsDirectives} />
            </section>

            <Separator />

            {/* Performance */}
            <section>
              <h3 className="mb-3 text-sm font-semibold">Performance</h3>
              <div className="grid grid-cols-3 gap-4">
                <StatCard
                  label="Response Time"
                  value={
                    data.page.responseTimeMs != null
                      ? `${data.page.responseTimeMs}ms`
                      : "N/A"
                  }
                />
                <StatCard
                  label="Body Size"
                  value={
                    data.page.bodySize != null ? formatBytes(data.page.bodySize) : "N/A"
                  }
                />
                <StatCard label="Depth" value={String(data.page.depth)} />
              </div>
            </section>

            {/* Issues */}
            {data.issues.length > 0 && (
              <>
                <Separator />
                <section>
                  <h3 className="mb-3 text-sm font-semibold">
                    Issues ({data.issues.length})
                  </h3>
                  <div className="space-y-2">
                    {sortIssues(data.issues).map((issue) => (
                      <div
                        key={issue.id}
                        className="rounded-md border p-3"
                        style={{ borderColor: "var(--color-border)" }}
                      >
                        <div className="flex items-center gap-2">
                          <Badge variant={severityVariant(issue.severity as Severity)}>
                            {issue.severity}
                          </Badge>
                          <span
                            className="font-mono text-xs"
                            style={{ color: "var(--color-muted-foreground)" }}
                          >
                            {issue.ruleId}
                          </span>
                        </div>
                        <p className="mt-1 text-sm">{issue.message}</p>
                      </div>
                    ))}
                  </div>
                </section>
              </>
            )}

            {/* Inbound Links */}
            {data.inboundLinks.length > 0 && (
              <>
                <Separator />
                <section>
                  <h3 className="mb-3 text-sm font-semibold">
                    Inbound Links ({data.inboundLinks.length})
                  </h3>
                  <LinkTable links={data.inboundLinks} showSource />
                </section>
              </>
            )}

            {/* Outbound Links */}
            {data.outboundLinks.length > 0 && (
              <>
                <Separator />
                <section>
                  <h3 className="mb-3 text-sm font-semibold">
                    Outbound Links ({data.outboundLinks.length})
                  </h3>
                  <LinkTable links={data.outboundLinks} showSource={false} />
                </section>
              </>
            )}

            {/* AI Analysis */}
            <Separator />
            <AiAnalysisSection crawlId={crawlId} pageId={pageId} />
          </div>
        )}
      </SheetContent>
    </Sheet>
  );
}

function MetadataField({
  label,
  value,
  charCount,
}: {
  label: string;
  value: string | null;
  charCount?: boolean;
}) {
  return (
    <div className="mb-2">
      <span
        className="text-xs font-medium"
        style={{ color: "var(--color-muted-foreground)" }}
      >
        {label}
        {charCount && value && (
          <span className="ml-1 tabular-nums">({value.length} chars)</span>
        )}
      </span>
      <p className="text-sm">
        {value || <span className="italic opacity-40">missing</span>}
      </p>
    </div>
  );
}

function StatCard({ label, value }: { label: string; value: string }) {
  return (
    <div
      className="rounded-md border p-3 text-center"
      style={{ borderColor: "var(--color-border)" }}
    >
      <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
        {label}
      </p>
      <p className="mt-1 text-sm font-semibold tabular-nums">{value}</p>
    </div>
  );
}

function LinkTable({ links, showSource }: { links: LinkRow[]; showSource: boolean }) {
  return (
    <div
      className="max-h-60 overflow-auto rounded-md border"
      style={{ borderColor: "var(--color-border)" }}
    >
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead className="text-xs">
              {showSource ? "Source" : "Target URL"}
            </TableHead>
            <TableHead className="w-32 text-xs">Anchor Text</TableHead>
            <TableHead className="w-16 text-xs">Type</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {links.map((link) => (
            <TableRow key={link.id}>
              <TableCell className="max-w-xs truncate py-1.5 font-mono text-xs">
                {showSource ? `#${link.sourcePage}` : link.targetUrl}
              </TableCell>
              <TableCell className="truncate py-1.5 text-xs">
                {link.anchorText || <span className="italic opacity-40">empty</span>}
              </TableCell>
              <TableCell className="py-1.5">
                <Badge variant="outline" className="text-xs">
                  {link.linkType}
                </Badge>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

function sortIssues(issues: IssueRow[]): IssueRow[] {
  const order: Record<string, number> = { error: 0, warning: 1, info: 2 };
  return [...issues].sort((a, b) => (order[a.severity] ?? 3) - (order[b.severity] ?? 3));
}

// ---------------------------------------------------------------------------
// AI Analysis section
// ---------------------------------------------------------------------------

function AiAnalysisSection({ crawlId, pageId }: { crawlId: string; pageId: number }) {
  const [analyses, setAnalyses] = useState<AiAnalysisRow[]>([]);
  const [isConfigured, setIsConfigured] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const loadAnalyses = useCallback(async () => {
    setIsLoading(true);
    try {
      const [config, cached] = await Promise.all([
        getAiConfig(),
        getPageAnalyses(crawlId, pageId),
      ]);
      setIsConfigured(config.isConfigured);
      setAnalyses(cached);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsLoading(false);
    }
  }, [crawlId, pageId]);

  useEffect(() => {
    loadAnalyses();
  }, [loadAnalyses]);

  const handleAnalyze = async () => {
    setIsAnalyzing(true);
    setError(null);
    try {
      const results = await analyzePage(crawlId, pageId, [
        "content_score",
        "meta_desc",
        "title_tag",
      ]);
      setAnalyses(results);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsAnalyzing(false);
    }
  };

  const copyText = (text: string) => {
    navigator.clipboard.writeText(text);
  };

  if (isLoading) return null;

  return (
    <section>
      <h3 className="mb-3 text-sm font-semibold">AI Analysis</h3>

      {!isConfigured && analyses.length === 0 && (
        <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
          Configure an AI provider in Settings to enable AI analysis.
        </p>
      )}

      {isConfigured && analyses.length === 0 && (
        <button
          onClick={handleAnalyze}
          disabled={isAnalyzing}
          className="rounded-md border px-4 py-2 text-sm font-medium disabled:opacity-50"
          style={{ borderColor: "var(--color-border)" }}
        >
          {isAnalyzing ? (
            <span className="flex items-center gap-2">
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
              Analyzing...
            </span>
          ) : (
            "Analyze Page"
          )}
        </button>
      )}

      {error && (
        <p className="mt-2 text-xs" style={{ color: "var(--color-severity-error)" }}>
          {error}
        </p>
      )}

      {analyses.length > 0 && (
        <div className="space-y-4">
          {analyses.map((a) => (
            <AnalysisResult key={a.id} analysis={a} onCopy={copyText} />
          ))}
          {isConfigured && (
            <button
              onClick={handleAnalyze}
              disabled={isAnalyzing}
              className="text-xs underline opacity-60 hover:opacity-100"
            >
              {isAnalyzing ? "Re-analyzing..." : "Re-analyze"}
            </button>
          )}
        </div>
      )}
    </section>
  );
}

function AnalysisResult({
  analysis,
  onCopy,
}: {
  analysis: AiAnalysisRow;
  onCopy: (text: string) => void;
}) {
  const tryParse = <T,>(json: string): T | null => {
    try {
      return JSON.parse(json) as T;
    } catch {
      return null;
    }
  };

  if (analysis.analysisType === "content_score") {
    const result = tryParse<ContentScoreResult>(analysis.resultJson);
    if (!result) return <RawResult analysis={analysis} />;
    return (
      <div
        className="rounded-md border p-3"
        style={{ borderColor: "var(--color-border)" }}
      >
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium">Content Quality</span>
          <span className="text-lg font-bold tabular-nums">
            {result.overallScore}/100
          </span>
        </div>
        <div className="mt-2 grid grid-cols-3 gap-2 text-center">
          <MiniScore label="Relevance" score={result.relevanceScore} />
          <MiniScore label="Readability" score={result.readabilityScore} />
          <MiniScore label="Depth" score={result.depthScore} />
        </div>
        {result.reasoning && (
          <p className="mt-2 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
            {result.reasoning}
          </p>
        )}
      </div>
    );
  }

  if (analysis.analysisType === "meta_desc") {
    const result = tryParse<MetaDescResult>(analysis.resultJson);
    if (!result) return <RawResult analysis={analysis} />;
    return (
      <div
        className="rounded-md border p-3"
        style={{ borderColor: "var(--color-border)" }}
      >
        <span className="text-xs font-medium">Suggested Meta Description</span>
        <div className="mt-1 flex items-start gap-2">
          <p className="flex-1 text-sm">{result.suggested}</p>
          <button
            onClick={() => onCopy(result.suggested)}
            className="shrink-0 opacity-50 hover:opacity-100"
            title="Copy"
          >
            <Copy className="h-3.5 w-3.5" />
          </button>
        </div>
        <p className="mt-1 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
          {result.charCount} chars
        </p>
      </div>
    );
  }

  if (analysis.analysisType === "title_tag") {
    const result = tryParse<TitleTagResult>(analysis.resultJson);
    if (!result) return <RawResult analysis={analysis} />;
    return (
      <div
        className="rounded-md border p-3"
        style={{ borderColor: "var(--color-border)" }}
      >
        <span className="text-xs font-medium">Title Suggestions</span>
        <div className="mt-1 space-y-1">
          {result.suggestions.map((s, i) => (
            <div key={i} className="flex items-center gap-2">
              <span className="flex-1 text-sm">{s.title}</span>
              <span
                className="shrink-0 text-xs tabular-nums"
                style={{ color: "var(--color-muted-foreground)" }}
              >
                {s.charCount}ch
              </span>
              <button
                onClick={() => onCopy(s.title)}
                className="shrink-0 opacity-50 hover:opacity-100"
                title="Copy"
              >
                <Copy className="h-3 w-3" />
              </button>
            </div>
          ))}
        </div>
      </div>
    );
  }

  return <RawResult analysis={analysis} />;
}

function RawResult({ analysis }: { analysis: AiAnalysisRow }) {
  return (
    <div className="rounded-md border p-3" style={{ borderColor: "var(--color-border)" }}>
      <span className="text-xs font-medium">{analysis.analysisType}</span>
      <pre
        className="mt-1 max-h-32 overflow-auto text-xs whitespace-pre-wrap"
        style={{ color: "var(--color-muted-foreground)" }}
      >
        {analysis.resultJson}
      </pre>
    </div>
  );
}

function MiniScore({ label, score }: { label: string; score: number }) {
  return (
    <div>
      <p className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
        {label}
      </p>
      <p className="text-sm font-semibold tabular-nums">{score}</p>
    </div>
  );
}
