/**
 * Results Explorer: tabbed interface for viewing crawl results.
 *
 * Tabs: All Pages | Issues | Links | Images
 *
 * Each tab uses a virtualized table (TanStack Table + TanStack Virtual)
 * with server-side sorting, filtering, and infinite scroll.
 */

import { useState } from "react";
import { cn } from "@/lib/utils";

// TODO(phase-4): Import and wire up the DataTable component with
// TanStack Table + Virtual for each tab.

interface ResultsExplorerProps {
  crawlId: string | null;
}

type ResultsTab = "pages" | "issues" | "links" | "images";

const TABS: { id: ResultsTab; label: string }[] = [
  { id: "pages", label: "All Pages" },
  { id: "issues", label: "Issues" },
  { id: "links", label: "Links" },
  { id: "images", label: "Images" },
];

export function ResultsExplorer({ crawlId }: ResultsExplorerProps) {
  const [activeTab, setActiveTab] = useState<ResultsTab>("pages");

  if (!crawlId) {
    return (
      <div className="flex h-full items-center justify-center">
        <p style={{ color: "var(--color-muted-foreground)" }}>
          Select a crawl from the Dashboard to view results.
        </p>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header with tabs */}
      <div
        className="flex items-center gap-4 border-b px-6 pt-6"
        style={{ borderColor: "var(--color-border)" }}
      >
        <h1 className="text-lg font-bold tracking-tight">Results</h1>
        <div className="flex gap-1">
          {TABS.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={cn(
                "rounded-t-md px-3 py-2 text-sm transition-colors",
                activeTab === tab.id && "font-medium",
              )}
              style={{
                backgroundColor:
                  activeTab === tab.id ? "var(--color-background)" : "transparent",
                borderBottom:
                  activeTab === tab.id
                    ? "2px solid var(--color-primary)"
                    : "2px solid transparent",
              }}
            >
              {tab.label}
            </button>
          ))}
        </div>
      </div>

      {/* Tab content area */}
      <div className="flex-1 overflow-hidden p-6">
        {activeTab === "pages" && <PagesTab crawlId={crawlId} />}
        {activeTab === "issues" && <IssuesTab crawlId={crawlId} />}
        {activeTab === "links" && <LinksTab crawlId={crawlId} />}
        {activeTab === "images" && <ImagesTab crawlId={crawlId} />}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Tab placeholders — each will use the shared DataTable component.
// ---------------------------------------------------------------------------

function PagesTab({ crawlId: _crawlId }: { crawlId: string }) {
  // TODO(phase-4): Implement with DataTable, TanStack Table columns for:
  // URL, Status Code, Title, Meta Description, H1, Word Count,
  // Response Time, Body Size, Canonical, Issues Count.
  // Server-side sorting and filtering via getCrawlResults.
  // Infinite scroll with prefetch at 20-row boundary.
  return (
    <PlaceholderTable
      heading="All Pages"
      description="Virtualized table with 100k+ row support. Implement in Phase 4 with TanStack Table + Virtual."
    />
  );
}

function IssuesTab({ crawlId: _crawlId }: { crawlId: string }) {
  // TODO(phase-4): Columns: Severity Badge, Rule ID, Page URL, Message, Category.
  // Group by severity, then by category.
  return (
    <PlaceholderTable
      heading="Issues"
      description="Issue list grouped by severity and category. Implement in Phase 4."
    />
  );
}

function LinksTab({ crawlId: _crawlId }: { crawlId: string }) {
  // TODO(phase-4): Columns: Source URL, Target URL, Anchor Text, Link Type,
  // Internal/External, Status, Nofollow.
  return (
    <PlaceholderTable
      heading="Links"
      description="Link inventory with type and status filtering. Implement in Phase 4."
    />
  );
}

function ImagesTab({ crawlId: _crawlId }: { crawlId: string }) {
  // TODO(phase-4): Columns: Image URL, Source Page, Alt Text, File Size, Status.
  // Filter by missing alt, broken, oversized.
  return (
    <PlaceholderTable
      heading="Images"
      description="Image inventory with alt text and size analysis. Implement in Phase 4."
    />
  );
}

function PlaceholderTable({
  heading,
  description,
}: {
  heading: string;
  description: string;
}) {
  return (
    <div
      className="flex h-full items-center justify-center rounded-lg border"
      style={{ borderColor: "var(--color-border)" }}
    >
      <div className="text-center">
        <p className="text-sm font-medium">{heading}</p>
        <p className="mt-1 text-xs" style={{ color: "var(--color-muted-foreground)" }}>
          {description}
        </p>
      </div>
    </div>
  );
}
