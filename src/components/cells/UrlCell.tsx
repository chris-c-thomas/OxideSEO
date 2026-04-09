/**
 * Truncated URL display for table cells.
 *
 * Shows a monospace truncated URL with a copy button visible on hover
 * and a tooltip displaying the full URL.
 */

import { useState } from "react";
import { Copy, ExternalLink, Check, X } from "lucide-react";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

interface UrlCellProps {
  url: string;
  maxLength?: number;
  className?: string;
}

type CopyState = "idle" | "copied" | "failed";

function formatDisplayUrl(url: string): string {
  try {
    const parsed = new URL(url);
    return parsed.pathname + parsed.search + parsed.hash;
  } catch {
    return url;
  }
}

export function UrlCell({ url, maxLength = 60, className }: UrlCellProps) {
  const [copyState, setCopyState] = useState<CopyState>("idle");

  const displayUrl = formatDisplayUrl(url);
  const isTruncated = displayUrl.length > maxLength;
  const truncated = isTruncated ? displayUrl.slice(0, maxLength) + "\u2026" : displayUrl;

  const handleCopy = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await navigator.clipboard.writeText(url);
      setCopyState("copied");
      setTimeout(() => setCopyState("idle"), 1500);
    } catch {
      setCopyState("failed");
      setTimeout(() => setCopyState("idle"), 2000);
    }
  };

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <div className={cn("group flex items-center gap-1", className)}>
          <span className="text-fg-default min-w-0 truncate font-mono text-xs">
            {truncated}
          </span>
          <button
            onClick={handleCopy}
            className={cn(
              "hidden shrink-0 rounded-[var(--radius-xs)] p-0.5 group-hover:inline-flex",
              copyState === "failed"
                ? "text-danger"
                : "text-fg-subtle hover:text-fg-default",
            )}
            aria-label={
              copyState === "copied"
                ? "Copied"
                : copyState === "failed"
                  ? "Copy failed"
                  : "Copy URL"
            }
          >
            {copyState === "copied" ? (
              <Check className="size-3" strokeWidth={1.75} />
            ) : copyState === "failed" ? (
              <X className="size-3" strokeWidth={1.75} />
            ) : (
              <Copy className="size-3" strokeWidth={1.75} />
            )}
          </button>
          <a
            href={url}
            target="_blank"
            rel="noopener noreferrer"
            onClick={(e) => e.stopPropagation()}
            className="text-fg-subtle hover:text-fg-default hidden shrink-0 rounded-[var(--radius-xs)] p-0.5 group-hover:inline-flex"
            aria-label="Open URL in browser"
          >
            <ExternalLink className="size-3" strokeWidth={1.75} />
          </a>
        </div>
      </TooltipTrigger>
      <TooltipContent side="bottom" className="max-w-[400px] font-mono text-xs break-all">
        {url}
      </TooltipContent>
    </Tooltip>
  );
}
