/**
 * Export dialog for exporting crawl data as CSV, NDJSON, or HTML report.
 * Supports format selection, data type selection, and column filtering (CSV).
 */

import { useState, useEffect } from "react";
import { exportData } from "@/lib/commands";
import type { ExportDataType, ExportResult } from "@/types";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";

type ExportFormat = "csv" | "json" | "html";

interface ExportDialogProps {
  crawlId: string;
  open: boolean;
  onClose: () => void;
  /** Pre-select data type based on the active results tab. */
  activeTab?: "pages" | "issues" | "links" | "images";
}

const FORMAT_OPTIONS: { value: ExportFormat; label: string; description: string }[] = [
  { value: "csv", label: "CSV", description: "Spreadsheet-compatible data" },
  { value: "json", label: "JSON Lines", description: "Line-delimited JSON" },
  { value: "html", label: "HTML Report", description: "Summary report with stats" },
];

const DATA_TYPE_OPTIONS: { value: ExportDataType; label: string }[] = [
  { value: "pages", label: "Pages" },
  { value: "issues", label: "Issues" },
  { value: "links", label: "Links" },
  { value: "images", label: "Images" },
];

const COLUMNS_BY_TYPE: Record<string, { key: string; label: string }[]> = {
  pages: [
    { key: "url", label: "URL" },
    { key: "statusCode", label: "Status Code" },
    { key: "title", label: "Title" },
    { key: "metaDesc", label: "Meta Description" },
    { key: "h1", label: "H1" },
    { key: "canonical", label: "Canonical" },
    { key: "contentType", label: "Content Type" },
    { key: "responseTimeMs", label: "Response Time (ms)" },
    { key: "bodySize", label: "Body Size" },
    { key: "depth", label: "Depth" },
    { key: "state", label: "State" },
    { key: "robotsDirectives", label: "Robots Directives" },
    { key: "fetchedAt", label: "Fetched At" },
    { key: "errorMessage", label: "Error Message" },
  ],
  issues: [
    { key: "ruleId", label: "Rule ID" },
    { key: "severity", label: "Severity" },
    { key: "category", label: "Category" },
    { key: "message", label: "Message" },
    { key: "pageId", label: "Page ID" },
    { key: "detailJson", label: "Detail (JSON)" },
  ],
  links: [
    { key: "sourcePage", label: "Source Page" },
    { key: "targetUrl", label: "Target URL" },
    { key: "anchorText", label: "Anchor Text" },
    { key: "linkType", label: "Link Type" },
    { key: "isInternal", label: "Internal" },
    { key: "nofollow", label: "Nofollow" },
  ],
  images: [
    { key: "targetUrl", label: "Image URL" },
    { key: "sourcePage", label: "Source Page" },
    { key: "anchorText", label: "Alt Text" },
    { key: "isInternal", label: "Internal" },
  ],
};

function tabToDataType(tab?: string): ExportDataType {
  if (tab === "pages" || tab === "issues" || tab === "links" || tab === "images")
    return tab;
  return "pages";
}

export function ExportDialog({ crawlId, open, onClose, activeTab }: ExportDialogProps) {
  const [format, setFormat] = useState<ExportFormat>("csv");
  const [dataType, setDataType] = useState<ExportDataType>(tabToDataType(activeTab));
  const [selectedColumns, setSelectedColumns] = useState<Set<string>>(
    new Set(COLUMNS_BY_TYPE[dataType]?.map((c) => c.key) ?? []),
  );
  const [isExporting, setIsExporting] = useState(false);
  const [result, setResult] = useState<ExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Sync dataType with activeTab when the dialog opens.
  useEffect(() => {
    if (open) {
      const dt = tabToDataType(activeTab);
      setDataType(dt);
      setSelectedColumns(new Set(COLUMNS_BY_TYPE[dt]?.map((c) => c.key) ?? []));
      setResult(null);
      setError(null);
    }
  }, [open, activeTab]);

  const isReport = format === "html";
  const showColumns = format === "csv";
  const availableColumns = COLUMNS_BY_TYPE[dataType] ?? [];

  const handleDataTypeChange = (dt: ExportDataType) => {
    setDataType(dt);
    setSelectedColumns(new Set(COLUMNS_BY_TYPE[dt]?.map((c) => c.key) ?? []));
    setResult(null);
    setError(null);
  };

  const handleFormatChange = (f: ExportFormat) => {
    setFormat(f);
    setResult(null);
    setError(null);
  };

  const toggleColumn = (key: string) => {
    setSelectedColumns((prev) => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  };

  const selectAllColumns = () => {
    setSelectedColumns(new Set(availableColumns.map((c) => c.key)));
  };

  const deselectAllColumns = () => {
    setSelectedColumns(new Set());
  };

  const handleExport = async () => {
    setIsExporting(true);
    setError(null);
    setResult(null);

    try {
      const res = await exportData({
        crawlId,
        format,
        dataType: isReport ? "full_report" : dataType,
        columns: showColumns ? Array.from(selectedColumns) : null,
      });
      setResult(res);
    } catch (err) {
      const message = String(err);
      if (!message.toLowerCase().includes("cancelled")) {
        setError(message);
      }
    } finally {
      setIsExporting(false);
    }
  };

  const handleClose = () => {
    setResult(null);
    setError(null);
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={(isOpen) => !isOpen && handleClose()}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle>Export Crawl Data</DialogTitle>
          <DialogDescription>Choose a format and data type to export.</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {/* Format selector */}
          <div className="space-y-2">
            <Label>Format</Label>
            <div className="flex gap-2">
              {FORMAT_OPTIONS.map((opt) => (
                <button
                  key={opt.value}
                  onClick={() => handleFormatChange(opt.value)}
                  className="flex-1 rounded-md border px-3 py-2 text-left text-sm transition-colors"
                  style={{
                    borderColor:
                      format === opt.value
                        ? "var(--color-primary)"
                        : "var(--color-border)",
                    backgroundColor:
                      format === opt.value ? "var(--color-primary)" : "transparent",
                    color:
                      format === opt.value
                        ? "var(--color-primary-foreground)"
                        : "var(--color-foreground)",
                  }}
                >
                  <div className="font-medium">{opt.label}</div>
                  <div
                    className="text-xs"
                    style={{
                      color:
                        format === opt.value
                          ? "var(--color-primary-foreground)"
                          : "var(--color-muted-foreground)",
                    }}
                  >
                    {opt.description}
                  </div>
                </button>
              ))}
            </div>
          </div>

          {/* Data type selector (hidden for HTML report) */}
          {!isReport && (
            <div className="space-y-2">
              <Label>Data Type</Label>
              <div className="flex gap-2">
                {DATA_TYPE_OPTIONS.map((opt) => (
                  <button
                    key={opt.value}
                    onClick={() => handleDataTypeChange(opt.value)}
                    className="rounded-md border px-3 py-1.5 text-sm transition-colors"
                    style={{
                      borderColor:
                        dataType === opt.value
                          ? "var(--color-primary)"
                          : "var(--color-border)",
                      backgroundColor:
                        dataType === opt.value ? "var(--color-muted)" : "transparent",
                    }}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Column selector (CSV only) */}
          {showColumns && availableColumns.length > 0 && (
            <div className="space-y-2">
              <div className="flex items-center justify-between">
                <Label>Columns</Label>
                <div className="flex gap-2 text-xs">
                  <button
                    onClick={selectAllColumns}
                    className="underline"
                    style={{ color: "var(--color-primary)" }}
                  >
                    All
                  </button>
                  <button
                    onClick={deselectAllColumns}
                    className="underline"
                    style={{ color: "var(--color-muted-foreground)" }}
                  >
                    None
                  </button>
                </div>
              </div>
              <div className="grid grid-cols-2 gap-x-4 gap-y-1.5">
                {availableColumns.map((col) => (
                  <label key={col.key} className="flex items-center gap-2 text-sm">
                    <Checkbox
                      checked={selectedColumns.has(col.key)}
                      onCheckedChange={() => toggleColumn(col.key)}
                    />
                    {col.label}
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Result / Error */}
          {result && (
            <div
              className="rounded-md border p-3 text-sm"
              style={{
                borderColor: "var(--color-border)",
                backgroundColor: "var(--color-muted)",
              }}
            >
              Exported{" "}
              {result.rowsExported > 0
                ? `${result.rowsExported.toLocaleString()} rows`
                : "report"}{" "}
              to <span className="font-medium">{result.filePath.split("/").pop()}</span>
            </div>
          )}
          {error && (
            <div
              className="rounded-md border p-3 text-sm"
              style={{ borderColor: "var(--color-severity-error)" }}
            >
              <span style={{ color: "var(--color-severity-error)" }}>{error}</span>
            </div>
          )}
        </div>

        <DialogFooter>
          <button
            onClick={handleClose}
            className="rounded-md border px-4 py-2 text-sm"
            style={{ borderColor: "var(--color-border)" }}
          >
            {result ? "Done" : "Cancel"}
          </button>
          {!result && (
            <button
              onClick={handleExport}
              disabled={isExporting || (showColumns && selectedColumns.size === 0)}
              className="rounded-md px-4 py-2 text-sm font-medium disabled:opacity-50"
              style={{
                backgroundColor: "var(--color-primary)",
                color: "var(--color-primary-foreground)",
              }}
            >
              {isExporting ? "Exporting..." : "Export"}
            </button>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
