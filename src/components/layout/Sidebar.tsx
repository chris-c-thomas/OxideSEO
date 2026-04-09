/**
 * Sidebar navigation with collapsible width, Lucide icons, and
 * active state highlighting. Theme toggle removed (moved to Settings).
 */

import type { AppView } from "@/App";
import { cn } from "@/lib/utils";
import { useCrawlStore } from "@/stores/crawlStore";
import { useUiStore } from "@/stores/uiStore";
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

export function Sidebar({ activeView, onNavigate }: SidebarProps) {
  const crawlState = useCrawlStore((s) => s.state);
  const collapsed = useUiStore((s) => s.sidebarCollapsed);
  const toggleSidebar = useUiStore((s) => s.toggleSidebar);

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
          const showDot = item.id === "crawl-monitor" && crawlState === "running";
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
                <span className="bg-status-running size-2 rounded-full" />
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

      {/* Collapse toggle */}
      <div className="border-border-subtle border-t p-2">
        <button
          onClick={toggleSidebar}
          className={cn(
            "text-fg-muted hover:bg-bg-hover hover:text-fg-default flex w-full items-center gap-3 rounded-[var(--radius-sm)] px-3 py-1.5 text-sm transition-colors",
            collapsed && "justify-center px-0",
          )}
          aria-label={collapsed ? "Expand sidebar" : "Collapse sidebar"}
        >
          {collapsed ? (
            <PanelLeft className="size-4" strokeWidth={1.75} />
          ) : (
            <>
              <PanelLeftClose className="size-4" strokeWidth={1.75} />
              <span>Collapse</span>
            </>
          )}
        </button>
      </div>
    </aside>
  );
}
