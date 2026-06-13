import { create } from 'zustand';

export interface ShortcutBinding {
  id: string;
  label: string;
  category: string;
  keys: string;
  action: () => void;
}

interface ShortcutsStore {
  bindings: ShortcutBinding[];
  editingId: string | null;
  register: (binding: ShortcutBinding) => void;
  unregister: (id: string) => void;
  updateKeys: (id: string, keys: string) => void;
  setEditingId: (id: string | null) => void;
  getBinding: (id: string) => ShortcutBinding | undefined;
}

export const useShortcutsStore = create<ShortcutsStore>((set, get) => ({
  bindings: [],
  editingId: null,

  register: (binding) =>
    set((state) => {
      if (state.bindings.some((b) => b.id === binding.id)) return state;
      return { bindings: [...state.bindings, binding] };
    }),

  unregister: (id) =>
    set((state) => ({
      bindings: state.bindings.filter((b) => b.id !== id),
    })),

  updateKeys: (id, keys) =>
    set((state) => ({
      bindings: state.bindings.map((b) => (b.id === id ? { ...b, keys } : b)),
    })),

  setEditingId: (id) => set({ editingId: id }),

  getBinding: (id) => get().bindings.find((b) => b.id === id),
}));

export function formatKeyEvent(e: KeyboardEvent): string {
  const parts: string[] = [];
  if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
  if (e.shiftKey) parts.push('Shift');
  if (e.altKey) parts.push('Alt');
  const key = e.key === ' ' ? 'Space' : e.key.length === 1 ? e.key.toUpperCase() : e.key;
  if (!['Control', 'Shift', 'Alt', 'Meta'].includes(key)) {
    parts.push(key);
  }
  return parts.join('+');
}

export function matchesShortcut(e: KeyboardEvent, keys: string): boolean {
  const expected = parseKeys(keys);
  const actual = formatKeyEvent(e);
  return expected === actual;
}

function parseKeys(keys: string): string {
  return keys
    .split('+')
    .map((k) => k.trim())
    .sort((a, b) => {
      const order = ['Ctrl', 'Shift', 'Alt'];
      const ai = order.indexOf(a);
      const bi = order.indexOf(b);
      if (ai !== -1 && bi !== -1) return ai - bi;
      if (ai !== -1) return -1;
      if (bi !== -1) return 1;
      return a.localeCompare(b);
    })
    .join('+');
}
