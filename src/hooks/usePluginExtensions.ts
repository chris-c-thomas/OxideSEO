/**
 * Hook to get plugin-contributed UI extensions for a given slot.
 *
 * Phase 8 scope: only "results-tab" slot. Returns empty array when
 * no plugins contribute to the requested slot.
 */

export interface PluginExtension {
  pluginName: string;
  slotId: string;
  label: string;
}

/**
 * Returns plugin extensions registered for the given slot ID.
 *
 * Currently a stub that returns an empty array. Will be populated
 * when UiExtension plugins are installed and enabled.
 */
export function usePluginExtensions(_slotId: string): PluginExtension[] {
  // Phase 8 stub — full implementation will query the PluginManager
  // for enabled UiExtension plugins that contribute to this slot.
  return [];
}
