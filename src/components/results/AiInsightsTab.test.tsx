import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { AiInsightsTab } from "./AiInsightsTab";
import type { AiProviderConfig, AiCrawlSummaryRow, AiUsageRow } from "@/types";

const mockConfig: AiProviderConfig = {
  providerType: "open_ai",
  model: "gpt-4o",
  ollamaEndpoint: null,
  maxTokensPerCrawl: 100000,
  isConfigured: true,
};

const mockSummary: AiCrawlSummaryRow = {
  id: 1,
  crawlId: "crawl-1",
  provider: "openai",
  model: "gpt-4o",
  summaryJson: JSON.stringify({
    summary: "This site is in good health.",
    overallHealth: "good",
    topActions: ["Fix missing meta descriptions", "Add alt text"],
    criticalIssuesCount: 2,
    keyFindings: ["Most pages load fast"],
  }),
  inputTokens: 1000,
  outputTokens: 500,
  costUsd: 0.015,
  createdAt: "2026-04-06T00:00:00Z",
};

const mockUsage: AiUsageRow[] = [
  {
    id: 1,
    crawlId: "crawl-1",
    provider: "openai",
    model: "gpt-4o",
    totalInputTokens: 5000,
    totalOutputTokens: 1500,
    totalCostUsd: 0.045,
    requestCount: 10,
    updatedAt: "2026-04-06T00:00:00Z",
  },
];

function mockInvoke(overrides: Record<string, unknown> = {}) {
  vi.mocked(invoke).mockImplementation(async (cmd: string) => {
    if (cmd in overrides) return overrides[cmd];
    switch (cmd) {
      case "get_ai_config":
        return mockConfig;
      case "get_crawl_ai_summary":
        return null;
      case "get_ai_usage":
        return [];
      default:
        return undefined;
    }
  });
}

describe("AiInsightsTab", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset();
  });

  it("shows loading spinner initially", () => {
    vi.mocked(invoke).mockReturnValue(new Promise(() => {}));
    render(<AiInsightsTab crawlId="crawl-1" />);
    expect(document.querySelector(".animate-spin")).toBeTruthy();
  });

  it("shows error state on IPC failure", async () => {
    vi.mocked(invoke).mockRejectedValue("Network error");
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("Failed to load AI configuration")).toBeTruthy();
    });
  });

  it("shows not configured message when provider is not set up", async () => {
    mockInvoke({ get_ai_config: { ...mockConfig, isConfigured: false } });
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("No AI provider configured.")).toBeTruthy();
    });
  });

  it("shows generate button when no summary exists", async () => {
    mockInvoke();
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("Generate AI Summary")).toBeTruthy();
    });
  });

  it("shows summary and regenerate button when summary exists", async () => {
    mockInvoke({ get_crawl_ai_summary: mockSummary });
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("This site is in good health.")).toBeTruthy();
      expect(screen.getByText("Regenerate")).toBeTruthy();
      expect(screen.getByText(/Health: good/)).toBeTruthy();
    });
  });

  it("renders batch analysis buttons", async () => {
    mockInvoke();
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("Analyze All Pages")).toBeTruthy();
      expect(screen.getByText("Analyze Pages with Issues")).toBeTruthy();
    });
  });

  it("renders cost tracking with usage data", async () => {
    mockInvoke({ get_ai_usage: mockUsage });
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      // Cost appears in both summary row and per-model table
      expect(screen.getAllByText("$0.0450").length).toBeGreaterThanOrEqual(1);
      expect(screen.getByText("Cost Tracking")).toBeTruthy();
    });
  });

  it("shows no analysis message when usage is empty", async () => {
    mockInvoke();
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(
        screen.getByText("No AI analysis has been run for this crawl yet."),
      ).toBeTruthy();
    });
  });

  it("calls generateCrawlSummary with force when regenerate is clicked", async () => {
    mockInvoke({ get_crawl_ai_summary: mockSummary });
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("Regenerate")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Regenerate"));

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith("generate_crawl_summary", {
        crawlId: "crawl-1",
        force: true,
      });
    });
  });

  it("shows cost estimate confirmation before batch analysis", async () => {
    mockInvoke({
      estimate_batch_cost: {
        eligiblePages: 25,
        estimatedInputTokens: 150000,
        estimatedOutputTokens: 37500,
        estimatedCostUsd: 0.1875,
      },
    });
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("Analyze All Pages")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Analyze All Pages"));

    await waitFor(() => {
      expect(vi.mocked(invoke)).toHaveBeenCalledWith(
        "estimate_batch_cost",
        expect.objectContaining({ crawlId: "crawl-1" }),
      );
      expect(screen.getByText("Cost Estimate")).toBeTruthy();
      expect(screen.getByText("Start Analysis")).toBeTruthy();
      expect(screen.getByText("Cancel")).toBeTruthy();
    });
  });

  it("dismisses estimate on cancel", async () => {
    mockInvoke({
      estimate_batch_cost: {
        eligiblePages: 10,
        estimatedInputTokens: 60000,
        estimatedOutputTokens: 15000,
        estimatedCostUsd: 0.075,
      },
    });
    render(<AiInsightsTab crawlId="crawl-1" />);

    await waitFor(() => {
      expect(screen.getByText("Analyze All Pages")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Analyze All Pages"));

    await waitFor(() => {
      expect(screen.getByText("Cancel")).toBeTruthy();
    });

    fireEvent.click(screen.getByText("Cancel"));

    await waitFor(() => {
      expect(screen.queryByText("Cost Estimate")).toBeNull();
      expect(screen.getByText("Analyze All Pages")).toBeTruthy();
    });
  });
});
