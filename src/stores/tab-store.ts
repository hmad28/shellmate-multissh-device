import { create } from 'zustand';
import type { Tab, ConnectionStatus } from '@/types';

interface TabStore {
  tabs: Tab[];
  activeTabId: string | null;
  addTab: (params?: { hostId?: string; label?: string }) => string;
  closeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTabStatus: (id: string, status: ConnectionStatus) => void;
  reorderTabs: (fromIndex: number, toIndex: number) => void;
}

let tabCounter = 0;
const newTabId = () => `tab_${Date.now()}_${tabCounter++}`;

export const useTabStore = create<TabStore>((set) => ({
  tabs: [],
  activeTabId: null,

  addTab: (params) => {
    const id = newTabId();
    const tab: Tab = {
      id,
      hostId: params?.hostId ?? null,
      label: params?.label ?? 'New Tab',
      status: 'disconnected',
    };
    set((state) => ({
      tabs: [...state.tabs, tab],
      activeTabId: id,
    }));
    return id;
  },

  closeTab: (id) =>
    set((state) => {
      const remaining = state.tabs.filter((t) => t.id !== id);
      let nextActive = state.activeTabId;
      if (state.activeTabId === id) {
        nextActive =
          remaining.length > 0
            ? (remaining[remaining.length - 1]?.id ?? null)
            : null;
      }
      return { tabs: remaining, activeTabId: nextActive };
    }),

  setActiveTab: (id) => set({ activeTabId: id }),

  updateTabStatus: (id, status) =>
    set((state) => ({
      tabs: state.tabs.map((t) => (t.id === id ? { ...t, status } : t)),
    })),

  reorderTabs: (fromIndex, toIndex) =>
    set((state) => {
      const next = state.tabs.slice();
      const [moved] = next.splice(fromIndex, 1);
      if (moved) next.splice(toIndex, 0, moved);
      return { tabs: next };
    }),
}));
