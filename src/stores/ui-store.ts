import { create } from 'zustand';

export type ActivePanel = 'hosts' | 'snippets' | 'settings' | 'sftp' | 'port-forward' | 'broadcast';

interface UiStore {
  sidebarCollapsed: boolean;
  activePanel: ActivePanel;
  vaultUnlocked: boolean;
  sftpSessionId: string | null;
  portForwardSessionId: string | null;
  toggleSidebar: () => void;
  setActivePanel: (panel: ActivePanel) => void;
  setVaultUnlocked: (unlocked: boolean) => void;
  setSftpSessionId: (sessionId: string | null) => void;
  setPortForwardSessionId: (sessionId: string | null) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  sidebarCollapsed: false,
  activePanel: 'hosts',
  vaultUnlocked: false,
  sftpSessionId: null,
  portForwardSessionId: null,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setActivePanel: (panel) => set({ activePanel: panel }),
  setVaultUnlocked: (unlocked) => set({ vaultUnlocked: unlocked }),
  setSftpSessionId: (sessionId) => set({ sftpSessionId: sessionId }),
  setPortForwardSessionId: (sessionId) => set({ portForwardSessionId: sessionId }),
}));
