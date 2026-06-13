import { useEffect } from 'react';
import { VaultGate } from '@/components/vault/VaultGate';
import { AppLayout } from '@/components/layout/AppLayout';
import { DragGhost } from '@/components/layout/DragGhost';
import { ErrorBoundary } from '@/components/ui/ErrorBoundary';
import { CommandPalette } from '@/components/ui/CommandPalette';
import { UpdateToast } from '@/components/updates/UpdateToast';
import { useAutoLock } from '@/hooks/useAutoLock';
import { useCustomDragDrop } from '@/hooks/useCustomDragDrop';
import { useRegisterCommands } from '@/hooks/useCommands';
import { useKeyboardShortcuts } from '@/hooks/useKeyboardShortcuts';
import { useSettingsStore } from '@/stores/settings-store';
import { useVaultStore } from '@/stores/vault-store';
import { useTabStore } from '@/stores/tab-store';
import { usePaneStore } from '@/stores/pane-store';
import { useSshStore } from '@/stores/ssh-store';
import { useSftpStore } from '@/stores/sftp-store';
import { usePortForwardStore } from '@/stores/port-forward-store';
import { useUiStore } from '@/stores/ui-store';
import { tauri } from '@/lib/tauri';

export default function App() {
  // Settings (theme, terminal prefs) are public — load on app start, before vault.
  const loadSettings = useSettingsStore((s) => s.load);
  const settingsLoaded = useSettingsStore((s) => s.loaded);
  useEffect(() => {
    if (!settingsLoaded) void loadSettings();
  }, [settingsLoaded, loadSettings]);

  // Idle auto-lock poll runs whenever vault is unlocked.
  useAutoLock();

  // Custom mouse-based drag and drop engine.
  useCustomDragDrop();

  // Register default command palette commands.
  useRegisterCommands();

  // Register global keyboard shortcuts.
  useKeyboardShortcuts();

  // Reset all session/tab/pane state whenever the vault lock state changes.
  const unlocked = useVaultStore((s) => s.unlocked);
  useEffect(() => {
    if (!unlocked) {
      const sessionIds = Object.values(useSshStore.getState().sessionByTab);
      for (const sid of sessionIds) {
        void tauri.ssh.disconnect(sid).catch(() => {});
      }
    }

    useTabStore.setState({ tabs: [], activeTabId: null });
    usePaneStore.setState({
      root: { type: 'leaf', id: 'pane-1', tabIds: [], activeTabId: null },
      activePaneId: 'pane-1',
      fullscreenPaneId: null,
    });
    useSshStore.setState({ sessionByTab: {}, pendingAttempts: {} });
    useSftpStore.setState({ browsers: {}, activeBrowserId: null });
    usePortForwardStore.setState({ forwards: {} });
    useUiStore.setState({
      activePanel: 'hosts',
      sftpSessionId: null,
      portForwardSessionId: null,
      vaultUnlocked: unlocked,
    });
  }, [unlocked]);

  return (
    <>
      <ErrorBoundary>
        <VaultGate>
          <AppLayout />
        </VaultGate>
      </ErrorBoundary>
      <CommandPalette />
      <UpdateToast />
      <DragGhost />
    </>
  );
}
