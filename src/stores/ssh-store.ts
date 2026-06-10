import { create } from 'zustand';

/**
 * Maps frontend tab id → backend SSH session id.
 * Lifecycle:
 *   - on connect: bind(tabId, sessionId)
 *   - on tab close or disconnect: unbind(tabId)
 */
interface SshStore {
  sessionByTab: Record<string, string>;
  bind: (tabId: string, sessionId: string) => void;
  unbind: (tabId: string) => void;
  getSession: (tabId: string) => string | undefined;
}

export const useSshStore = create<SshStore>((set, get) => ({
  sessionByTab: {},
  bind: (tabId, sessionId) =>
    set((state) => ({
      sessionByTab: { ...state.sessionByTab, [tabId]: sessionId },
    })),
  unbind: (tabId) =>
    set((state) => {
      const next = { ...state.sessionByTab };
      delete next[tabId];
      return { sessionByTab: next };
    }),
  getSession: (tabId) => get().sessionByTab[tabId],
}));
