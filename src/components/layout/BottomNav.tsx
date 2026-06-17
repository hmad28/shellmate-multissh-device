import type { ReactNode } from 'react';
import { useUiStore } from '@/stores/ui-store';
import { Code2, Home, Settings, TerminalSquare } from 'lucide-react';

type MobileTab = 'hosts' | 'terminal' | 'snippets' | 'settings';

const tabs: { id: MobileTab; label: string; icon: ReactNode }[] = [
  { id: 'hosts', label: 'Hosts', icon: <Home size={18} /> },
  { id: 'terminal', label: 'Terminal', icon: <TerminalSquare size={18} /> },
  { id: 'snippets', label: 'Snippets', icon: <Code2 size={18} /> },
  { id: 'settings', label: 'Settings', icon: <Settings size={18} /> },
];

export function BottomNav() {
  const { activePanel, setActivePanel } = useUiStore();

  const currentTab: MobileTab =
    activePanel === 'hosts' ||
    activePanel === 'snippets' ||
    activePanel === 'settings'
      ? activePanel
      : 'terminal';

  return (
    <nav className="fixed left-3 right-3 z-50 flex h-[var(--mobile-bottom-nav-height)] items-center justify-around rounded-xl border border-border bg-bg-sidebar shadow-[0_-8px_24px_rgba(0,0,0,0.35)] [bottom:var(--mobile-system-bottom)]">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          onClick={() => {
            if (tab.id === 'terminal') {
              setActivePanel('terminal');
            } else {
              setActivePanel(tab.id);
            }
          }}
          className={`flex h-full min-w-0 flex-1 flex-col items-center justify-center gap-0.5 py-1 text-[11px] transition-colors ${
            currentTab === tab.id
              ? 'text-[var(--color-accent)]'
              : 'text-[var(--color-fg-muted)]'
          }`}
        >
          <span aria-hidden="true">{tab.icon}</span>
          <span className="truncate">{tab.label}</span>
        </button>
      ))}
    </nav>
  );
}
