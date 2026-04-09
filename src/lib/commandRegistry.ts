/**
 * Command palette registry.
 *
 * Stores commands that can be searched and executed via the
 * command palette (Cmd/Ctrl+K). Views register their commands
 * on mount and unregister on unmount.
 *
 * This is separate from lib/commands.ts which wraps Tauri IPC calls.
 */

import type { LucideIcon } from "lucide-react";

export interface PaletteCommand {
  id: string;
  label: string;
  shortcut?: string[];
  icon?: LucideIcon;
  group: string;
  run: () => void;
  keywords?: string[];
}

class CommandRegistry {
  private commands = new Map<string, PaletteCommand>();
  private listeners = new Set<() => void>();

  register(command: PaletteCommand) {
    this.commands.set(command.id, command);
    this.notify();
  }

  registerMany(commands: PaletteCommand[]) {
    for (const cmd of commands) {
      this.commands.set(cmd.id, cmd);
    }
    this.notify();
  }

  unregister(id: string) {
    this.commands.delete(id);
    this.notify();
  }

  unregisterMany(ids: string[]) {
    for (const id of ids) {
      this.commands.delete(id);
    }
    this.notify();
  }

  getAll(): PaletteCommand[] {
    return Array.from(this.commands.values());
  }

  getGrouped(): Map<string, PaletteCommand[]> {
    const groups = new Map<string, PaletteCommand[]>();
    for (const cmd of this.commands.values()) {
      const group = groups.get(cmd.group) ?? [];
      group.push(cmd);
      groups.set(cmd.group, group);
    }
    return groups;
  }

  subscribe(listener: () => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  private notify() {
    for (const listener of this.listeners) {
      listener();
    }
  }
}

export const commandRegistry = new CommandRegistry();
