/**
 * Sidebar navigation with view switching and theme toggle.
 */

import type { AppView } from "@/App";
import { cn } from "@/lib/utils";
import { useCrawlStore } from "@/stores/crawlStore";

interface SidebarProps {
  activeView: AppView;
  onNavigate: (view: AppView) => void;
  theme: "light" | "dark";
  onToggleTheme: () => void;
}

interface NavItem {
  id: AppView;
  label: string;
  icon: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: "grid" },
  { id: "crawl-config", label: "New Crawl", icon: "plus-circle" },
  { id: "crawl-monitor", label: "Monitor", icon: "activity" },
  { id: "results", label: "Results", icon: "table" },
  { id: "settings", label: "Settings", icon: "settings" },
];

export function Sidebar({ activeView, onNavigate, theme, onToggleTheme }: SidebarProps) {
  const crawlState = useCrawlStore((s) => s.state);

  return (
    <aside
      className="flex h-full flex-col border-r"
      style={{
        width: "var(--sidebar-width)",
        backgroundColor: "var(--color-sidebar)",
        borderColor: "var(--color-sidebar-border)",
        color: "var(--color-sidebar-foreground)",
      }}
    >
      {/* Logo / App Name */}
      <div
        className="flex items-center gap-2 border-b px-4"
        style={{
          height: "var(--header-height)",
          borderColor: "var(--color-sidebar-border)",
        }}
      >
        <span className="text-base font-bold tracking-tight">OxideSEO</span>
        <span className="text-xs" style={{ color: "var(--color-muted-foreground)" }}>
          v0.1
        </span>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 p-2">
        {NAV_ITEMS.map((item) => {
          const isActive = activeView === item.id;
          const showDot = item.id === "crawl-monitor" && crawlState === "running";

          return (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={cn(
                "flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors",
                isActive && "font-medium",
              )}
              style={{
                backgroundColor: isActive ? "var(--color-sidebar-active)" : "transparent",
              }}
            >
              <span className="flex-1 text-left">{item.label}</span>
              {showDot && (
                <span
                  className="h-2 w-2 rounded-full"
                  style={{ backgroundColor: "var(--color-status-running)" }}
                />
              )}
            </button>
          );
        })}
      </nav>

      {/* Theme toggle */}
      <div
        className="border-t p-2"
        style={{ borderColor: "var(--color-sidebar-border)" }}
      >
        <button
          onClick={onToggleTheme}
          className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm transition-colors"
        >
          <span>{theme === "dark" ? "Light Mode" : "Dark Mode"}</span>
        </button>
      </div>
    </aside>
  );
}
