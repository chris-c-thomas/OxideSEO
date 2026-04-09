/**
 * Global command palette.
 *
 * Opens via Cmd/Ctrl+K. Wraps shadcn Command inside Dialog.
 * Commands are sourced from the CommandRegistry singleton.
 */

import { useCommandPalette } from "@/hooks/useCommandPalette";
import { KeyboardHint } from "@/components/KeyboardHint";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { commandRegistry } from "@/lib/commandRegistry";

export function CommandPalette() {
  const { isOpen, setOpen } = useCommandPalette();

  const grouped = commandRegistry.getGrouped();

  const handleSelect = (commandId: string) => {
    const cmd = commandRegistry.getAll().find((c) => c.id === commandId);
    if (cmd) {
      setOpen(false);
      // Defer execution so the dialog closes before the command runs.
      requestAnimationFrame(() => cmd.run());
    }
  };

  return (
    <CommandDialog open={isOpen} onOpenChange={setOpen}>
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <CommandEmpty>No commands found.</CommandEmpty>
        {Array.from(grouped.entries()).map(([group, commands]) => (
          <CommandGroup key={group} heading={group}>
            {commands.map((cmd) => {
              const Icon = cmd.icon;
              return (
                <CommandItem
                  key={cmd.id}
                  value={cmd.id}
                  onSelect={handleSelect}
                  keywords={cmd.keywords}
                >
                  {Icon && <Icon className="text-fg-muted size-4" strokeWidth={1.75} />}
                  <span className="flex-1">{cmd.label}</span>
                  {cmd.shortcut && (
                    <KeyboardHint keys={cmd.shortcut} className="ml-auto" />
                  )}
                </CommandItem>
              );
            })}
          </CommandGroup>
        ))}
      </CommandList>
    </CommandDialog>
  );
}
