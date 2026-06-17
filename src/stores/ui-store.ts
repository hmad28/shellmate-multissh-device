import { create } from 'zustand';

export type ActivePanel =
  | 'hosts'
  | 'terminal'
  | 'snippets'
  | 'settings'
  | 'sftp'
  | 'port-forward'
  | 'broadcast'
  | 'vip-access'
  | 'p2p-sync'
  | 'history'
  | 'server-stats'
  | 'docker'
  | 'import';

interface UiStore {
  sidebarCollapsed: boolean;
  activePanel: ActivePanel;
  vaultUnlocked: boolean;
  sftpSessionId: string | null;
  portForwardSessionId: string | null;
  draggedHostId: string | null;
  toggleSidebar: () => void;
  setActivePanel: (panel: ActivePanel) => void;
  setVaultUnlocked: (unlocked: boolean) => void;
  setSftpSessionId: (sessionId: string | null) => void;
  setPortForwardSessionId: (sessionId: string | null) => void;
  setDraggedHostId: (id: string | null) => void;
}

export const useUiStore = create<UiStore>((set) => ({
  sidebarCollapsed: false,
  activePanel: 'hosts',
  vaultUnlocked: false,
  sftpSessionId: null,
  portForwardSessionId: null,
  draggedHostId: null,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  setActivePanel: (panel) => set({ activePanel: panel }),
  setVaultUnlocked: (unlocked) => set({ vaultUnlocked: unlocked }),
  setSftpSessionId: (sessionId) => set({ sftpSessionId: sessionId }),
  setPortForwardSessionId: (sessionId) =>
    set({ portForwardSessionId: sessionId }),
  setDraggedHostId: (id) => set({ draggedHostId: id }),
}));
