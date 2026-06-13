import { useState } from 'react';

interface MobileKeyBarProps {
  onSend: (data: string) => void;
}

const KEY_ROWS = [
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
    { label: '-', key: '-' },
    { label: '/', key: '/' },
  ],
];

export function MobileKeyBar({ onSend }: MobileKeyBarProps) {
  const [activeModifiers, setActiveModifiers] = useState<Set<string>>(new Set());

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

  const handleKey = (key: string, modifier?: string) => {
    if (modifier) {
      toggleModifier(modifier);
      return;
    }

    let data = key;
    if (activeModifiers.has('ctrl') && key.length === 1) {
      // Convert to control character: Ctrl+A = \x01, Ctrl+C = \x03, etc.
      const code = key.toUpperCase().charCodeAt(0) - 64;
      if (code >= 1 && code <= 26) {
        data = String.fromCharCode(code);
      }
    }
    if (activeModifiers.has('alt') && key.length === 1) {
      data = `\x1b${key}`;
    }

    onSend(data);
    setActiveModifiers(new Set());
  };

  return (
    <div className="flex flex-col gap-0.5 border-t border-[var(--color-border)] bg-[var(--color-surface)] p-1">
      {KEY_ROWS.map((row, ri) => (
        <div key={ri} className="flex gap-0.5">
          {row.map((item) => (
            <button
              key={item.label}
              onClick={() => handleKey(item.key ?? '', item.modifier)}
              className={`flex-1 rounded px-1.5 py-1.5 text-xs font-medium transition-colors ${
                item.modifier && activeModifiers.has(item.modifier)
                  ? 'bg-[var(--color-primary)] text-[var(--color-primary-fg)]'
                  : 'bg-[var(--color-surface-alt)] text-[var(--color-text)] active:bg-[var(--color-border)]'
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
