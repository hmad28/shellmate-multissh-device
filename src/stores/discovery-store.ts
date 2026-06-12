import { create } from 'zustand';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { tauri } from '@/lib/tauri';

export interface DiscoveredHost {
  hostname: string;
  ipAddresses: string[];
  port: number;
}

interface DiscoveryStore {
  hosts: DiscoveredHost[];
  isScanning: boolean;
  startDiscovery: () => Promise<void>;
  stopDiscovery: () => Promise<void>;
}

let unlisten: UnlistenFn | null = null;

export const useDiscoveryStore = create<DiscoveryStore>((set, get) => ({
  hosts: [],
  isScanning: false,

  startDiscovery: async () => {
    if (get().isScanning) return;
    try {
      set({ isScanning: true, hosts: [] });
      await tauri.discovery.start();

      unlisten = await listen<DiscoveredHost>(
        'discovery:host_found',
        (event) => {
          set((state) => {
            // Prevent duplicates
            const exists = state.hosts.find(
              (h) => h.hostname === event.payload.hostname,
            );
            if (exists) return state;
            return { hosts: [...state.hosts, event.payload] };
          });
        },
      );
    } catch (err) {
      console.error('Failed to start discovery', err);
      set({ isScanning: false });
    }
  },

  stopDiscovery: async () => {
    if (!get().isScanning) return;
    try {
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
      await tauri.discovery.stop();
      set({ isScanning: false });
    } catch (err) {
      console.error('Failed to stop discovery', err);
    }
  },
}));
