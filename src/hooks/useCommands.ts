import { useEffect } from 'react';
import { useCommandPaletteStore, type Command } from '@/stores/command-store';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';
import { useVaultStore } from '@/stores/vault-store';

const ACTIONS: Record<string, () => void> = {
  'new-tab': () => useTabStore.getState().addTab(),
  'close-tab': () => {
    const { activeTabId, closeTab } = useTabStore.getState();
    if (activeTabId) closeTab(activeTabId);
  },
  'toggle-sidebar': () => useUiStore.getState().toggleSidebar(),
  'show-hosts': () => useUiStore.getState().setActivePanel('hosts'),
  'show-snippets': () => useUiStore.getState().setActivePanel('snippets'),
  'show-settings': () => useUiStore.getState().setActivePanel('settings'),
  'show-sftp': () => useUiStore.getState().setActivePanel('sftp'),
  'show-broadcast': () => useUiStore.getState().setActivePanel('broadcast'),
  'show-port-forward': () => useUiStore.getState().setActivePanel('port-forward'),
  'lock-vault': () => useVaultStore.getState().lock(),
  'show-vip': () => useUiStore.getState().setActivePanel('vip-access'),
  'show-p2p': () => useUiStore.getState().setActivePanel('p2p-sync'),
  'show-history': () => useUiStore.getState().setActivePanel('history'),
};

const DEFAULT_COMMANDS: Command[] = [
  { id: 'new-tab', label: 'New Terminal Tab', category: 'Terminal', shortcut: 'Ctrl+T', action: ACTIONS['new-tab']!, keywords: ['create', 'open'] },
  { id: 'close-tab', label: 'Close Current Tab', category: 'Terminal', shortcut: 'Ctrl+W', action: ACTIONS['close-tab']!, keywords: ['close', 'remove'] },
  { id: 'toggle-sidebar', label: 'Toggle Sidebar', category: 'View', shortcut: 'Ctrl+B', action: ACTIONS['toggle-sidebar']!, keywords: ['sidebar', 'panel', 'hide', 'show'] },
  { id: 'show-hosts', label: 'Show Hosts Panel', category: 'Navigation', action: ACTIONS['show-hosts']!, keywords: ['hosts', 'connections', 'servers'] },
  { id: 'show-snippets', label: 'Show Snippets Panel', category: 'Navigation', action: ACTIONS['show-snippets']!, keywords: ['snippets', 'commands', 'templates'] },
  { id: 'show-settings', label: 'Open Settings', category: 'Settings', shortcut: 'Ctrl+,', action: ACTIONS['show-settings']!, keywords: ['settings', 'preferences', 'config'] },
  { id: 'show-sftp', label: 'Open SFTP Browser', category: 'Tools', action: ACTIONS['show-sftp']!, keywords: ['sftp', 'files', 'transfer', 'browse'] },
  { id: 'show-broadcast', label: 'Open Broadcast Mode', category: 'Tools', action: ACTIONS['show-broadcast']!, keywords: ['broadcast', 'multi', 'send'] },
  { id: 'show-port-forward', label: 'Open Port Forwarding', category: 'Tools', action: ACTIONS['show-port-forward']!, keywords: ['port', 'forward', 'tunnel'] },
  { id: 'lock-vault', label: 'Lock Vault', category: 'Security', action: ACTIONS['lock-vault']!, keywords: ['lock', 'vault', 'secure'] },
  { id: 'show-vip', label: 'VIP Passwordless Access', category: 'Tools', action: ACTIONS['show-vip']!, keywords: ['vip', 'passwordless', 'key'] },
  { id: 'show-p2p', label: 'P2P Sync', category: 'Tools', action: ACTIONS['show-p2p']!, keywords: ['p2p', 'sync', 'mobile'] },
  { id: 'show-history', label: 'Command History', category: 'Tools', shortcut: 'Ctrl+H', action: ACTIONS['show-history']!, keywords: ['history', 'commands', 'log', 'previous'] },
];

export function useRegisterCommands() {
  const register = useCommandPaletteStore((s) => s.register);

  useEffect(() => {
    for (const cmd of DEFAULT_COMMANDS) {
      register(cmd);
    }
  }, [register]);
}
