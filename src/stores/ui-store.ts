import { create } from 'zustand';

export type ActivePanel = 'hosts' | 'snippets' | 'settings';

interface UiStore {
  sidebarCollapsed: boolean;
  activePanel: ActivePanel;
  vaultUnlocked: boolean;
  toggleSidebar: () => void;
  setActivePanel: (panel: ActivePanel) => void;
  setVaultUnlocked: (unlocked: boolean) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  sidebarCollapsed: false,
  activePanel: 'hosts',
  vaultUnlocked: false,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setActivePanel: (panel) => set({ activePanel: panel }),
  setVaultUnlocked: (unlocked) => set({ vaultUnlocked: unlocked }),
}));
