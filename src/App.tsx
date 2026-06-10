import { useEffect } from 'react';
import { VaultGate } from '@/components/vault/VaultGate';
import { AppLayout } from '@/components/layout/AppLayout';
import { useAutoLock } from '@/hooks/useAutoLock';
import { useSettingsStore } from '@/stores/settings-store';
import { useVaultStore } from '@/stores/vault-store';

export default function App() {
  // Settings (theme, terminal prefs) are public — load on app start, before vault.
  const loadSettings = useSettingsStore((s) => s.load);
  const settingsLoaded = useSettingsStore((s) => s.loaded);
  useEffect(() => {
    if (!settingsLoaded) void loadSettings();
  }, [settingsLoaded, loadSettings]);

  // Idle auto-lock poll runs whenever vault is unlocked.
  useAutoLock();

  return (
    <VaultGate>
      <AppLayout />
    </VaultGate>
  );
}

// Suppress unused import (referenced via hook implicitly when vault state reads)
void useVaultStore;
