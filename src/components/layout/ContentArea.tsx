import { strings } from '@/i18n/en';
import { QuickConnect } from '@/components/connect/QuickConnect';
import { Terminal } from '@/components/terminal/Terminal';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';

/**
 * ContentArea — renders the active tab's terminal or, if no tab is active,
 * the QuickConnect form for one-off SSH sessions.
 */
export function ContentArea() {
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

  // Render ALL bound terminals, hiding inactive ones via CSS so xterm doesn't
  // tear down state when switching tabs. Each Terminal is keyed by sessionId
  // so it remounts only when the underlying session id changes.
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
