import { Lock, Unlock } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useTabStore } from '@/stores/tab-store';
import { useVaultStore } from '@/stores/vault-store';

const APP_VERSION = '0.1.0';

export function StatusBar() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const vaultUnlocked = useVaultStore((s) => s.unlocked);
  const lock = useVaultStore((s) => s.lock);

  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;
  const leftLabel = activeTab
    ? `${activeTab.label} — ${labelForStatus(activeTab.status)}`
    : strings.app.ready;

  return (
    <footer
      className={cn(
        'flex h-6 shrink-0 items-center justify-between',
        'border-t border-border bg-bg-sidebar px-3 text-xs text-fg-muted',
      )}
      aria-label="Status bar"
    >
      <div aria-live="polite" className="truncate">
        {leftLabel}
      </div>

      <div className="flex items-center gap-3">
        <button
          type="button"
          onClick={() => void lock()}
          aria-label={
            vaultUnlocked
              ? strings.status.vaultUnlocked
              : strings.status.vaultLocked
          }
          className={cn(
            'flex items-center gap-1 rounded px-1 transition-colors',
            vaultUnlocked
              ? 'text-status-connected hover:bg-bg-elevated'
              : 'text-fg-muted',
          )}
          disabled={!vaultUnlocked}
        >
          {vaultUnlocked ? <Unlock size={12} /> : <Lock size={12} />}
          <span>
            {vaultUnlocked
              ? strings.status.vaultUnlocked
              : strings.status.vaultLocked}
          </span>
        </button>
        <span className="text-fg-subtle">v{APP_VERSION}</span>
      </div>
    </footer>
  );
}

function labelForStatus(
  status: 'connected' | 'connecting' | 'disconnected',
): string {
  switch (status) {
    case 'connected':
      return strings.status.connected;
    case 'connecting':
      return strings.status.connecting;
    case 'disconnected':
      return strings.status.disconnected;
  }
}
