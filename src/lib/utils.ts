/**
 * Shared utility functions.
 */

import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/** Merge CSS class names with Tailwind conflict resolution. */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

/** Format a duration in milliseconds to a human-readable string. */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  if (minutes < 60) return `${minutes}m ${remainingSeconds}s`;
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours}h ${remainingMinutes}m`;
}

/** Format a byte count to a human-readable string. */
export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

/** Format a number with commas. */
export function formatNumber(n: number): string {
  return n.toLocaleString();
}

/** Format requests per second. */
export function formatRps(rps: number): string {
  return `${rps.toFixed(1)} req/s`;
}

/** Truncate a string to a max length with ellipsis. */
export function truncate(str: string, maxLen: number): string {
  if (str.length <= maxLen) return str;
  return str.slice(0, maxLen - 1) + "\u2026";
}

/** Extract the domain from a URL string. */
export function extractDomain(url: string): string {
  try {
    return new URL(url).hostname;
  } catch {
    return url;
  }
}

/** Map a severity string to a CSS color variable name. */
export function severityColor(severity: "error" | "warning" | "info"): string {
  return `var(--color-severity-${severity})`;
}

/** Strip protocol and trailing slash from a URL for compact display. */
export function formatUrl(url: string): string {
  return url.replace(/^https?:\/\//, "").replace(/\/$/, "");
}

/** Map a status code to a CSS color string. */
export function statusCodeColor(code: number): string {
  if (code >= 500) return "var(--color-severity-error)";
  if (code >= 400) return "var(--color-severity-warning)";
  if (code >= 300) return "var(--color-primary)";
  return "var(--color-status-completed)";
}

/** Map a crawl state to a CSS color variable name. */
export function stateColor(state: string): string {
  switch (state) {
    case "running":
      return "var(--color-status-running)";
    case "paused":
      return "var(--color-status-paused)";
    case "completed":
      return "var(--color-status-completed)";
    case "error":
    case "stopped":
      return "var(--color-status-error)";
    default:
      return "var(--color-muted-foreground)";
  }
}
