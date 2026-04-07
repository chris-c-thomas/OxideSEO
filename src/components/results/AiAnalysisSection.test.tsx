import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { PageDetail } from "./PageDetail";
import type {
  PageDetail as PageDetailType,
  AiProviderConfig,
  AiAnalysisRow,
} from "@/types";

const mockPage: PageDetailType = {
  page: {
    id: 1,
    crawlId: "crawl-1",
    url: "https://example.com/test",
    depth: 0,
    statusCode: 200,
    contentType: "text/html",
    responseTimeMs: 150,
    bodySize: 5000,
    title: "Test Page",
    metaDesc: "A test page",
    h1: "Hello",
    canonical: null,
    robotsDirectives: null,
    state: "analyzed",
    fetchedAt: "2026-04-06T00:00:00Z",
    errorMessage: null,
    customExtractions: null,
    isJsRendered: false,
    bodyText: "Some page content for analysis",
  },
  issues: [],
  inboundLinks: [],
  outboundLinks: [],
};

const configuredConfig: AiProviderConfig = {
  providerType: "open_ai",
  model: "gpt-4o",
  ollamaEndpoint: null,
  maxTokensPerCrawl: 100000,
  isConfigured: true,
};

const notConfiguredConfig: AiProviderConfig = {
  ...configuredConfig,
  isConfigured: false,
};

const mockContentScoreAnalysis: AiAnalysisRow = {
  id: 1,
  crawlId: "crawl-1",
  pageId: 1,
  analysisType: "content_score",
  provider: "openai",
  model: "gpt-4o",
  resultJson: JSON.stringify({
    overallScore: 82,
    relevanceScore: 85,
    readabilityScore: 78,
    depthScore: 83,
    reasoning: "Good content quality.",
    suggestions: ["Add more examples"],
  }),
  inputTokens: 2000,
  outputTokens: 300,
  costUsd: 0.008,
  latencyMs: 1200,
  createdAt: "2026-04-06T00:00:00Z",
};

const mockMetaDescAnalysis: AiAnalysisRow = {
  id: 2,
  crawlId: "crawl-1",
  pageId: 1,
  analysisType: "meta_desc",
  provider: "openai",
  model: "gpt-4o",
  resultJson: JSON.stringify({
    suggested: "A comprehensive test page for SEO analysis.",
    charCount: 44,
    reasoning: "Current description is too generic.",
  }),
  inputTokens: 1500,
  outputTokens: 200,
  costUsd: 0.005,
  latencyMs: 900,
  createdAt: "2026-04-06T00:00:00Z",
};

function mockInvoke(overrides: Record<string, unknown> = {}) {
  vi.mocked(invoke).mockImplementation(async (cmd: string) => {
    if (cmd in overrides) return overrides[cmd];
    switch (cmd) {
      case "get_page_detail":
        return mockPage;
      case "get_ai_config":
        return configuredConfig;
      case "get_page_analyses":
        return [];
      default:
        return undefined;
    }
  });
}

describe("AiAnalysisSection (via PageDetail)", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("shows configure message when AI is not configured", async () => {
    mockInvoke({ get_ai_config: notConfiguredConfig });
    render(<PageDetail crawlId="crawl-1" pageId={1} open={true} onClose={() => {}} />);

    await waitFor(() => {
      expect(
        screen.getByText("Configure an AI provider in Settings to enable AI analysis."),
      ).toBeTruthy();
    });
  });

  it("shows analyze button when configured with no cached analyses", async () => {
    mockInvoke();
    render(<PageDetail crawlId="crawl-1" pageId={1} open={true} onClose={() => {}} />);

    await waitFor(() => {
      expect(screen.getByText("Analyze Page")).toBeTruthy();
    });
  });

  it("calls analyzePage when button is clicked", async () => {
    mockInvoke({
      analyze_page: [mockContentScoreAnalysis],
    });
    render(<PageDetail crawlId="crawl-1" pageId={1} open={true} onClose={() => {}} />);

    await waitFor(() => {
      expect(screen.getByText("Analyze Page")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Analyze Page"));

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("analyze_page", {
        crawlId: "crawl-1",
        pageId: 1,
        analysisTypes: ["content_score", "meta_desc", "title_tag"],
      });
    });
  });

  it("displays content score results", async () => {
    mockInvoke({
      get_page_analyses: [mockContentScoreAnalysis],
    });
    render(<PageDetail crawlId="crawl-1" pageId={1} open={true} onClose={() => {}} />);

    await waitFor(() => {
      expect(screen.getByText("82/100")).toBeTruthy();
      expect(screen.getByText("Content Quality")).toBeTruthy();
    });
  });

  it("displays meta description suggestion", async () => {
    mockInvoke({
      get_page_analyses: [mockMetaDescAnalysis],
    });
    render(<PageDetail crawlId="crawl-1" pageId={1} open={true} onClose={() => {}} />);

    await waitFor(() => {
      expect(
        screen.getByText("A comprehensive test page for SEO analysis."),
      ).toBeTruthy();
    });
  });

  it("shows re-analyze link when analyses exist", async () => {
    mockInvoke({
      get_page_analyses: [mockContentScoreAnalysis],
    });
    render(<PageDetail crawlId="crawl-1" pageId={1} open={true} onClose={() => {}} />);

    await waitFor(() => {
      expect(screen.getByText("Re-analyze")).toBeTruthy();
    });
  });
});
