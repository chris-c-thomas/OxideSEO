import { useEffect, useState } from "react";
import { useTheme } from "@/hooks/useTheme";
import { useHotkeys } from "@/hooks/useHotkeys";
import { useCrawlProgress } from "@/hooks/useCrawlProgress";
import { useCrawlStateEvents } from "@/hooks/useCrawlStateEvents";
import { toggleCommandPalette } from "@/hooks/useCommandPalette";
import { commandRegistry } from "@/lib/commandRegistry";
import { pauseCrawl, resumeCrawl, stopCrawl } from "@/lib/commands";
import { SHORTCUTS, shortcutToKeys } from "@/lib/shortcuts";
import { useCrawlStore } from "@/stores/crawlStore";
import { AppShell } from "@/components/AppShell";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { CommandPalette } from "@/components/CommandPalette";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { Dashboard } from "@/features/dashboard/Dashboard";
import { CrawlConfig } from "@/features/crawl-config/CrawlConfig";
import { CrawlMonitor } from "@/features/crawl-monitor/CrawlMonitor";
import { ResultsExplorer } from "@/features/results-explorer/ResultsExplorer";
import { SettingsView } from "@/features/settings/SettingsView";
import { IssuesView } from "@/features/issues/IssuesView";
import { PluginManagerView } from "@/components/plugins/PluginManagerView";
import { CrawlComparison } from "@/components/comparison/CrawlComparison";
import { toast } from "sonner";
import {
  LayoutDashboard,
  PlusCircle,
  Activity,
  Table2,
  Settings,
  Palette,
  Pause,
  Play,
  Square,
} from "lucide-react";

/** Application views mapped to sidebar navigation items. */
export type AppView =
  | "dashboard"
  | "crawl-config"
  | "crawl-monitor"
  | "results"
  | "issues"
  | "plugins"
  | "settings"
  | "crawl-comparison";

export function App() {
  const [activeView, setActiveView] = useState<AppView>("dashboard");
  const [activeCrawlId, setActiveCrawlId] = useState<string | null>(null);
  const [compareCrawlId, setCompareCrawlId] = useState<string | null>(null);
  const [showStopConfirm, setShowStopConfirm] = useState(false);

  // Initialize theme system (applies data-theme attribute).
  const { setTheme, resolved } = useTheme();
  const crawlState = useCrawlStore((s) => s.state);
  const setCrawlState = useCrawlStore((s) => s.setCrawlState);

  // App-level crawl event listeners — active regardless of current view.
  useCrawlProgress(activeCrawlId);
  useCrawlStateEvents();

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

  // Register global shortcuts.
  useHotkeys({
    commandPalette: { shortcut: SHORTCUTS.commandPalette, handler: toggleCommandPalette },
    settings: { shortcut: SHORTCUTS.settings, handler: () => navigate("settings") },
    newCrawl: { shortcut: SHORTCUTS.newCrawl, handler: () => navigate("crawl-config") },
    switchView1: {
      shortcut: SHORTCUTS.switchView1,
      handler: () => navigate("dashboard"),
    },
    switchView2: {
      shortcut: SHORTCUTS.switchView2,
      handler: () => navigate("crawl-config"),
    },
    switchView3: {
      shortcut: SHORTCUTS.switchView3,
      handler: () => navigate("crawl-monitor"),
    },
    switchView4: { shortcut: SHORTCUTS.switchView4, handler: () => navigate("results") },
    switchView5: { shortcut: SHORTCUTS.switchView5, handler: () => navigate("settings") },
  });

  // Register core commands in the palette.
  useEffect(() => {
    const commands = [
      {
        id: "nav:dashboard",
        label: "Go to Dashboard",
        icon: LayoutDashboard,
        group: "Navigation",
        shortcut: shortcutToKeys(SHORTCUTS.switchView1),
        run: () => navigate("dashboard"),
      },
      {
        id: "nav:new-crawl",
        label: "Start New Crawl",
        icon: PlusCircle,
        group: "Navigation",
        shortcut: shortcutToKeys(SHORTCUTS.newCrawl),
        run: () => navigate("crawl-config"),
      },
      {
        id: "nav:monitor",
        label: "Go to Monitor",
        icon: Activity,
        group: "Navigation",
        run: () => navigate("crawl-monitor"),
      },
      {
        id: "nav:results",
        label: "Go to Results",
        icon: Table2,
        group: "Navigation",
        run: () => navigate("results"),
      },
      {
        id: "nav:settings",
        label: "Go to Settings",
        icon: Settings,
        group: "Navigation",
        shortcut: shortcutToKeys(SHORTCUTS.settings),
        run: () => navigate("settings"),
      },
      {
        id: "theme:toggle",
        label: "Toggle Theme",
        icon: Palette,
        group: "Appearance",
        keywords: ["dark", "light", "mode"],
        run: () => setTheme(resolved === "dark" ? "light" : "dark"),
      },
    ];

    // Conditionally add crawl lifecycle commands based on active state.
    if (crawlState === "running" && activeCrawlId) {
      commands.push({
        id: "crawl:pause",
        label: "Pause Active Crawl",
        icon: Pause,
        group: "Crawl",
        run: async () => {
          try {
            await pauseCrawl(activeCrawlId);
            setCrawlState("paused");
          } catch (err) {
            toast.error(`Failed to pause crawl: ${String(err)}`);
          }
        },
      });
    }
    if (crawlState === "paused" && activeCrawlId) {
      commands.push({
        id: "crawl:resume",
        label: "Resume Active Crawl",
        icon: Play,
        group: "Crawl",
        run: async () => {
          try {
            await resumeCrawl(activeCrawlId);
            setCrawlState("running");
          } catch (err) {
            toast.error(`Failed to resume crawl: ${String(err)}`);
          }
        },
      });
    }
    if ((crawlState === "running" || crawlState === "paused") && activeCrawlId) {
      commands.push({
        id: "crawl:stop",
        label: "Stop Active Crawl",
        icon: Square,
        group: "Crawl",
        run: () => setShowStopConfirm(true),
      });
    }

    commandRegistry.registerMany(commands);
    return () => commandRegistry.unregisterMany(commands.map((c) => c.id));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [resolved, crawlState, activeCrawlId]);

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
            onCancel={() => setActiveView("dashboard")}
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
      case "issues":
        return <IssuesView crawlId={activeCrawlId} />;
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

  const handleConfirmStop = async () => {
    setShowStopConfirm(false);
    if (!activeCrawlId) return;
    try {
      await stopCrawl(activeCrawlId);
      setCrawlState("stopped");
    } catch (err) {
      toast.error(`Failed to stop crawl: ${String(err)}`);
    }
  };

  return (
    <AppShell activeView={activeView} onNavigate={navigate}>
      <ErrorBoundary>{renderView()}</ErrorBoundary>
      <CommandPalette />
      <ConfirmDialog
        open={showStopConfirm}
        title="Stop Crawl"
        description="This will stop the crawl. In-flight requests will complete but no new URLs will be fetched."
        confirmLabel="Stop"
        variant="destructive"
        onConfirm={handleConfirmStop}
        onCancel={() => setShowStopConfirm(false)}
      />
    </AppShell>
  );
}
