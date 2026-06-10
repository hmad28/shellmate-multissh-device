import { create } from 'zustand';
import type { Group, GroupInput, Host, HostInput } from '@/types';
import { tauri } from '@/lib/tauri';

interface HostStore {
  hosts: Host[];
  groups: Group[];
  loading: boolean;
  error: string | null;
  searchQuery: string;
  expandedGroups: Set<string>;

  // Loaders
  loadAll: () => Promise<void>;
  loadHosts: () => Promise<void>;
  loadGroups: () => Promise<void>;

  // Hosts
  addHost: (input: HostInput) => Promise<Host>;
  updateHost: (id: string, input: HostInput) => Promise<Host>;
  deleteHost: (id: string) => Promise<void>;
  moveHostToGroup: (hostId: string, groupId: string | null) => Promise<void>;

  // Groups
  addGroup: (input: GroupInput) => Promise<Group>;
  updateGroup: (id: string, input: GroupInput) => Promise<Group>;
  deleteGroup: (id: string) => Promise<void>;

  // Search & UI
  setSearchQuery: (q: string) => void;
  toggleGroupExpanded: (id: string) => void;
  setGroupExpanded: (id: string, expanded: boolean) => void;
}

export const useHostStore = create<HostStore>((set, get) => ({
  hosts: [],
  groups: [],
  loading: false,
  error: null,
  searchQuery: '',
  expandedGroups: new Set<string>(),

  loadAll: async () => {
    set({ loading: true, error: null });
    try {
      const [hosts, groups] = await Promise.all([
        tauri.hosts.list(),
        tauri.groups.list(),
      ]);
      set({ hosts, groups, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  loadHosts: async () => {
    try {
      const hosts = await tauri.hosts.list();
      set({ hosts });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  loadGroups: async () => {
    try {
      const groups = await tauri.groups.list();
      set({ groups });
    } catch (err) {
      set({ error: String(err) });
    }
  },

  addHost: async (input) => {
    const host = await tauri.hosts.create(input);
    set({ hosts: [...get().hosts, host] });
    return host;
  },

  updateHost: async (id, input) => {
    const updated = await tauri.hosts.update(id, input);
    set({ hosts: get().hosts.map((h) => (h.id === id ? updated : h)) });
    return updated;
  },

  deleteHost: async (id) => {
    await tauri.hosts.delete(id);
    set({ hosts: get().hosts.filter((h) => h.id !== id) });
  },

  moveHostToGroup: async (hostId, groupId) => {
    await tauri.hosts.moveToGroup(hostId, groupId);
    set({
      hosts: get().hosts.map((h) => (h.id === hostId ? { ...h, groupId } : h)),
    });
  },

  addGroup: async (input) => {
    const group = await tauri.groups.create(input);
    set({ groups: [...get().groups, group] });
    return group;
  },

  updateGroup: async (id, input) => {
    const updated = await tauri.groups.update(id, input);
    set({ groups: get().groups.map((g) => (g.id === id ? updated : g)) });
    return updated;
  },

  deleteGroup: async (id) => {
    await tauri.groups.delete(id);
    // Detach: hosts that were in this group become ungrouped locally
    set({
      groups: get().groups.filter((g) => g.id !== id),
      hosts: get().hosts.map((h) =>
        h.groupId === id ? { ...h, groupId: null } : h,
      ),
    });
  },

  setSearchQuery: (q) => set({ searchQuery: q }),

  toggleGroupExpanded: (id) => {
    const next = new Set(get().expandedGroups);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    set({ expandedGroups: next });
  },

  setGroupExpanded: (id, expanded) => {
    const next = new Set(get().expandedGroups);
    if (expanded) next.add(id);
    else next.delete(id);
    set({ expandedGroups: next });
  },
}));
