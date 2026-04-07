/**
 * Plugin manager view — lists installed plugins with enable/disable toggles,
 * install/reload buttons, and a detail sheet.
 */

import { useCallback, useEffect, useState } from "react";
import type { PluginDetail, PluginInfo } from "@/types";
import {
  listPlugins,
  enablePlugin,
  disablePlugin,
  getPluginDetail,
  reloadPlugins,
  installPluginFromFile,
  uninstallPlugin,
} from "@/lib/commands";
import { PluginCard } from "./PluginCard";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Sheet, SheetContent, SheetHeader, SheetTitle } from "@/components/ui/sheet";

export function PluginManagerView() {
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedPlugin, setSelectedPlugin] = useState<string | null>(null);
  const [detail, setDetail] = useState<PluginDetail | null>(null);

  const fetchPlugins = useCallback(async () => {
    try {
      const list = await listPlugins();
      setPlugins(list);
      setError(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchPlugins();
  }, [fetchPlugins]);

  const handleToggle = async (name: string, enabled: boolean) => {
    setError(null);
    try {
      if (enabled) {
        await enablePlugin(name);
      } else {
        await disablePlugin(name);
      }
      await fetchPlugins();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleInstall = async () => {
    setError(null);
    try {
      await installPluginFromFile();
      await fetchPlugins();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleReload = async () => {
    setError(null);
    try {
      const list = await reloadPlugins();
      setPlugins(list);
    } catch (e) {
      setError(String(e));
    }
  };

  const handleUninstall = async (name: string) => {
    setError(null);
    try {
      await uninstallPlugin(name);
      setSelectedPlugin(null);
      setDetail(null);
      await fetchPlugins();
    } catch (e) {
      setError(String(e));
    }
  };

  const handleCardClick = async (name: string) => {
    setError(null);
    setSelectedPlugin(name);
    try {
      const d = await getPluginDetail(name);
      setDetail(d);
    } catch (e) {
      setError(String(e));
    }
  };

  if (loading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div
          role="status"
          aria-label="Loading"
          className="h-8 w-8 animate-spin rounded-full border-4 border-[var(--color-primary)] border-t-transparent"
        />
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col gap-4 p-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-xl font-bold">Plugins</h1>
          <p className="text-sm text-[var(--color-muted-foreground)]">
            Extend OxideSEO with custom rules, exporters, and post-processors.
          </p>
        </div>
        <div className="flex gap-2">
          <Button variant="outline" onClick={handleReload}>
            Reload
          </Button>
          <Button onClick={handleInstall}>Install Plugin</Button>
        </div>
      </div>

      {error && (
        <div className="rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-800 dark:border-red-800 dark:bg-red-950 dark:text-red-200">
          {error}
        </div>
      )}

      {plugins.length === 0 ? (
        <div className="flex flex-1 flex-col items-center justify-center gap-2 text-[var(--color-muted-foreground)]">
          <p className="text-lg">No plugins installed</p>
          <p className="text-sm">
            Click &quot;Install Plugin&quot; to add your first plugin, or place plugin
            directories in the plugins folder.
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {plugins.map((p) => (
            <PluginCard
              key={p.name}
              plugin={p}
              onToggle={handleToggle}
              onClick={handleCardClick}
            />
          ))}
        </div>
      )}

      {/* Plugin detail sheet */}
      <Sheet
        open={selectedPlugin !== null}
        onOpenChange={(open) => {
          if (!open) {
            setSelectedPlugin(null);
            setDetail(null);
          }
        }}
      >
        <SheetContent className="w-[400px] overflow-y-auto sm:max-w-[400px]">
          <SheetHeader>
            <SheetTitle>{detail?.name ?? "Plugin Details"}</SheetTitle>
          </SheetHeader>

          {detail && (
            <div className="mt-4 flex flex-col gap-4">
              <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-[var(--color-muted-foreground)]">
                  Version
                </span>
                <span className="text-sm">{detail.version}</span>
              </div>

              <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-[var(--color-muted-foreground)]">
                  Description
                </span>
                <span className="text-sm">{detail.description}</span>
              </div>

              {detail.author && (
                <div className="flex flex-col gap-1">
                  <span className="text-xs font-medium text-[var(--color-muted-foreground)]">
                    Author
                  </span>
                  <span className="text-sm">{detail.author}</span>
                </div>
              )}

              {detail.license && (
                <div className="flex flex-col gap-1">
                  <span className="text-xs font-medium text-[var(--color-muted-foreground)]">
                    License
                  </span>
                  <span className="text-sm">{detail.license}</span>
                </div>
              )}

              <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-[var(--color-muted-foreground)]">
                  Kind
                </span>
                <span className="text-sm capitalize">{detail.kind}</span>
              </div>

              {detail.capabilities.length > 0 && (
                <div className="flex flex-col gap-1">
                  <span className="text-xs font-medium text-[var(--color-muted-foreground)]">
                    Capabilities
                  </span>
                  <div className="flex flex-wrap gap-1">
                    {detail.capabilities.map((cap) => (
                      <Badge key={cap} variant="outline" className="text-xs">
                        {cap}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {detail.isNative && (
                <div className="rounded-md border border-yellow-300 bg-yellow-50 p-3 text-xs text-yellow-800 dark:border-yellow-800 dark:bg-yellow-950 dark:text-yellow-200">
                  This is a native plugin that executes arbitrary code. Only install
                  native plugins from trusted sources.
                </div>
              )}

              <div className="flex flex-col gap-1">
                <span className="text-xs font-medium text-[var(--color-muted-foreground)]">
                  Installed
                </span>
                <span className="text-sm">{detail.installedAt}</span>
              </div>

              <div className="mt-4 border-t pt-4">
                <Button
                  variant="destructive"
                  className="w-full"
                  onClick={() => handleUninstall(detail.name)}
                >
                  Uninstall Plugin
                </Button>
              </div>
            </div>
          )}
        </SheetContent>
      </Sheet>
    </div>
  );
}
