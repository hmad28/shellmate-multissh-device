import { create } from 'zustand';
import { tauri } from '@/lib/tauri';

interface BroadcastStore {
  broadcastSessions: Set<string>;
  loadSessions: () => Promise<void>;
  addSession: (sessionId: string) => Promise<void>;
  removeSession: (sessionId: string) => Promise<void>;
  isSessionActive: (sessionId: string) => boolean;
  sendToAll: (data: string) => Promise<void>;
}

export const useBroadcastStore = create<BroadcastStore>((set, get) => ({
  broadcastSessions: new Set(),

  loadSessions: async () => {
    const sessions = await tauri.broadcast.getSessions();
    set({ broadcastSessions: new Set(sessions) });
  },

  addSession: async (sessionId: string) => {
    await tauri.broadcast.add(sessionId);
    set((state) => ({
      broadcastSessions: new Set([...state.broadcastSessions, sessionId]),
    }));
  },

  removeSession: async (sessionId: string) => {
    await tauri.broadcast.remove(sessionId);
    set((state) => {
      const newSessions = new Set(state.broadcastSessions);
      newSessions.delete(sessionId);
      return { broadcastSessions: newSessions };
    });
  },

  isSessionActive: (sessionId: string) => {
    return get().broadcastSessions.has(sessionId);
  },

  sendToAll: async (data: string) => {
    const sessions = Array.from(get().broadcastSessions);
    for (const sessionId of sessions) {
      await tauri.broadcast.send(sessionId, data);
    }
  },
}));
