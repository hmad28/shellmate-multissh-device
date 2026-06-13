import { create } from 'zustand';

export interface Command {
  id: string;
  label: string;
  category: string;
  shortcut?: string;
  action: () => void;
  keywords?: string[];
}

interface CommandPaletteStore {
  commands: Command[];
  isOpen: boolean;
  query: string;
  selectedIndex: number;
  register: (command: Command) => void;
  unregister: (id: string) => void;
  open: () => void;
  close: () => void;
  setQuery: (query: string) => void;
  setSelectedIndex: (index: number) => void;
  executeSelected: () => void;
}

export const useCommandPaletteStore = create<CommandPaletteStore>((set, get) => ({
  commands: [],
  isOpen: false,
  query: '',
  selectedIndex: 0,

  register: (command) =>
    set((state) => {
      if (state.commands.some((c) => c.id === command.id)) return state;
      return { commands: [...state.commands, command] };
    }),

  unregister: (id) =>
    set((state) => ({
      commands: state.commands.filter((c) => c.id !== id),
    })),

  open: () => set({ isOpen: true, query: '', selectedIndex: 0 }),
  close: () => set({ isOpen: false, query: '', selectedIndex: 0 }),

  setQuery: (query) => set({ query, selectedIndex: 0 }),

  setSelectedIndex: (index) => set({ selectedIndex: index }),

  executeSelected: () => {
    const { commands, query, selectedIndex } = get();
    const filtered = filterCommands(commands, query);
    const cmd = filtered[selectedIndex];
    if (cmd) {
      get().close();
      cmd.action();
    }
  },
}));

export function filterCommands(commands: Command[], query: string): Command[] {
  if (!query.trim()) return commands;
  const lower = query.toLowerCase();
  return commands
    .map((cmd) => {
      const haystack = [cmd.label, cmd.category, ...(cmd.keywords ?? [])]
        .join(' ')
        .toLowerCase();
      const score = haystack.includes(lower) ? 1 : 0;
      return { cmd, score };
    })
    .filter((r) => r.score > 0)
    .sort((a, b) => b.score - a.score)
    .map((r) => r.cmd);
}
