import { create } from 'zustand';
import { tauri } from '@/lib/tauri';

interface VaultStore {
  initialized: boolean;
  unlocked: boolean;
  loading: boolean;
  error: string | null;
  refresh: () => Promise<void>;
  setup: (password: string) => Promise<void>;
  unlock: (password: string) => Promise<void>;
  lock: () => Promise<void>;
  recordActivity: () => Promise<void>;
}

export const useVaultStore = create<VaultStore>((set, get) => ({
  initialized: false,
  unlocked: false,
  loading: false,
  error: null,

  refresh: async () => {
    set({ loading: true, error: null });
    try {
      const status = await tauri.vault.status();
      set({
        initialized: status.initialized,
        unlocked: status.unlocked,
        loading: false,
      });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  setup: async (password) => {
    set({ loading: true, error: null });
    try {
      await tauri.vault.setup(password);
      await get().refresh();
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },

  unlock: async (password) => {
    set({ loading: true, error: null });
    try {
      await tauri.vault.unlock(password);
      await get().refresh();
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },

  lock: async () => {
    await tauri.vault.lock();
    await get().refresh();
  },

  recordActivity: async () => {
    if (!get().unlocked) return;
    try {
      await tauri.vault.recordActivity();
    } catch {
      // non-fatal
    }
  },
}));
