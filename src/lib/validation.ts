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
});

export type CrawlConfigFormValues = z.infer<typeof crawlConfigSchema>;

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
};
