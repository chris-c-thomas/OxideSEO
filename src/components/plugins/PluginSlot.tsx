/**
 * Placeholder component for plugin-contributed UI extensions.
 *
 * Phase 8 scope: only "results-tab" slot. Full implementation will use
 * React.lazy + Suspense to dynamically import plugin frontend modules.
 */

import type { PluginExtension } from "@/hooks/usePluginExtensions";

interface PluginSlotProps {
  extension: PluginExtension;
}

export function PluginSlot({ extension }: PluginSlotProps) {
  return (
    <div className="flex h-full items-center justify-center text-[var(--color-muted-foreground)]">
      <p className="text-sm">
        Plugin UI extension: {extension.pluginName} ({extension.label})
      </p>
    </div>
  );
}
