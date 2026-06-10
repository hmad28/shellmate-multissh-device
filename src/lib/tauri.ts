import { invoke } from '@tauri-apps/api/core';
import type { Group, GroupInput, Host, HostInput } from '@/types/host';
import type { Setting } from '@/types/settings';
import type { Snippet, SnippetInput } from '@/types/snippet';
import type { Theme, ThemeInput } from '@/types/theme';
import type {
  ConnectByHostInput,
  QuickConnectInput,
  VaultStatus,
} from '@/types/ssh';

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
    search: (query: string) => invoke<Host[]>('search_hosts', { query }),
    moveToGroup: (hostId: string, groupId: string | null) =>
      invoke<void>('move_host_to_group', { hostId, groupId }),
  },
  groups: {
    list: () => invoke<Group[]>('get_groups'),
    create: (input: GroupInput) => invoke<Group>('create_group', { input }),
    update: (id: string, input: GroupInput) =>
      invoke<Group>('update_group', { id, input }),
    delete: (id: string) => invoke<void>('delete_group', { id }),
  },
  snippets: {
    list: () => invoke<Snippet[]>('get_snippets'),
    create: (input: SnippetInput) =>
      invoke<Snippet>('create_snippet', { input }),
    update: (id: string, input: SnippetInput) =>
      invoke<Snippet>('update_snippet', { id, input }),
    delete: (id: string) => invoke<void>('delete_snippet', { id }),
  },
  themes: {
    list: () => invoke<Theme[]>('get_themes'),
    save: (input: ThemeInput) => invoke<Theme>('save_theme', { input }),
    delete: (id: string) => invoke<void>('delete_theme', { id }),
  },
  settings: {
    list: () => invoke<Setting[]>('get_settings'),
    set: (key: string, value: string) =>
      invoke<void>('set_setting', { key, value }),
  },
  vault: {
    status: () => invoke<VaultStatus>('vault_status'),
    setup: (password: string) => invoke<void>('vault_setup', { password }),
    unlock: (password: string) => invoke<void>('vault_unlock', { password }),
    lock: () => invoke<void>('vault_lock'),
    checkIdle: () => invoke<boolean>('vault_check_idle'),
    recordActivity: () => invoke<void>('vault_record_activity'),
    changeMasterPassword: (currentPassword: string, newPassword: string) =>
      invoke<void>('vault_change_master_password', {
        currentPassword,
        newPassword,
      }),
  },
  credentials: {
    save: (credType: 'password' | 'private_key', plaintext: string) =>
      invoke<string>('save_credential', { credType, plaintext }),
    delete: (id: string) => invoke<void>('delete_credential', { id }),
  },
  ssh: {
    connect: (input: ConnectByHostInput) =>
      invoke<string>('ssh_connect', { input }),
    quickConnect: (input: QuickConnectInput) =>
      invoke<string>('ssh_quick_connect', { input }),
    send: (sessionId: string, data: string) =>
      invoke<void>('ssh_send', { sessionId, data }),
    resize: (sessionId: string, cols: number, rows: number) =>
      invoke<void>('ssh_resize', { sessionId, cols, rows }),
    disconnect: (sessionId: string) =>
      invoke<void>('ssh_disconnect', { sessionId }),
  },
  app: {
    version: () => invoke<string>('app_version'),
  },
} as const;
