import { useEffect } from 'react';
import { useShortcutsStore, matchesShortcut } from '@/stores/shortcuts-store';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';
import { useCommandPaletteStore } from '@/stores/command-store';

const DEFAULT_BINDINGS = [
  { id: 'new-tab', label: 'New Terminal Tab', category: 'Terminal', keys: 'Ctrl+T' },
  { id: 'close-tab', label: 'Close Tab', category: 'Terminal', keys: 'Ctrl+W' },
  { id: 'toggle-sidebar', label: 'Toggle Sidebar', category: 'View', keys: 'Ctrl+B' },
  { id: 'command-palette', label: 'Command Palette', category: 'General', keys: 'Ctrl+K' },
  { id: 'next-tab', label: 'Next Tab', category: 'Terminal', keys: 'Ctrl+PageDown' },
  { id: 'prev-tab', label: 'Previous Tab', category: 'Terminal', keys: 'Ctrl+PageUp' },
];

const ACTIONS: Record<string, () => void> = {
  'new-tab': () => useTabStore.getState().addTab(),
  'close-tab': () => {
    const { activeTabId, closeTab } = useTabStore.getState();
    if (activeTabId) closeTab(activeTabId);
  },
  'toggle-sidebar': () => useUiStore.getState().toggleSidebar(),
  'command-palette': () => {
    const store = useCommandPaletteStore.getState();
    if (store.isOpen) {
      store.close();
    } else {
      store.open();
    }
  },
  'next-tab': () => {
    const { tabs, activeTabId, setActiveTab } = useTabStore.getState();
    if (tabs.length < 2) return;
    const idx = tabs.findIndex((t) => t.id === activeTabId);
    const next = tabs[(idx + 1) % tabs.length];
    if (next) setActiveTab(next.id);
  },
  'prev-tab': () => {
    const { tabs, activeTabId, setActiveTab } = useTabStore.getState();
    if (tabs.length < 2) return;
    const idx = tabs.findIndex((t) => t.id === activeTabId);
    const prev = tabs[(idx - 1 + tabs.length) % tabs.length];
    if (prev) setActiveTab(prev.id);
  },
};

export function useKeyboardShortcuts() {
  const register = useShortcutsStore((s) => s.register);

  useEffect(() => {
    for (const def of DEFAULT_BINDINGS) {
      register({
        ...def,
        action: ACTIONS[def.id] ?? (() => {}),
      });
    }
  }, [register]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const { bindings } = useShortcutsStore.getState();
      for (const binding of bindings) {
        if (matchesShortcut(e, binding.keys)) {
          e.preventDefault();
          binding.action();
          return;
        }
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);
}
