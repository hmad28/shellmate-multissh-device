import { useUiStore } from '@/stores/ui-store';

type MobileTab = 'hosts' | 'terminal' | 'snippets' | 'settings';

const tabs: { id: MobileTab; label: string; icon: string }[] = [
  { id: 'hosts', label: 'Hosts', icon: '🏠' },
  { id: 'terminal', label: 'Terminal', icon: '⌨️' },
  { id: 'snippets', label: 'Snippets', icon: '📋' },
  { id: 'settings', label: 'Settings', icon: '⚙️' },
];

export function BottomNav() {
  const { activePanel, setActivePanel } = useUiStore();

  const currentTab: MobileTab =
    activePanel === 'hosts' || activePanel === 'snippets' || activePanel === 'settings'
      ? activePanel
      : 'terminal';

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 flex h-14 items-center justify-around border-t border-[var(--color-border)] bg-[var(--color-surface)] safe-area-inset-bottom">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => {
            if (tab.id === 'terminal') {
              setActivePanel('hosts');
            } else {
              setActivePanel(tab.id);
            }
          }}
          className={`flex flex-1 flex-col items-center justify-center gap-0.5 py-1 text-xs transition-colors ${
            currentTab === tab.id
              ? 'text-[var(--color-primary)]'
              : 'text-[var(--color-muted)]'
          }`}
        >
          <span className="text-lg">{tab.icon}</span>
          <span>{tab.label}</span>
        </button>
      ))}
    </nav>
  );
}
