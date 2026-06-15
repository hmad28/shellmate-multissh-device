import { useState, useEffect } from 'react';
import { useSnippetStore } from '@/stores/snippet-store';

interface MobileKeyBarProps {
  onSend: (data: string) => void;
}

interface KeyItem {
  label: string;
  key?: string;
  modifier?: string;
  action?: string;
}

const KEY_ROWS: KeyItem[][] = [
  [
    { label: 'Esc', key: '\x1b' },
    { label: 'Tab', key: '\t' },
    { label: 'Ctrl', modifier: 'ctrl' },
    { label: 'Alt', modifier: 'alt' },
    { label: '↑', key: '\x1b[A' },
    { label: '↓', key: '\x1b[B' },
  ],
  [
    { label: '←', key: '\x1b[D' },
    { label: '→', key: '\x1b[C' },
    { label: '|', key: '|' },
    { label: '~', key: '~' },
    { label: '/', key: '/' },
    { label: 'Snippet', action: 'snippets' },
  ],
];

export function MobileKeyBar({ onSend }: MobileKeyBarProps) {
  const [activeModifiers, setActiveModifiers] = useState<Set<string>>(new Set());
  const { snippets, loaded, load } = useSnippetStore();
  const [showSnippets, setShowSnippets] = useState(false);

  useEffect(() => {
    if (showSnippets && !loaded) {
      void load();
    }
  }, [showSnippets, loaded, load]);

  const toggleModifier = (mod: string) => {
    setActiveModifiers((prev) => {
      const next = new Set(prev);
      if (next.has(mod)) {
        next.delete(mod);
      } else {
        next.add(mod);
      }
      return next;
    });
  };

  const handleKey = (item: KeyItem) => {
    if (item.action === 'snippets') {
      setShowSnippets((prev) => !prev);
      return;
    }
    if (item.modifier) {
      toggleModifier(item.modifier);
      return;
    }

    let data = item.key ?? '';
    if (activeModifiers.has('ctrl') && data.length === 1) {
      // Convert to control character: Ctrl+A = \x01, Ctrl+C = \x03, etc.
      const code = data.toUpperCase().charCodeAt(0) - 64;
      if (code >= 1 && code <= 26) {
        data = String.fromCharCode(code);
      }
    }
    if (activeModifiers.has('alt') && data.length === 1) {
      data = `\x1b${data}`;
    }

    onSend(data);
    setActiveModifiers(new Set());
  };

  const handleExecuteSnippet = (cmd: string) => {
    onSend(cmd + '\n');
    setShowSnippets(false);
  };

  return (
    <div className="flex flex-col gap-0.5 border-t border-[var(--color-border)] bg-[var(--color-bg-sidebar)] p-1">
      {/* Snippets Panel */}
      {showSnippets && (
        <div className="flex gap-1 overflow-x-auto py-1.5 px-1 border-b border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)]">
          {snippets.length === 0 ? (
            <span className="text-xs text-[var(--color-fg-muted)] px-2 py-1">
              No snippets found
            </span>
          ) : (
            snippets.map((s) => (
              <button
                key={s.id}
                onClick={() => handleExecuteSnippet(s.command)}
                className="shrink-0 rounded-full bg-[var(--color-bg-elevated)] border border-[var(--color-border-strong)] px-3 py-1 text-xs text-[var(--color-fg)] font-medium active:bg-[var(--color-accent-subtle)] hover:bg-[var(--color-accent-subtle)] transition-colors"
              >
                {s.title}
              </button>
            ))
          )}
        </div>
      )}

      {/* Auxiliary Key Rows */}
      {KEY_ROWS.map((row, ri) => (
        <div key={ri} className="flex gap-0.5">
          {row.map((item) => (
            <button
              key={item.label}
              onClick={() => handleKey(item)}
              className={`flex-1 rounded px-1.5 py-2 text-xs font-semibold transition-colors ${
                (item.modifier && activeModifiers.has(item.modifier)) || (item.action === 'snippets' && showSnippets)
                  ? 'bg-[var(--color-accent)] text-[var(--color-fg)]'
                  : 'bg-[var(--color-bg-elevated)] text-[var(--color-fg)] active:bg-[var(--color-border-strong)]'
              }`}
            >
              {item.label}
            </button>
          ))}
        </div>
      ))}
    </div>
  );
}
