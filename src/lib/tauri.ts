import { invoke } from '@tauri-apps/api/core';
import type { Host, HostInput } from '@/types/host';
import type { Setting } from '@/types/settings';

/**
 * Typed wrappers around Tauri `invoke`.
 * Keep all backend command names in one place.
 */

export const tauri = {
  hosts: {
    list: () => invoke<Host[]>('get_hosts'),
    create: (input: HostInput) => invoke<Host>('create_host', { input }),
    update: (id: string, input: HostInput) =>
      invoke<Host>('update_host', { id, input }),
    delete: (id: string) => invoke<void>('delete_host', { id }),
  },
  settings: {
    list: () => invoke<Setting[]>('get_settings'),
    set: (key: string, value: string) =>
      invoke<void>('set_setting', { key, value }),
  },
  app: {
    version: () => invoke<string>('app_version'),
  },
} as const;
