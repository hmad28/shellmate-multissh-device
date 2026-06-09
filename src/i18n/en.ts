/**
 * User-facing strings (English, MVP default).
 * Post-MVP: extract to `react-i18next` or similar.
 */
export const strings = {
  app: {
    name: 'ShellMate',
    tagline: 'Self-hosted, local-first SSH client',
    locked: 'Vault Locked',
    ready: 'Ready',
  },
  titlebar: {
    minimize: 'Minimize',
    maximize: 'Maximize',
    close: 'Close',
  },
  sidebar: {
    searchPlaceholder: 'Search hosts...',
    addHost: 'Add Host',
    snippets: 'Snippets',
    settings: 'Settings',
    noHosts: 'No hosts yet. Click "Add Host" to start.',
    groups: {
      production: 'Production',
      staging: 'Staging',
      development: 'Development',
      ungrouped: 'Ungrouped',
    },
  },
  tabs: {
    newTab: 'New tab',
    closeTab: 'Close tab',
    noTabs: 'No active sessions. Open a host from the sidebar to start.',
  },
  status: {
    connected: 'Connected',
    connecting: 'Connecting',
    disconnected: 'Disconnected',
    vaultLocked: 'Vault: locked',
    vaultUnlocked: 'Vault: unlocked',
  },
  vault: {
    setup: 'Create Master Password',
    unlock: 'Unlock Vault',
    masterPasswordPlaceholder: 'Enter master password',
    setupWarning:
      'There is no way to recover your master password. If you forget it, all stored credentials will be permanently lost.',
    setupConfirm: 'I understand. There is no recovery.',
  },
} as const;
