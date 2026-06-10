import { useState } from 'react';
import { strings } from '@/i18n/en';
import { QuickConnect } from '@/components/connect/QuickConnect';
import { SettingsDialog } from '@/components/settings/SettingsDialog';
import { SnippetPanel } from '@/components/snippets/SnippetPanel';
import { Terminal } from '@/components/terminal/Terminal';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';

/**
 * ContentArea — renders the active panel:
 *   - 'hosts' (default): active terminal or QuickConnect form
 *   - 'snippets': SnippetPanel
 *   - 'settings': SettingsDialog (modal-style takeover; but we render
 *     it in a centered surface for keyboard nav; closing returns to hosts)
 */
export function ContentArea() {
  const activePanel = useUiStore((s) => s.activePanel);
  const setActivePanel = useUiStore((s) => s.setActivePanel);

  const [settingsOpen, setSettingsOpen] = useState(false);

  // Settings panel button → open dialog
  if (activePanel === 'settings' && !settingsOpen) {
    setSettingsOpen(true);
  }

  if (activePanel === 'snippets') {
    return (
      <main className="flex flex-1 overflow-hidden bg-bg">
        <SnippetPanel />
      </main>
    );
  }

  return (
    <>
      <SettingsDialog
        open={settingsOpen}
        onClose={() => {
          setSettingsOpen(false);
          if (activePanel === 'settings') setActivePanel('hosts');
        }}
      />
      <HostsContent />
    </>
  );
}

function HostsContent() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const sessionByTab = useSshStore((s) => s.sessionByTab);

  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;
  const activeSessionId = activeTabId
    ? (sessionByTab[activeTabId] ?? null)
    : null;

  if (!activeTab) {
    return (
      <main className="flex flex-1 items-stretch overflow-hidden bg-bg">
        <div className="m-auto w-full max-w-md">
          <QuickConnect />
        </div>
      </main>
    );
  }

  if (!activeSessionId) {
    return (
      <main className="flex flex-1 items-center justify-center bg-bg p-6 text-fg-muted">
        <p className="text-sm">{strings.terminal.waitingForConnection}</p>
      </main>
    );
  }

  return (
    <main className="relative flex flex-1 overflow-hidden bg-bg">
      {tabs.map((tab) => {
        const sid = sessionByTab[tab.id];
        if (!sid) return null;
        const isActive = tab.id === activeTabId;
        return (
          <div
            key={tab.id}
            className="absolute inset-0"
            style={{
              visibility: isActive ? 'visible' : 'hidden',
              pointerEvents: isActive ? 'auto' : 'none',
            }}
          >
            <Terminal tabId={tab.id} sessionId={sid} />
          </div>
        );
      })}
    </main>
  );
}
