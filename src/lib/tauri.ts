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
import type {
  SftpFile,
  SftpOpenInput,
  SftpListInput,
  SftpUploadInput,
  SftpDownloadInput,
  SftpRenameInput,
  SftpPathInput,
  SftpCloseInput,
} from '@/types/sftp';
import type {
  PortForwardRule,
  PortForwardCreateInput,
  PortForwardListInput,
  PortForwardIdInput,
} from '@/types/port-forward';
import type {
  KnownHost,
  HostKeyVerificationResult,
  VerifyHostKeyInput,
  TrustHostKeyInput,
} from '@/types/known-hosts';

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
  sftp: {
    open: (input: SftpOpenInput) => invoke<string>('sftp_open', { input }),
    list: (input: SftpListInput) => invoke<SftpFile[]>('sftp_list', { input }),
    upload: (input: SftpUploadInput) => invoke<void>('sftp_upload', { input }),
    download: (input: SftpDownloadInput) =>
      invoke<void>('sftp_download', { input }),
    rename: (input: SftpRenameInput) => invoke<void>('sftp_rename', { input }),
    remove: (input: SftpPathInput) => invoke<void>('sftp_remove', { input }),
    mkdir: (input: SftpPathInput) => invoke<void>('sftp_mkdir', { input }),
    close: (input: SftpCloseInput) => invoke<void>('sftp_close', { input }),
  },
  portForward: {
    create: (input: PortForwardCreateInput) =>
      invoke<PortForwardRule>('port_forward_create', { input }),
    list: (input: PortForwardListInput) =>
      invoke<PortForwardRule[]>('port_forward_list', { input }),
    remove: (input: PortForwardIdInput) =>
      invoke<void>('port_forward_remove', { input }),
    toggle: (input: PortForwardIdInput) =>
      invoke<PortForwardRule>('port_forward_toggle', { input }),
  },
  knownHosts: {
    verify: (input: VerifyHostKeyInput) =>
      invoke<HostKeyVerificationResult>('known_hosts_verify', { input }),
    trust: (input: TrustHostKeyInput) =>
      invoke<KnownHost>('known_hosts_trust', { input }),
    list: () => invoke<KnownHost[]>('known_hosts_list'),
    remove: (id: string) => invoke<void>('known_hosts_remove', { id }),
    setTrusted: (id: string, trusted: boolean) =>
      invoke<void>('known_hosts_set_trusted', { id, trusted }),
  },
  broadcast: {
    add: (sessionId: string) => invoke<void>('broadcast_add', { sessionId }),
    remove: (sessionId: string) =>
      invoke<void>('broadcast_remove', { sessionId }),
    isActive: (sessionId: string) =>
      invoke<boolean>('broadcast_is_active', { sessionId }),
    getSessions: () => invoke<string[]>('broadcast_get_sessions'),
    send: (sessionId: string, data: string) =>
      invoke<void>('broadcast_send', { sessionId, data }),
  },
  discovery: {
    start: () => invoke<void>('start_discovery'),
    stop: () => invoke<void>('stop_discovery'),
    startBroadcasting: (serviceName: string, port: number) =>
      invoke<void>('start_broadcasting', { serviceName, port }),
  },
  vipAccess: {
    generateKeypair: () => invoke<string>('vip_generate_keypair'),
    injectAuthorizedKeys: (pubkeyHex: string) =>
      invoke<string>('vip_inject_authorized_keys', { pubkeyHex }),
    createLocalhostHost: (credentialId: string, label?: string) =>
      invoke<string>('vip_create_localhost_host', { credentialId, label }),
    getKeyStatus: () =>
      invoke<{ hostExists: boolean; authorizedKeysInjected: boolean }>(
        'vip_get_key_status',
      ),
  },
  p2p: {
    startSyncServer: () => invoke<string>('p2p_start_sync_server'),
    stopSyncServer: () => invoke<void>('p2p_stop_sync_server'),
    getSyncStatus: () =>
      invoke<{ isRunning: boolean; hasPin: boolean }>('p2p_get_sync_status'),
    exportForSync: () => invoke<string>('p2p_export_for_sync'),
  },
  app: {
    version: () => invoke<string>('app_version'),
  },
} as const;
