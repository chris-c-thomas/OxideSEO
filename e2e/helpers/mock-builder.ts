/**
 * Fluent builder for composing Tauri IPC mock command maps.
 *
 * Provides sensible defaults for all known commands so tests only
 * need to override the data relevant to their scenario.
 */

import type { MockCommandMap } from "../setup/tauri-mock";
import type {
  AppSettings,
  AiProviderConfig,
  CrawlSummary,
  CrawlStatus,
  CrawlComparisonSummary,
  ExportResult,
  PageDetail,
  PageRow,
  IssueRow,
  LinkRow,
  ExternalLinkRow,
  SiteTreeNode,
  SitemapReportEntry,
  PaginatedResponse,
  PluginInfo,
  RuleConfigOverride,
  AiAnalysisRow,
  AiUsageRow,
  AiCrawlSummaryRow,
} from "@/types";

const DEFAULT_SETTINGS: AppSettings = {
  defaultCrawlConfig: {},
  theme: "light",
  defaultExportFormat: "csv",
};

const DEFAULT_AI_CONFIG: AiProviderConfig = {
  providerType: "open_ai",
  model: "gpt-4o",
  ollamaEndpoint: null,
  maxTokensPerCrawl: 100000,
  isConfigured: false,
};

const DEFAULT_CRAWL_STATUS: CrawlStatus = {
  crawlId: "",
  state: "completed",
  urlsCrawled: 0,
  urlsQueued: 0,
  urlsErrored: 0,
  elapsedMs: 0,
  currentRps: 0,
};

function emptyPaginated<T>(): PaginatedResponse<T> {
  return { items: [], total: 0, offset: 0, limit: 50 };
}

export class TauriMockBuilder {
  private commands: MockCommandMap;

  constructor() {
    this.commands = {};
    this.withDefaults();
  }

  /** Pre-populate all known commands with safe defaults. */
  withDefaults(): this {
    this.commands = {
      // Crawl lifecycle
      start_crawl: "mock-crawl-id",
      pause_crawl: null,
      resume_crawl: null,
      stop_crawl: null,
      delete_crawl: null,
      rerun_crawl: "mock-rerun-crawl-id",
      get_crawl_status: DEFAULT_CRAWL_STATUS,

      // Result queries
      get_recent_crawls: [],
      get_crawl_results: emptyPaginated<PageRow>(),
      get_crawl_summary: null,
      get_page_detail: null,
      get_issues: emptyPaginated<IssueRow>(),
      get_sitemap_report: [],
      get_external_links: emptyPaginated<ExternalLinkRow>(),
      get_site_tree: [],
      get_links: emptyPaginated<LinkRow>(),

      // Crawl comparison
      get_comparison_summary: null,
      get_page_diffs: emptyPaginated(),
      get_issue_diffs: emptyPaginated(),
      get_metadata_diffs: emptyPaginated(),

      // Settings
      get_settings: DEFAULT_SETTINGS,
      set_settings: null,
      get_rule_config: [],
      set_rule_config: null,

      // Export
      export_data: { filePath: "/tmp/export.csv", rowsExported: 0 },
      save_crawl_file: null,
      open_crawl_file: null,

      // AI
      get_ai_config: DEFAULT_AI_CONFIG,
      set_ai_config: null,
      set_api_key: null,
      delete_api_key: null,
      has_api_key: false,
      test_ai_connection: "Connected successfully",
      analyze_page: [],
      batch_analyze_pages: {
        pagesAnalyzed: 0,
        totalInputTokens: 0,
        totalOutputTokens: 0,
        totalCostUsd: 0,
        errors: 0,
        budgetExhausted: false,
      },
      generate_crawl_summary: null,
      get_page_analyses: [],
      get_ai_usage: [],
      get_crawl_ai_summary: null,
      list_ollama_models: [],
      estimate_batch_cost: {
        eligiblePages: 0,
        estimatedInputTokens: 0,
        estimatedOutputTokens: 0,
        estimatedCostUsd: 0,
      },

      // Plugins
      list_plugins: [],
      enable_plugin: null,
      disable_plugin: null,
      get_plugin_detail: null,
      reload_plugins: [],
      install_plugin_from_file: null,
      uninstall_plugin: null,
    };
    return this;
  }

  // -------------------------------------------------------------------------
  // Crawl lifecycle
  // -------------------------------------------------------------------------

  withRecentCrawls(crawls: CrawlSummary[]): this {
    this.commands.get_recent_crawls = crawls;
    return this;
  }

  withCrawlStatus(status: CrawlStatus): this {
    this.commands.get_crawl_status = status;
    return this;
  }

  withStartCrawlId(crawlId: string): this {
    this.commands.start_crawl = crawlId;
    return this;
  }

  // -------------------------------------------------------------------------
  // Results
  // -------------------------------------------------------------------------

  withCrawlSummary(summary: CrawlSummary): this {
    this.commands.get_crawl_summary = summary;
    return this;
  }

  withCrawlResults(results: PaginatedResponse<PageRow>): this {
    this.commands.get_crawl_results = results;
    return this;
  }

  withPageDetail(detail: PageDetail): this {
    this.commands.get_page_detail = detail;
    return this;
  }

  withIssues(issues: PaginatedResponse<IssueRow>): this {
    this.commands.get_issues = issues;
    return this;
  }

  withLinks(links: PaginatedResponse<LinkRow>): this {
    this.commands.get_links = links;
    return this;
  }

  withExternalLinks(links: PaginatedResponse<ExternalLinkRow>): this {
    this.commands.get_external_links = links;
    return this;
  }

  withSiteTree(tree: SiteTreeNode[]): this {
    this.commands.get_site_tree = tree;
    return this;
  }

  withSitemapReport(report: SitemapReportEntry[]): this {
    this.commands.get_sitemap_report = report;
    return this;
  }

  // -------------------------------------------------------------------------
  // Comparison
  // -------------------------------------------------------------------------

  withComparisonSummary(summary: CrawlComparisonSummary): this {
    this.commands.get_comparison_summary = summary;
    return this;
  }

  // -------------------------------------------------------------------------
  // Settings
  // -------------------------------------------------------------------------

  withSettings(settings: AppSettings): this {
    this.commands.get_settings = settings;
    return this;
  }

  withAiConfig(config: AiProviderConfig): this {
    this.commands.get_ai_config = config;
    return this;
  }

  withHasApiKey(hasKey: boolean): this {
    this.commands.has_api_key = hasKey;
    return this;
  }

  withRuleConfig(overrides: RuleConfigOverride[]): this {
    this.commands.get_rule_config = overrides;
    return this;
  }

  // -------------------------------------------------------------------------
  // Export
  // -------------------------------------------------------------------------

  withExportResult(result: ExportResult): this {
    this.commands.export_data = result;
    return this;
  }

  // -------------------------------------------------------------------------
  // AI
  // -------------------------------------------------------------------------

  withPageAnalyses(analyses: AiAnalysisRow[]): this {
    this.commands.get_page_analyses = analyses;
    return this;
  }

  withAiUsage(usage: AiUsageRow[]): this {
    this.commands.get_ai_usage = usage;
    return this;
  }

  withCrawlAiSummary(summary: AiCrawlSummaryRow): this {
    this.commands.get_crawl_ai_summary = summary;
    return this;
  }

  // -------------------------------------------------------------------------
  // Plugins
  // -------------------------------------------------------------------------

  withPlugins(plugins: PluginInfo[]): this {
    this.commands.list_plugins = plugins;
    return this;
  }

  // -------------------------------------------------------------------------
  // Generic
  // -------------------------------------------------------------------------

  /** Set a custom response for any command by name. */
  withCommand(name: string, response: unknown): this {
    this.commands[name] = response;
    return this;
  }

  /** Build the final command map. */
  build(): MockCommandMap {
    return { ...this.commands };
  }
}
