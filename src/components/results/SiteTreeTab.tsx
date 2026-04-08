/**
 * Site tree tab: hierarchical collapsible tree view of crawled URLs.
 *
 * Builds a path-segment tree from the backend, color-coded by HTTP status.
 * Click a leaf node to open the page detail sheet.
 */

import { useEffect, useState, useCallback } from "react";
import { getSiteTree } from "@/lib/commands";
import type { SiteTreeNode } from "@/types";

interface SiteTreeTabProps {
  crawlId: string;
  onPageClick?: (pageId: number) => void;
}

export function SiteTreeTab({ crawlId, onPageClick }: SiteTreeTabProps) {
  const [tree, setTree] = useState<SiteTreeNode[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setIsLoading(true);
    setError(null);
    getSiteTree(crawlId)
      .then(setTree)
      .catch((err) => setError(String(err)))
      .finally(() => setIsLoading(false));
  }, [crawlId]);

  if (isLoading) {
    return (
      <div className="flex h-64 items-center justify-center">
        <p style={{ color: "var(--color-muted-foreground)" }}>Loading site tree...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="p-4">
        <p style={{ color: "var(--color-severity-error)" }}>
          Failed to load site tree: {error}
        </p>
      </div>
    );
  }

  if (tree.length === 0) {
    return (
      <div className="flex h-64 items-center justify-center">
        <p style={{ color: "var(--color-muted-foreground)" }}>No pages found.</p>
      </div>
    );
  }

  return (
    <div
      className="custom-scrollbar overflow-auto p-4"
      style={{ maxHeight: "calc(100vh - 220px)" }}
    >
      {tree.map((root) => (
        <TreeNodeRow key={root.label} node={root} depth={0} onPageClick={onPageClick} />
      ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Tree node component
// ---------------------------------------------------------------------------

interface TreeNodeRowProps {
  node: SiteTreeNode;
  depth: number;
  onPageClick?: (pageId: number) => void;
}

function TreeNodeRow({ node, depth, onPageClick }: TreeNodeRowProps) {
  const [isExpanded, setIsExpanded] = useState(depth < 1);
  const hasChildren = node.children.length > 0;
  const isPage = node.pageId != null;

  const handleToggle = useCallback(() => {
    if (hasChildren) {
      setIsExpanded((prev) => !prev);
    }
  }, [hasChildren]);

  const handlePageClick = useCallback(() => {
    if (isPage && onPageClick && node.pageId != null) {
      onPageClick(node.pageId);
    }
  }, [isPage, onPageClick, node.pageId]);

  return (
    <div>
      <div
        className="flex items-center gap-1.5 rounded px-1 py-0.5 text-sm hover:bg-[var(--color-muted)]"
        style={{ paddingLeft: `${depth * 20 + 4}px` }}
      >
        {/* Expand/collapse toggle */}
        <button
          onClick={handleToggle}
          className="flex h-5 w-5 shrink-0 items-center justify-center rounded text-xs"
          style={{
            color: "var(--color-muted-foreground)",
            visibility: hasChildren ? "visible" : "hidden",
          }}
          aria-label={isExpanded ? "Collapse" : "Expand"}
        >
          {isExpanded ? "\u25BE" : "\u25B8"}
        </button>

        {/* Status indicator */}
        {isPage && (
          <span
            className="inline-block h-2 w-2 shrink-0 rounded-full"
            style={{ backgroundColor: statusColor(node.statusCode) }}
            title={node.statusCode != null ? `HTTP ${node.statusCode}` : "No status"}
          />
        )}
        {!isPage && hasChildren && (
          <span
            className="inline-block h-2 w-2 shrink-0 rounded-sm"
            style={{ backgroundColor: "var(--color-muted-foreground)", opacity: 0.4 }}
          />
        )}

        {/* Label */}
        {isPage ? (
          <button
            onClick={handlePageClick}
            className="truncate text-left hover:underline"
            style={{ color: "var(--color-foreground)" }}
            title={node.url ?? node.label}
          >
            {node.label}
          </button>
        ) : (
          <span
            className="truncate font-medium"
            style={{ color: "var(--color-foreground)" }}
            title={node.label}
          >
            {node.label}/
          </span>
        )}

        {/* Page count badge */}
        {hasChildren && (
          <span
            className="ml-auto shrink-0 rounded-full px-1.5 text-xs tabular-nums"
            style={{
              backgroundColor: "var(--color-muted)",
              color: "var(--color-muted-foreground)",
            }}
          >
            {node.pageCount}
          </span>
        )}

        {/* Status code */}
        {isPage && node.statusCode != null && (
          <span
            className="shrink-0 text-xs tabular-nums"
            style={{ color: statusColor(node.statusCode) }}
          >
            {node.statusCode}
          </span>
        )}
      </div>

      {/* Children */}
      {isExpanded &&
        hasChildren &&
        node.children.map((child) => (
          <TreeNodeRow
            key={child.label}
            node={child}
            depth={depth + 1}
            onPageClick={onPageClick}
          />
        ))}
    </div>
  );
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function statusColor(code: number | null): string {
  if (code == null) return "var(--color-muted-foreground)";
  if (code >= 500) return "var(--color-severity-error)";
  if (code >= 400) return "var(--color-severity-warning)";
  if (code >= 300) return "var(--color-primary)";
  return "var(--color-status-completed)";
}
