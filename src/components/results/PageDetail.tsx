import { useEffect, useState } from "react";
import { getPageDetail } from "@/lib/commands";
import type { PageDetail as PageDetailType, IssueRow, LinkRow, Severity } from "@/types";
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
