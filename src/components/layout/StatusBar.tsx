import { Lock, Unlock } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';

const APP_VERSION = '0.1.0';

export function StatusBar() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const vaultUnlocked = useUiStore((s) => s.vaultUnlocked);

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
        <div
          className={cn(
            'flex items-center gap-1',
            vaultUnlocked ? 'text-status-connected' : 'text-fg-muted',
          )}
        >
          {vaultUnlocked ? <Unlock size={12} /> : <Lock size={12} />}
          <span>
            {vaultUnlocked
              ? strings.status.vaultUnlocked
              : strings.status.vaultLocked}
          </span>
        </div>
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
