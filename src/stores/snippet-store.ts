import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import type { Snippet, SnippetInput } from '@/types/snippet';

interface SnippetStore {
  snippets: Snippet[];
  loaded: boolean;
  loading: boolean;
  error: string | null;
  searchQuery: string;
  load: () => Promise<void>;
  add: (input: SnippetInput) => Promise<Snippet>;
  update: (id: string, input: SnippetInput) => Promise<Snippet>;
  remove: (id: string) => Promise<void>;
  setSearchQuery: (q: string) => void;
}

export const useSnippetStore = create<SnippetStore>((set, get) => ({
  snippets: [],
  loaded: false,
  loading: false,
  error: null,
  searchQuery: '',

  load: async () => {
    set({ loading: true, error: null });
    try {
      const snippets = await tauri.snippets.list();
      set({ snippets, loaded: true, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  add: async (input) => {
    const created = await tauri.snippets.create(input);
    set({ snippets: [...get().snippets, created] });
    return created;
  },

  update: async (id, input) => {
    const updated = await tauri.snippets.update(id, input);
    set({
      snippets: get().snippets.map((s) => (s.id === id ? updated : s)),
    });
    return updated;
  },

  remove: async (id) => {
    await tauri.snippets.delete(id);
    set({ snippets: get().snippets.filter((s) => s.id !== id) });
  },

  setSearchQuery: (q) => set({ searchQuery: q }),
}));
