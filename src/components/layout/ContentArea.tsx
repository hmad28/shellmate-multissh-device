import { strings } from '@/i18n/en';
import { useTabStore } from '@/stores/tab-store';

/**
 * ContentArea — placeholder for the active tab's terminal / SFTP / settings.
 * Real terminal will be wired in Phase 2.
 */
export function ContentArea() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;

  return (
    <main className="flex flex-1 items-center justify-center bg-bg p-6 text-fg-muted">
      {activeTab ? (
        <div className="text-center">
          <p className="font-mono text-sm text-fg">
            Tab: <span className="text-accent">{activeTab.label}</span>
          </p>
          <p className="mt-2 text-xs text-fg-subtle">
            Terminal will render here (Phase 2)
          </p>
        </div>
      ) : (
        <p className="max-w-md text-center text-sm">{strings.tabs.noTabs}</p>
      )}
    </main>
  );
}
