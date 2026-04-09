/**
 * Sidebar navigation with collapsible width, Lucide icons,
 * active state highlighting, and theme toggle.
 */

import type { AppView } from "@/App";
import { cn } from "@/lib/utils";
import { useCrawlStore } from "@/stores/crawlStore";
import { useUiStore } from "@/stores/uiStore";
import { useTheme } from "@/hooks/useTheme";
import {
  LayoutDashboard,
  PlusCircle,
  Activity,
  Table2,
  AlertTriangle,
  Puzzle,
  Settings,
  PanelLeftClose,
  PanelLeft,
  Sun,
  Moon,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

interface SidebarProps {
  activeView: AppView;
  onNavigate: (view: AppView) => void;
}

interface NavItem {
  id: AppView;
  label: string;
  icon: LucideIcon;
}

const NAV_ITEMS: NavItem[] = [
  { id: "dashboard", label: "Dashboard", icon: LayoutDashboard },
  { id: "crawl-config", label: "New Crawl", icon: PlusCircle },
  { id: "crawl-monitor", label: "Monitor", icon: Activity },
  { id: "results", label: "Results", icon: Table2 },
  { id: "issues", label: "Issues", icon: AlertTriangle },
  { id: "plugins", label: "Plugins", icon: Puzzle },
  { id: "settings", label: "Settings", icon: Settings },
];

const STATUS_DOT_COLORS: Record<string, string> = {
  running: "bg-status-running",
  paused: "bg-status-paused",
  error: "bg-status-error",
};

export function Sidebar({ activeView, onNavigate }: SidebarProps) {
  const crawlState = useCrawlStore((s) => s.state);
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);
  const { resolved, setTheme } = useTheme();

  const toggleTheme = () => {
    setTheme(resolved === "dark" ? "light" : "dark");
  };

  return (
    <aside
      className={cn(
        "border-border-subtle bg-bg-subtle flex h-full flex-col border-r transition-[width] duration-200",
        collapsed ? "w-14" : "w-56",
      )}
    >
      {/* Navigation */}
      <nav className="flex flex-1 flex-col gap-1 p-2">
        {NAV_ITEMS.map((item) => {
          const isActive = activeView === item.id;
          const dotColor =
            item.id === "crawl-monitor" && crawlState
              ? (STATUS_DOT_COLORS[crawlState] ?? null)
              : null;
          const showDot = dotColor !== null;
          const Icon = item.icon;

          const button = (
            <button
              key={item.id}
              onClick={() => onNavigate(item.id)}
              className={cn(
                "flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-1.5 text-sm transition-colors",
                isActive
                  ? "bg-bg-active text-fg-default font-medium"
                  : "text-fg-muted hover:bg-bg-hover hover:text-fg-default",
                collapsed && "justify-center px-0",
              )}
            >
              <Icon className="size-4 shrink-0" strokeWidth={1.75} />
              {!collapsed && <span className="flex-1 text-left">{item.label}</span>}
              {!collapsed && showDot && (
                <span
                  className={cn(
                    "size-2 rounded-full",
                    dotColor,
                    crawlState === "running" && "animate-pulse",
                  )}
                />
              )}
            </button>
          );

          if (collapsed) {
            return (
              <Tooltip key={item.id}>
                <TooltipTrigger asChild>{button}</TooltipTrigger>
                <TooltipContent side="right" sideOffset={8}>
                  {item.label}
                </TooltipContent>
              </Tooltip>
            );
          }

          return button;
        })}
      </nav>

      {/* Bottom: theme toggle + collapse */}
      <div className="border-border-subtle flex flex-col gap-1 border-t p-2">
        {/* Theme toggle */}
        {collapsed ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={toggleTheme}
                className="text-fg-muted hover:bg-bg-hover hover:text-fg-default flex w-full items-center justify-center rounded-[var(--radius-sm)] px-3 py-1.5 text-sm transition-colors"
                aria-label={
                  resolved === "dark" ? "Switch to light mode" : "Switch to dark mode"
                }
              >
                {resolved === "dark" ? (
                  <Sun className="size-4" strokeWidth={1.75} />
                ) : (
                  <Moon className="size-4" strokeWidth={1.75} />
                )}
              </button>
            </TooltipTrigger>
            <TooltipContent side="right" sideOffset={8}>
              {resolved === "dark" ? "Light mode" : "Dark mode"}
            </TooltipContent>
          </Tooltip>
        ) : (
          <button
            onClick={toggleTheme}
            className="text-fg-muted hover:bg-bg-hover hover:text-fg-default flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-1.5 text-sm transition-colors"
          >
            {resolved === "dark" ? (
              <Sun className="size-4" strokeWidth={1.75} />
            ) : (
              <Moon className="size-4" strokeWidth={1.75} />
            )}
            <span>{resolved === "dark" ? "Light Mode" : "Dark Mode"}</span>
          </button>
        )}

        {/* Collapse toggle */}
        {collapsed ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={toggleSidebar}
                className="text-fg-muted hover:bg-bg-hover hover:text-fg-default flex w-full items-center justify-center rounded-[var(--radius-sm)] px-3 py-1.5 text-sm transition-colors"
                aria-label="Expand sidebar"
              >
                <PanelLeft className="size-4" strokeWidth={1.75} />
              </button>
            </TooltipTrigger>
            <TooltipContent side="right" sideOffset={8}>
              Expand sidebar
            </TooltipContent>
          </Tooltip>
        ) : (
          <button
            onClick={toggleSidebar}
            className="text-fg-muted hover:bg-bg-hover hover:text-fg-default flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-1.5 text-sm transition-colors"
            aria-label="Collapse sidebar"
          >
            <PanelLeftClose className="size-4" strokeWidth={1.75} />
            <span>Collapse</span>
          </button>
        )}
      </div>
    </aside>
  );
}
