/**
 * Root application shell layout.
 *
 * CSS Grid layout with TitleBar (full width), Sidebar + Main (middle row),
 * and StatusBar (full width). Wraps all view content.
 */

import type { ReactNode } from "react";
import type { AppView } from "@/App";
import { TitleBar } from "@/components/TitleBar";
import { Sidebar } from "@/components/layout/Sidebar";
import { StatusBar } from "@/components/StatusBar";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/sonner";

interface AppShellProps {
  activeView: AppView;
  onNavigate: (view: AppView) => void;
  children: ReactNode;
}

export function AppShell({ activeView, onNavigate, children }: AppShellProps) {
  return (
    <TooltipProvider delayDuration={200}>
      <div className="bg-bg-app text-fg-default grid h-screen w-screen grid-rows-[auto_1fr_auto] overflow-hidden">
        {/* Row 1: Title bar */}
        <TitleBar />

        {/* Row 2: Sidebar + Main content */}
        <div className="flex min-h-0 overflow-hidden">
          <Sidebar activeView={activeView} onNavigate={onNavigate} />
          <main className="flex-1 overflow-auto">{children}</main>
        </div>

        {/* Row 3: Status bar */}
        <StatusBar />

        <Toaster position="bottom-right" />
      </div>
    </TooltipProvider>
  );
}
