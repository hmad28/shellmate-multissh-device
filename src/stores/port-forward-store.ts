import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import type { PortForwardRule, PortForwardType } from '@/types/port-forward';

interface PortForwardStore {
  forwards: Record<string, PortForwardRule[]>;
  loadForwards: (sessionId: string) => Promise<void>;
  createForward: (
    sessionId: string,
    ruleType: PortForwardType,
    localPort: number,
    remoteHost: string,
    remotePort: number,
  ) => Promise<void>;
  removeForward: (forwardId: string, sessionId: string) => Promise<void>;
  toggleForward: (forwardId: string, sessionId: string) => Promise<void>;
}

export const usePortForwardStore = create<PortForwardStore>((set) => ({
  forwards: {},

  loadForwards: async (sessionId) => {
    const forwards = await tauri.portForward.list({ sessionId });
    set((state) => ({
      forwards: { ...state.forwards, [sessionId]: forwards },
    }));
  },

  createForward: async (
    sessionId,
    ruleType,
    localPort,
    remoteHost,
    remotePort,
  ) => {
    const rule = await tauri.portForward.create({
      sessionId,
      ruleType,
      localPort,
      remoteHost,
      remotePort,
    });
    set((state) => ({
      forwards: {
        ...state.forwards,
        [sessionId]: [...(state.forwards[sessionId] || []), rule],
      },
    }));
  },

  removeForward: async (forwardId, sessionId) => {
    await tauri.portForward.remove({ forwardId });
    set((state) => ({
      forwards: {
        ...state.forwards,
        [sessionId]:
          state.forwards[sessionId]?.filter((f) => f.id !== forwardId) || [],
      },
    }));
  },

  toggleForward: async (forwardId, sessionId) => {
    const rule = await tauri.portForward.toggle({ forwardId });
    set((state) => ({
      forwards: {
        ...state.forwards,
        [sessionId]:
          state.forwards[sessionId]?.map((f) =>
            f.id === forwardId ? rule : f,
          ) || [],
      },
    }));
  },
}));
