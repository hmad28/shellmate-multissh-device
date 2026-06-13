import { useEffect, useState } from 'react';
import { Lock, Unlock, GitBranch } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useTabStore } from '@/stores/tab-store';
import { useVaultStore } from '@/stores/vault-store';
import { tauri } from '@/lib/tauri';

const APP_VERSION = '0.1.0';

interface GitInfo {
  branch: string | null;
  hasChanges: boolean;
  ahead: number;
  behind: number;
}

export function StatusBar() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const vaultUnlocked = useVaultStore((s) => s.unlocked);
  const lock = useVaultStore((s) => s.lock);
  const [gitInfo, setGitInfo] = useState<GitInfo | null>(null);

  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;
  const leftLabel = activeTab
    ? `${activeTab.label} — ${labelForStatus(activeTab.status)}`
    : strings.app.ready;

  useEffect(() => {
    let mounted = true;
    const poll = async () => {
      try {
        const info = await tauri.git.getInfo();
        if (mounted) setGitInfo(info);
      } catch {
        if (mounted) setGitInfo(null);
      }
    };
    poll();
    const interval = setInterval(poll, 10000);
    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, []);

  return (
    <footer
      className={cn(
        'flex h-6 shrink-0 items-center justify-between',
        'border-t border-border bg-bg-sidebar px-3 text-xs text-fg-muted',
      )}
      aria-label="Status bar"
    >
      <div className="flex items-center gap-3">
        <div aria-live="polite" className="truncate">
          {leftLabel}
        </div>
        {gitInfo?.branch && (
          <div className="flex items-center gap-1 text-fg-subtle">
            <GitBranch size={12} />
            <span>{gitInfo.branch}</span>
            {gitInfo.hasChanges && (
              <span className="text-status-connecting">*</span>
            )}
            {gitInfo.ahead > 0 && (
              <span className="text-status-connecting">↑{gitInfo.ahead}</span>
            )}
            {gitInfo.behind > 0 && (
              <span className="text-status-disconnected">↓{gitInfo.behind}</span>
            )}
          </div>
        )}
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
