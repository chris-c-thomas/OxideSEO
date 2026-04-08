import { useState } from "react";
import { useTheme } from "@/hooks/useTheme";
import { Sidebar } from "@/components/layout/Sidebar";
import { Dashboard } from "@/components/layout/Dashboard";
import { CrawlConfig } from "@/components/crawl/CrawlConfig";
import { CrawlMonitor } from "@/components/crawl/CrawlMonitor";
import { ResultsExplorer } from "@/components/results/ResultsExplorer";
import { SettingsView } from "@/components/settings/SettingsView";
import { PluginManagerView } from "@/components/plugins/PluginManagerView";
import { CrawlComparison } from "@/components/comparison/CrawlComparison";

/** Application views mapped to sidebar navigation items. */
export type AppView =
  | "dashboard"
  | "crawl-config"
  | "crawl-monitor"
  | "results"
  | "plugins"
  | "settings"
  | "crawl-comparison";

export function App() {
  const [activeView, setActiveView] = useState<AppView>("dashboard");
  const [activeCrawlId, setActiveCrawlId] = useState<string | null>(null);
  const [compareCrawlId, setCompareCrawlId] = useState<string | null>(null);
  const { theme, setTheme } = useTheme();

  /** Navigate to a view. Optionally set crawl context (1 or 2 IDs for comparison). */
  const navigate = (view: AppView, crawlId?: string, secondCrawlId?: string) => {
    setActiveView(view);
    if (crawlId !== undefined) {
      setActiveCrawlId(crawlId);
    }
    if (secondCrawlId !== undefined) {
      setCompareCrawlId(secondCrawlId);
    }
  };

  const renderView = () => {
    switch (activeView) {
      case "dashboard":
        return <Dashboard onNavigate={navigate} />;
      case "crawl-config":
        return (
          <CrawlConfig
            onCrawlStarted={(crawlId) => {
              setActiveCrawlId(crawlId);
              setActiveView("crawl-monitor");
            }}
          />
        );
      case "crawl-monitor":
        return (
          <CrawlMonitor
            crawlId={activeCrawlId}
            onCompleted={() => setActiveView("results")}
          />
        );
      case "results":
        return <ResultsExplorer crawlId={activeCrawlId} />;
      case "plugins":
        return <PluginManagerView />;
      case "settings":
        return <SettingsView />;
      case "crawl-comparison":
        return (
          <CrawlComparison baseCrawlId={activeCrawlId} compareCrawlId={compareCrawlId} />
        );
      default:
        return <Dashboard onNavigate={navigate} />;
    }
  };

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-[var(--color-background)] text-[var(--color-foreground)]">
      <Sidebar
        activeView={activeView}
        onNavigate={navigate}
        theme={theme}
        onToggleTheme={() => setTheme(theme === "dark" ? "light" : "dark")}
      />
      <main className="flex-1 overflow-auto">{renderView()}</main>
    </div>
  );
}
