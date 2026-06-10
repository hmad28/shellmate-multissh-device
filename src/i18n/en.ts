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
    title: 'ShellMate Vault',
    setup: 'Create Master Password',
    setupSubtitle: 'Set a master password to encrypt your credentials.',
    unlock: 'Unlock Vault',
    unlockSubtitle: 'Enter your master password to continue.',
    masterPasswordPlaceholder: 'Enter master password',
    confirmPasswordPlaceholder: 'Confirm master password',
    setupWarningTitle: 'Important: There is no recovery',
    setupWarning:
      'If you forget your master password, all stored credentials will be permanently lost. Consider writing it down and storing it somewhere safe, or using a separate password manager.',
    setupConfirm: 'I understand the risks. There is no recovery.',
    minLengthHint:
      'At least 12 characters. Passphrases (e.g. four random words) are encouraged.',
    mismatch: 'Passwords do not match.',
    tooShort: 'Password must be at least 12 characters.',
    create: 'Create Vault',
    unlockButton: 'Unlock',
    cancel: 'Cancel',
    locking: 'Locking...',
    unlocking: 'Unlocking...',
    creating: 'Creating...',
  },
  quickConnect: {
    title: 'Quick Connect',
    subtitle: 'One-off SSH session for testing. Credentials are not saved.',
    hostnameLabel: 'Hostname',
    portLabel: 'Port',
    usernameLabel: 'Username',
    authLabel: 'Authentication',
    authPassword: 'Password',
    authKey: 'Private Key',
    passwordLabel: 'Password',
    privateKeyLabel: 'Private key (PEM)',
    privateKeyPlaceholder: '-----BEGIN OPENSSH PRIVATE KEY-----\n...',
    passphraseLabel: 'Key passphrase (optional)',
    connect: 'Connect',
    connecting: 'Connecting...',
  },
  terminal: {
    waitingForConnection: 'Waiting for connection...',
    sessionEnded: 'Session ended.',
  },
} as const;
