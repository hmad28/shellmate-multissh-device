import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import type { ServerStats } from '@/types/server-stats';

interface StatsStore {
  stats: Record<string, ServerStats>;
  loading: Record<string, boolean>;
  errors: Record<string, string>;
  fetchStats: (hostId: string) => Promise<void>;
  clearStats: (hostId: string) => void;
}

export const useStatsStore = create<StatsStore>((set) => ({
  stats: {},
  loading: {},
  errors: {},

  fetchStats: async (hostId) => {
    set((s) => ({ loading: { ...s.loading, [hostId]: true }, errors: { ...s.errors, [hostId]: '' } }));
    try {
      const stats = await tauri.serverStats.exec(hostId);
      set((s) => ({
        stats: { ...s.stats, [hostId]: stats },
        loading: { ...s.loading, [hostId]: false },
      }));
    } catch (err) {
      set((s) => ({
        errors: { ...s.errors, [hostId]: String(err) },
        loading: { ...s.loading, [hostId]: false },
      }));
    }
  },

  clearStats: (hostId) => {
    set((s) => {
      const stats = { ...s.stats };
      const loading = { ...s.loading };
      const errors = { ...s.errors };
      delete stats[hostId];
      delete loading[hostId];
      delete errors[hostId];
      return { stats, loading, errors };
    });
  },
}));
