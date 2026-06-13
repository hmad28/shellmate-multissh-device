import { useEffect, useRef, useCallback } from 'react';
import { Search, Command as CommandIcon } from 'lucide-react';
import { useCommandPaletteStore, filterCommands } from '@/stores/command-store';
import { cn } from '@/lib/cn';

export function CommandPalette() {
  const {
    isOpen,
    query,
    selectedIndex,
    commands,
    close,
    setQuery,
    setSelectedIndex,
    executeSelected,
  } = useCommandPaletteStore();

  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const filtered = filterCommands(commands, query);

  useEffect(() => {
    if (isOpen) {
      inputRef.current?.focus();
    }
  }, [isOpen]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        const store = useCommandPaletteStore.getState();
        if (store.isOpen) {
          store.close();
        } else {
          store.open();
        }
      }
      if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'P') {
        e.preventDefault();
        useCommandPaletteStore.getState().open();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case 'Escape':
          e.preventDefault();
          close();
          break;
        case 'ArrowDown':
          e.preventDefault();
          setSelectedIndex(Math.min(selectedIndex + 1, filtered.length - 1));
          break;
        case 'ArrowUp':
          e.preventDefault();
          setSelectedIndex(Math.max(selectedIndex - 1, 0));
          break;
        case 'Enter':
          e.preventDefault();
          executeSelected();
          break;
      }
    },
    [close, setSelectedIndex, executeSelected, selectedIndex, filtered.length],
  );

  useEffect(() => {
    const el = listRef.current?.children[selectedIndex];
    el?.scrollIntoView({ block: 'nearest' });
  }, [selectedIndex]);

  if (!isOpen) return null;

  // Group commands by category
  const grouped = new Map<string, typeof filtered>();
  for (const cmd of filtered) {
    const list = grouped.get(cmd.category) ?? [];
    list.push(cmd);
    grouped.set(cmd.category, list);
  }

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/50 pt-[20vh]"
      onClick={close}
    >
      <div
        className="w-full max-w-lg overflow-hidden rounded-lg border border-border bg-bg shadow-2xl"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={handleKeyDown}
      >
        {/* Search input */}
        <div className="flex items-center gap-2 border-b border-border px-4 py-3">
          <Search className="h-4 w-4 text-fg-muted" />
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Type a command..."
            className="flex-1 bg-transparent text-sm text-fg placeholder:text-fg-muted focus:outline-none"
          />
          <kbd className="rounded border border-border px-1.5 py-0.5 text-[10px] text-fg-subtle">
            ESC
          </kbd>
        </div>

        {/* Command list */}
        <div ref={listRef} className="max-h-72 overflow-y-auto p-1">
          {filtered.length === 0 ? (
            <div className="flex flex-col items-center gap-2 py-8 text-sm text-fg-muted">
              <CommandIcon className="h-5 w-5" />
              No commands found
            </div>
          ) : (
            Array.from(grouped.entries()).map(([category, cmds]) => (
              <div key={category}>
                <div className="px-3 py-1.5 text-[10px] font-medium uppercase tracking-wider text-fg-subtle">
                  {category}
                </div>
                {cmds.map((cmd) => {
                  const idx = filtered.indexOf(cmd);
                  return (
                    <button
                      key={cmd.id}
                      className={cn(
                        'flex w-full items-center justify-between rounded-md px-3 py-2 text-left text-sm transition-colors',
                        idx === selectedIndex
                          ? 'bg-accent/10 text-fg'
                          : 'text-fg-muted hover:bg-bg-elevated hover:text-fg',
                      )}
                      onClick={() => {
                        close();
                        cmd.action();
                      }}
                      onMouseEnter={() => setSelectedIndex(idx)}
                    >
                      <span>{cmd.label}</span>
                      {cmd.shortcut && (
                        <kbd className="ml-2 rounded border border-border px-1.5 py-0.5 text-[10px] text-fg-subtle">
                          {cmd.shortcut}
                        </kbd>
                      )}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
