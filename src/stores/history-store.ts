import { create } from 'zustand';
import { tauri } from '@/lib/tauri';

export interface HistoryEntry {
  id: string;
  sessionId: string;
  command: string;
  exitCode: number | null;
  workingDir: string | null;
  executedAt: string;
}

interface HistoryStore {
  entries: HistoryEntry[];
  loading: boolean;
  load: (sessionId?: string) => Promise<void>;
  search: (query: string) => Promise<void>;
  add: (sessionId: string, command: string, exitCode?: number, workingDir?: string) => Promise<void>;
  clear: (sessionId?: string) => Promise<void>;
}

export const useHistoryStore = create<HistoryStore>((set) => ({
  entries: [],
  loading: false,

  load: async (sessionId) => {
    set({ loading: true });
    try {
      const entries = await tauri.history.list(sessionId, 200);
      set({ entries });
    } catch (e) {
      console.error('Failed to load history', e);
    } finally {
      set({ loading: false });
    }
  },

  search: async (query) => {
    set({ loading: true });
    try {
      const entries = await tauri.history.search(query, 100);
      set({ entries });
    } catch (e) {
      console.error('Failed to search history', e);
    } finally {
      set({ loading: false });
    }
  },

  add: async (sessionId, command, exitCode, workingDir) => {
    try {
      await tauri.history.add({
        sessionId,
        command,
        ...(exitCode !== undefined ? { exitCode } : {}),
        ...(workingDir !== undefined ? { workingDir } : {}),
      });
    } catch (e) {
      console.error('Failed to add history entry', e);
    }
  },

  clear: async (sessionId) => {
    try {
      await tauri.history.clear(sessionId);
      set({ entries: [] });
    } catch (e) {
      console.error('Failed to clear history', e);
    }
  },
}));
