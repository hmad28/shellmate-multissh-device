import { useState, useEffect } from 'react';
import { History, Search, Trash2, X } from 'lucide-react';
import { useHistoryStore } from '@/stores/history-store';
import { cn } from '@/lib/cn';

interface CommandHistoryPanelProps {
  onClose: () => void;
  onRunCommand?: (command: string) => void;
}

export function CommandHistoryPanel({ onClose, onRunCommand }: CommandHistoryPanelProps) {
  const { entries, loading, load, search, clear } = useHistoryStore();
  const [query, setQuery] = useState('');

  useEffect(() => {
    void load();
  }, [load]);

  useEffect(() => {
    const timer = setTimeout(() => {
      if (query.trim()) {
        void search(query);
      } else {
        void load();
      }
    }, 300);
    return () => clearTimeout(timer);
  }, [query, load, search]);

  return (
    <div className="flex h-full flex-col bg-bg">
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <div className="flex items-center gap-2">
          <History size={16} className="text-accent" />
          <h2 className="text-sm font-semibold text-fg">Command History</h2>
          <span className="text-xs text-fg-muted">({entries.length})</span>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={() => void clear()}
            className="text-fg-muted transition-colors hover:text-red-400"
            title="Clear all history"
          >
            <Trash2 size={14} />
          </button>
          <button
            onClick={onClose}
            className="text-fg-muted transition-colors hover:text-fg"
          >
            <X size={16} />
          </button>
        </div>
      </div>

      <div className="border-b border-border px-4 py-2">
        <div className="flex items-center gap-2 rounded-md border border-border-subtle bg-bg-elevated px-3 py-1.5">
          <Search size={14} className="text-fg-muted" />
          <input
            type="text"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search commands..."
            className="flex-1 bg-transparent text-sm text-fg placeholder:text-fg-muted focus:outline-none"
          />
        </div>
      </div>

      <div className="flex-1 overflow-auto">
        {loading ? (
          <div className="flex h-32 items-center justify-center text-sm text-fg-muted">
            Loading...
          </div>
        ) : entries.length === 0 ? (
          <div className="flex h-32 flex-col items-center justify-center gap-2">
            <History size={20} className="text-fg-subtle" />
            <span className="text-sm text-fg-muted">
              {query ? 'No matching commands' : 'No command history yet'}
            </span>
          </div>
        ) : (
          <div className="p-2">
            {entries.map((entry) => (
              <div
                key={entry.id}
                className={cn(
                  'group flex items-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors',
                  'hover:bg-bg-elevated cursor-pointer',
                )}
                onClick={() => onRunCommand?.(entry.command)}
              >
                <span
                  className={cn(
                    'flex-1 truncate font-mono text-xs',
                    entry.exitCode === 0
                      ? 'text-fg'
                      : entry.exitCode != null
                        ? 'text-red-400'
                        : 'text-fg-muted',
                  )}
                >
                  {entry.command}
                </span>
                {entry.exitCode != null && (
                  <span
                    className={cn(
                      'text-[10px]',
                      entry.exitCode === 0 ? 'text-fg-subtle' : 'text-red-400',
                    )}
                  >
                    [{entry.exitCode}]
                  </span>
                )}
                <span className="text-[10px] text-fg-subtle">
                  {formatTime(entry.executedAt)}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

function formatTime(iso: string): string {
  try {
    const d = new Date(iso);
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    if (diffMins < 1) return 'now';
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    return d.toLocaleDateString();
  } catch {
    return '';
  }
}
