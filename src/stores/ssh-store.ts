import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import { useTabStore } from './tab-store';
import { useToastStore } from './toast-store';

export interface ConnectionAttempt {
  tabId: string;
  type: 'saved' | 'quick';
  hostId?: string;
  quickParams?: any;
}

interface SshStore {
  sessionByTab: Record<string, string>;
  pendingAttempts: Record<string, ConnectionAttempt>;
  bind: (tabId: string, sessionId: string) => void;
  unbind: (tabId: string) => void;
  getSession: (tabId: string) => string | undefined;
  connectSaved: (tabId: string, hostId: string) => Promise<void>;
  connectQuick: (tabId: string, params: any) => Promise<void>;
  registerAttempt: (sessionId: string, attempt: ConnectionAttempt) => void;
  removeAttempt: (sessionId: string) => void;
  retryAttempt: (sessionId: string) => Promise<void>;
}

const generateSessionId = () => {
  if (typeof crypto !== 'undefined' && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
};

export const useSshStore = create<SshStore>((set, get) => ({
  sessionByTab: {},
  pendingAttempts: {},

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

  registerAttempt: (sessionId, attempt) =>
    set((state) => ({
      pendingAttempts: { ...state.pendingAttempts, [sessionId]: attempt },
    })),

  removeAttempt: (sessionId) =>
    set((state) => {
      const next = { ...state.pendingAttempts };
      delete next[sessionId];
      return { pendingAttempts: next };
    }),

  connectSaved: async (tabId, hostId) => {
    const { updateTabStatus } = useTabStore.getState();
    updateTabStatus(tabId, 'connecting');
    const sessionId = generateSessionId();
    get().bind(tabId, sessionId);
    get().registerAttempt(sessionId, { tabId, type: 'saved', hostId });
    try {
      await tauri.ssh.connect({ hostId, sessionId });
    } catch (err) {
      console.error('SSH connect failed', err);
      updateTabStatus(tabId, 'disconnected');
      get().unbind(tabId);
      get().removeAttempt(sessionId);
      useToastStore.getState().addToast('error', `Connection failed: ${err}`);
      throw err;
    }
  },

  connectQuick: async (tabId, params) => {
    const { updateTabStatus } = useTabStore.getState();
    updateTabStatus(tabId, 'connecting');
    const sessionId = generateSessionId();
    get().bind(tabId, sessionId);
    get().registerAttempt(sessionId, {
      tabId,
      type: 'quick',
      quickParams: params,
    });
    try {
      await tauri.ssh.quickConnect({ ...params, sessionId });
    } catch (err) {
      console.error('SSH connect failed', err);
      updateTabStatus(tabId, 'disconnected');
      get().unbind(tabId);
      get().removeAttempt(sessionId);
      useToastStore.getState().addToast('error', `Connection failed: ${err}`);
      throw err;
    }
  },

  retryAttempt: async (sessionId) => {
    const attempt = get().pendingAttempts[sessionId];
    if (!attempt) return;
    const { tabId, type, hostId, quickParams } = attempt;
    get().removeAttempt(sessionId);
    if (type === 'saved' && hostId) {
      await get().connectSaved(tabId, hostId);
    } else if (type === 'quick' && quickParams) {
      await get().connectQuick(tabId, quickParams);
    }
  },
}));
