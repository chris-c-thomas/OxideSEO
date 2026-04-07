/**
 * Zod validation schemas for OxideSEO forms and data.
 *
 * Used for frontend form validation before invoking Tauri commands.
 */

import { z } from "zod";

/** Crawl configuration validation schema. */
export const crawlConfigSchema = z.object({
  startUrl: z.string().min(1, "Start URL is required").url("Must be a valid URL"),
  maxDepth: z.number().int().min(1).max(100).default(10),
  maxConcurrency: z.number().int().min(1).max(200).default(50),
  fetchWorkers: z.number().int().min(1).max(32).default(8),
  parseThreads: z.number().int().min(1).max(64).default(6),
  maxMemoryMb: z.number().int().min(128).max(16384).default(512),
  respectRobotsTxt: z.boolean().default(true),
  includePatterns: z.array(z.string()).default([]),
  excludePatterns: z.array(z.string()).default([]),
  requestTimeoutSecs: z.number().int().min(5).max(120).default(30),
  crawlDelayMs: z.number().int().min(0).max(10000).default(500),
  userAgent: z.string().nullable().default(null),
  customHeaders: z.array(z.tuple([z.string(), z.string()])).default([]),
  perHostConcurrency: z.number().int().min(1).max(20).default(2),
  maxPages: z.number().int().min(0).default(0),
  // Phase 6: Advanced crawl features
  enableJsRendering: z.boolean().default(false),
  jsRenderMaxConcurrent: z.number().int().min(1).max(8).default(2),
  jsRenderPatterns: z.array(z.string()).default([]),
  jsNeverRenderPatterns: z.array(z.string()).default([]),
  enableSitemapDiscovery: z.boolean().default(true),
  enableExternalLinkCheck: z.boolean().default(false),
  externalLinkConcurrency: z.number().int().min(1).max(50).default(5),
  urlRewriteRules: z.array(z.tuple([z.string(), z.string()])).default([]),
  cookies: z.array(z.tuple([z.string(), z.string()])).default([]),
  customCssSelectors: z.array(z.tuple([z.string(), z.string()])).default([]),
});

export type CrawlConfigFormValues = z.infer<typeof crawlConfigSchema>;

// ---------------------------------------------------------------------------
// Plugin schemas (Phase 8)
// ---------------------------------------------------------------------------

export const pluginKindSchema = z.enum([
  "rule",
  "exporter",
  "post_processor",
  "ui_extension",
]);

export const pluginInfoSchema = z.object({
  name: z.string(),
  version: z.string(),
  description: z.string(),
  kind: pluginKindSchema,
  enabled: z.boolean(),
  isNative: z.boolean(),
  loadError: z.string().nullable(),
});

export const pluginDetailSchema = z.object({
  name: z.string(),
  version: z.string(),
  description: z.string(),
  author: z.string().nullable(),
  license: z.string().nullable(),
  kind: pluginKindSchema,
  capabilities: z.array(z.string()),
  enabled: z.boolean(),
  isNative: z.boolean(),
  config: z.record(z.unknown()).nullable(),
  installedAt: z.string(),
});

/** Default values for a new crawl configuration form. */
export const defaultCrawlConfig: CrawlConfigFormValues = {
  startUrl: "",
  maxDepth: 10,
  maxConcurrency: 50,
  fetchWorkers: 8,
  parseThreads: 6,
  maxMemoryMb: 512,
  respectRobotsTxt: true,
  includePatterns: [],
  excludePatterns: [],
  requestTimeoutSecs: 30,
  crawlDelayMs: 500,
  userAgent: null,
  customHeaders: [],
  perHostConcurrency: 2,
  maxPages: 0,
  enableJsRendering: false,
  jsRenderMaxConcurrent: 2,
  jsRenderPatterns: [],
  jsNeverRenderPatterns: [],
  enableSitemapDiscovery: true,
  enableExternalLinkCheck: false,
  externalLinkConcurrency: 5,
  urlRewriteRules: [],
  cookies: [],
  customCssSelectors: [],
};
