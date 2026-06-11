import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import type { SftpFile } from '@/types/sftp';

interface SftpTransfer {
  id: string;
  filename: string;
  bytesTransferred: number;
  totalBytes: number;
  progress: number;
}

interface SftpBrowser {
  sftpId: string;
  sessionId: string;
  currentPath: string;
  files: SftpFile[];
  loading: boolean;
  error: string | null;
  transfers: SftpTransfer[];
}

interface SftpStore {
  browsers: Record<string, SftpBrowser>;
  activeBrowserId: string | null;
  openBrowser: (sessionId: string) => Promise<string>;
  closeBrowser: (sftpId: string) => Promise<void>;
  listDirectory: (sftpId: string, path?: string) => Promise<void>;
  uploadFile: (sftpId: string, localPath: string, remotePath: string) => Promise<void>;
  downloadFile: (sftpId: string, remotePath: string, localPath: string) => Promise<void>;
  renameFile: (sftpId: string, oldPath: string, newPath: string) => Promise<void>;
  removeFile: (sftpId: string, path: string) => Promise<void>;
  mkdir: (sftpId: string, path: string) => Promise<void>;
  setActiveBrowser: (sftpId: string | null) => void;
  updateTransfer: (transfer: SftpTransfer) => void;
  removeTransfer: (transferId: string) => void;
}

export const useSftpStore = create<SftpStore>((set, get) => ({
  browsers: {},
  activeBrowserId: null,

  openBrowser: async (sessionId) => {
    const sftpId = await tauri.sftp.open({ sessionId });
    set((state) => ({
      browsers: {
        ...state.browsers,
        [sftpId]: {
          sftpId,
          sessionId,
          currentPath: '.',
          files: [],
          loading: false,
          error: null,
          transfers: [],
        },
      },
      activeBrowserId: sftpId,
    }));
    await get().listDirectory(sftpId);
    return sftpId;
  },

  closeBrowser: async (sftpId) => {
    await tauri.sftp.close({ sftpId });
    set((state) => {
      const browsers = { ...state.browsers };
      delete browsers[sftpId];
      const activeBrowserId = state.activeBrowserId === sftpId ? null : state.activeBrowserId;
      return { browsers, activeBrowserId };
    });
  },

  listDirectory: async (sftpId, path) => {
    set((state) => {
      const browser = state.browsers[sftpId];
      if (!browser) return state;
      return {
        browsers: {
          ...state.browsers,
          [sftpId]: { ...browser, loading: true, error: null },
        },
      };
    });
    try {
      const files = await tauri.sftp.list({ sftpId, path: path || '.' });
      set((state) => {
        const browser = state.browsers[sftpId];
        if (!browser) return state;
        return {
          browsers: {
            ...state.browsers,
            [sftpId]: {
              ...browser,
              files,
              currentPath: path || browser.currentPath,
              loading: false,
            },
          },
        };
      });
    } catch (error) {
      set((state) => {
        const browser = state.browsers[sftpId];
        if (!browser) return state;
        return {
          browsers: {
            ...state.browsers,
            [sftpId]: {
              ...browser,
              loading: false,
              error: error instanceof Error ? error.message : 'Failed to list directory',
            },
          },
        };
      });
    }
  },

  uploadFile: async (sftpId, localPath, remotePath) => {
    try {
      await tauri.sftp.upload({ sftpId, localPath, remotePath });
      await get().listDirectory(sftpId);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Upload failed';
      set((state) => {
        const browser = state.browsers[sftpId];
        if (!browser) return state;
        return {
          browsers: {
            ...state.browsers,
            [sftpId]: { ...browser, error: message },
          },
        };
      });
    }
  },

  downloadFile: async (sftpId, remotePath, localPath) => {
    try {
      await tauri.sftp.download({ sftpId, remotePath, localPath });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Download failed';
      set((state) => {
        const browser = state.browsers[sftpId];
        if (!browser) return state;
        return {
          browsers: {
            ...state.browsers,
            [sftpId]: { ...browser, error: message },
          },
        };
      });
    }
  },

  renameFile: async (sftpId, oldPath, newPath) => {
    try {
      await tauri.sftp.rename({ sftpId, oldPath, newPath });
      await get().listDirectory(sftpId);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Rename failed';
      set((state) => {
        const browser = state.browsers[sftpId];
        if (!browser) return state;
        return {
          browsers: {
            ...state.browsers,
            [sftpId]: { ...browser, error: message },
          },
        };
      });
    }
  },

  removeFile: async (sftpId, path) => {
    try {
      await tauri.sftp.remove({ sftpId, path });
      await get().listDirectory(sftpId);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Delete failed';
      set((state) => {
        const browser = state.browsers[sftpId];
        if (!browser) return state;
        return {
          browsers: {
            ...state.browsers,
            [sftpId]: { ...browser, error: message },
          },
        };
      });
    }
  },

  mkdir: async (sftpId, path) => {
    try {
      await tauri.sftp.mkdir({ sftpId, path });
      await get().listDirectory(sftpId);
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to create directory';
      set((state) => {
        const browser = state.browsers[sftpId];
        if (!browser) return state;
        return {
          browsers: {
            ...state.browsers,
            [sftpId]: { ...browser, error: message },
          },
        };
      });
    }
  },

  setActiveBrowser: (sftpId) => {
    set({ activeBrowserId: sftpId });
  },

  updateTransfer: (transfer) => {
    set((state) => {
      const browserId = Object.keys(state.browsers).find(id => {
        const browser = state.browsers[id];
        return browser && browser.transfers.some(t => t.id === transfer.id);
      });
      if (!browserId) return state;
      
      const browser = state.browsers[browserId];
      if (!browser) return state;

      return {
        browsers: {
          ...state.browsers,
          [browser.sftpId]: {
            ...browser,
            transfers: browser.transfers.map(t => 
              t.id === transfer.id ? transfer : t
            ),
          },
        },
      };
    });
  },

  removeTransfer: (transferId) => {
    set((state) => {
      const browsers = { ...state.browsers };
      for (const sftpId in browsers) {
        const browser = browsers[sftpId];
        if (browser && browser.transfers) {
          browsers[sftpId] = {
            ...browser,
            transfers: browser.transfers.filter(t => t.id !== transferId),
          };
        }
      }
      return { browsers };
    });
  },
}));
