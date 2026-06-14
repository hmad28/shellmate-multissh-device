import { invoke } from '@tauri-apps/api/core';
import type { Group, GroupInput, Host, HostInput } from '@/types/host';
import type { Setting } from '@/types/settings';
import type { Snippet, SnippetInput } from '@/types/snippet';
import type { Theme, ThemeInput } from '@/types/theme';
import type {
  ConnectByHostInput,
  QuickConnectInput,
  VaultStatus,
  BiometricStatus,
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
import type { SyncStatus, SyncConfigureInput, SyncResult } from '@/types/sync';
import type {
  Team,
  TeamMember,
  TeamShare,
  CreateTeamInput,
  AddMemberInput,
  ShareHostInput,
} from '@/types/team';
import type { Plugin, PluginCapability } from '@/types/plugin';
import type { AuditEvent, AuditSettings, AuditQuery } from '@/types/audit';
import type { ServerStats } from '@/types/server-stats';
import type { SessionRecording, SessionEvent, SshKey, LocalSession, SftpTransfer, ParsedHost, ConnectionDiagnostics } from '@/types/advanced';

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
  git: {
    getInfo: (path?: string) =>
      invoke<{ branch: string | null; hasChanges: boolean; ahead: number; behind: number }>(
        'git_get_info',
        { path },
      ),
  },
  history: {
    add: (input: { sessionId: string; command: string; exitCode?: number; workingDir?: string }) =>
      invoke<string>('history_add', { input }),
    list: (sessionId?: string, limit?: number) =>
      invoke<{ id: string; sessionId: string; command: string; exitCode: number | null; workingDir: string | null; executedAt: string }[]>(
        'history_list',
        { sessionId, limit },
      ),
    search: (query: string, limit?: number) =>
      invoke<{ id: string; sessionId: string; command: string; exitCode: number | null; workingDir: string | null; executedAt: string }[]>(
        'history_search',
        { query, limit },
      ),
    clear: (sessionId?: string) =>
      invoke<void>('history_clear', { sessionId }),
  },
  biometric: {
    status: () => invoke<BiometricStatus>('biometric_status'),
    enable: () => invoke<void>('biometric_enable'),
    disable: () => invoke<void>('biometric_disable'),
    unlock: () => invoke<void>('biometric_unlock'),
  },
  sync: {
    status: () => invoke<SyncStatus>('sync_status'),
    configure: (input: SyncConfigureInput) =>
      invoke<void>('sync_configure', {
        backendType: input.backendType,
        endpointUrl: input.endpointUrl,
        credentials: input.credentials,
      }),
    now: () => invoke<SyncResult>('sync_now'),
    pause: () => invoke<void>('sync_pause'),
    resume: () => invoke<void>('sync_resume'),
  },
  team: {
    create: (input: CreateTeamInput) =>
      invoke<Team>('team_create', { input }),
    list: () => invoke<Team[]>('team_list'),
    delete: (teamId: string) =>
      invoke<void>('team_delete', { teamId }),
    addMember: (input: AddMemberInput) =>
      invoke<TeamMember>('team_add_member', { input }),
    listMembers: (teamId: string) =>
      invoke<TeamMember[]>('team_list_members', { teamId }),
    revokeMember: (memberId: string) =>
      invoke<void>('team_revoke_member', { memberId }),
    shareHost: (input: ShareHostInput) =>
      invoke<TeamShare>('team_share_host', { input }),
    listShares: (teamId: string) =>
      invoke<TeamShare[]>('team_list_shares', { teamId }),
    removeShare: (shareId: string) =>
      invoke<void>('team_remove_share', { shareId }),
  },
  plugin: {
    list: () => invoke<Plugin[]>('plugin_list'),
    install: (manifestJson: string, wasmPath: string) =>
      invoke<Plugin>('plugin_install', { manifestJson, wasmPath }),
    uninstall: (pluginId: string) =>
      invoke<void>('plugin_uninstall', { pluginId }),
    enable: (pluginId: string) =>
      invoke<void>('plugin_enable', { pluginId }),
    disable: (pluginId: string) =>
      invoke<void>('plugin_disable', { pluginId }),
    getCapabilities: (pluginId: string) =>
      invoke<PluginCapability[]>('plugin_get_capabilities', { pluginId }),
    grantCapability: (pluginId: string, capability: string) =>
      invoke<void>('plugin_grant_capability', { pluginId, capability }),
    revokeCapability: (pluginId: string, capability: string) =>
      invoke<void>('plugin_revoke_capability', { pluginId, capability }),
    execute: (pluginId: string) =>
      invoke<string>('plugin_execute', { pluginId }),
  },
  audit: {
    record: (eventType: string, payload: string, hostId?: string) =>
      invoke<string>('audit_record', { eventType, hostId: hostId ?? null, payload }),
    query: (filter: AuditQuery) =>
      invoke<AuditEvent[]>('audit_query', { filter }),
    export: (filter: AuditQuery) =>
      invoke<string>('audit_export', { filter }),
    purge: () => invoke<number>('audit_purge'),
    getSettings: (hostId: string) =>
      invoke<AuditSettings | null>('audit_get_settings', { hostId }),
    setSettings: (settings: AuditSettings) =>
      invoke<void>('audit_set_settings', { settings }),
  },
  export: {
    hostsEncrypted: (password: string) =>
      invoke<string>('export_hosts_encrypted', { exportPassword: password }),
    importHostsEncrypted: (data: string, password: string) =>
      invoke<number>('import_hosts_encrypted', { exportData: data, exportPassword: password }),
  },
  serverStats: {
    exec: (hostId: string) =>
      invoke<ServerStats>('server_stats_exec', { hostId }),
    execRaw: (hostId: string, command: string) =>
      invoke<string>('remote_exec', { hostId, command }),
  },
  recording: {
    start: (sessionId: string, hostId: string | null, hostLabel: string) =>
      invoke<string>('recording_start', { sessionId, hostId, hostLabel }),
    stop: (recordingId: string) =>
      invoke<void>('recording_stop', { recordingId }),
    event: (recordingId: string, eventType: string, data: string) =>
      invoke<void>('recording_event', { recordingId, eventType, data }),
    list: () => invoke<SessionRecording[]>('recording_list'),
    events: (recordingId: string) =>
      invoke<SessionEvent[]>('recording_events', { recordingId }),
    delete: (recordingId: string) =>
      invoke<void>('recording_delete', { recordingId }),
  },
  sshKey: {
    generate: (name: string, keyType: string, bits: number, passphrase?: string) =>
      invoke<SshKey>('ssh_key_generate', { name, keyType, bits, passphrase: passphrase ?? null }),
    list: () => invoke<SshKey[]>('ssh_key_list'),
    getPrivate: (keyId: string) =>
      invoke<string>('ssh_key_get_private', { keyId }),
    getPublic: (keyId: string) =>
      invoke<string>('ssh_key_get_public', { keyId }),
    delete: (keyId: string) =>
      invoke<void>('ssh_key_delete', { keyId }),
  },
  localShell: {
    spawn: (shell?: string) =>
      invoke<LocalSession>('local_shell_spawn', { shell: shell ?? null }),
    send: (sessionId: string, data: string) =>
      invoke<void>('local_shell_send', { sessionId, data }),
    read: (sessionId: string) =>
      invoke<string>('local_shell_read', { sessionId }),
    kill: (sessionId: string) =>
      invoke<void>('local_shell_kill', { sessionId }),
    list: () => invoke<string[]>('local_shell_list'),
  },
  hostTransfer: {
    start: (sourceHostId: string, sourcePath: string, destHostId: string, destPath: string) =>
      invoke<SftpTransfer>('sftp_host_transfer', { sourceHostId, sourcePath, destHostId, destPath }),
  },
  sshConfig: {
    import: (content: string) =>
      invoke<ParsedHost[]>('ssh_import_config', { content }),
  },
  diagnostics: {
    connection: (hostId: string) =>
      invoke<ConnectionDiagnostics>('connection_diagnose', { hostId }),
  },
} as const;
