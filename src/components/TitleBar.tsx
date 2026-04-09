/**
 * App title/branding bar.
 *
 * Sits below the native OS title bar as a branded strip with
 * drag region support, app name, and global actions slot.
 * Native decorations remain enabled in tauri.conf.json.
 */

import type { ReactNode } from "react";

const IS_MAC =
  typeof navigator !== "undefined" && /Mac|iPhone|iPad/.test(navigator.userAgent);

interface TitleBarProps {
  actions?: ReactNode;
}

export function TitleBar({ actions }: TitleBarProps) {
  return (
    <div
      data-tauri-drag-region
      className="border-border-subtle bg-bg-app flex shrink-0 items-center border-b"
      style={{ height: IS_MAC ? 36 : 32 }}
    >
      {/* Left zone: app branding */}
      <div className="flex items-center gap-2 px-4" data-tauri-drag-region>
        <span
          className="text-fg-default text-sm font-bold tracking-tight"
          data-tauri-drag-region
        >
          OxideSEO
        </span>
        <span className="text-fg-subtle text-[0.6875rem]">v0.4</span>
      </div>

      {/* Spacer with drag region */}
      <div className="flex-1" data-tauri-drag-region />

      {/* Right zone: global actions */}
      {actions && <div className="flex items-center gap-1 px-2">{actions}</div>}
    </div>
  );
}
