import { create } from 'zustand';
import type { Host, HostInput, Group } from '@/types';
import { tauri } from '@/lib/tauri';

interface HostStore {
  hosts: Host[];
  groups: Group[];
  loading: boolean;
  error: string | null;
  loadHosts: () => Promise<void>;
  addHost: (input: HostInput) => Promise<void>;
  updateHost: (id: string, input: HostInput) => Promise<void>;
  deleteHost: (id: string) => Promise<void>;
}

export const useHostStore = create<HostStore>((set, get) => ({
  hosts: [],
  groups: [],
  loading: false,
  error: null,

  loadHosts: async () => {
    set({ loading: true, error: null });
    try {
      const hosts = await tauri.hosts.list();
      set({ hosts, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  addHost: async (input) => {
    const host = await tauri.hosts.create(input);
    set({ hosts: [...get().hosts, host] });
  },

  updateHost: async (id, input) => {
    const updated = await tauri.hosts.update(id, input);
    set({
      hosts: get().hosts.map((h) => (h.id === id ? updated : h)),
    });
  },

  deleteHost: async (id) => {
    await tauri.hosts.delete(id);
    set({ hosts: get().hosts.filter((h) => h.id !== id) });
  },
}));
