import { useEffect } from 'react';
import {
  Search,
  Code2,
  Smartphone,
  Settings as SettingsIcon,
  PanelLeftClose,
  PanelLeftOpen,
  Activity,
  Box,
} from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { HostList } from '@/components/hosts/HostList';
import { DiscoveredHostList } from '@/components/hosts/DiscoveredHostList';
import { useHostStore } from '@/stores/host-store';
import { useUiStore } from '@/stores/ui-store';
import { useVaultStore } from '@/stores/vault-store';

import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';

export function Sidebar() {
  const { sidebarCollapsed, activePanel, setActivePanel, toggleSidebar } =
    useUiStore();
  const vaultUnlocked = useVaultStore((s) => s.unlocked);
  const loadAll = useHostStore((s) => s.loadAll);
  const searchQuery = useHostStore((s) => s.searchQuery);
  const setSearchQuery = useHostStore((s) => s.setSearchQuery);

  const handleOpenLocalTerminal = async () => {
    const tabId = useTabStore.getState().addTab({
      label: 'Local Terminal',
      hostId: 'local',
    });
    try {
      await useSshStore.getState().connectLocal(tabId);
    } catch (err) {
      console.error('Failed to connect to local terminal', err);
    }
  };

  useEffect(() => {
    if (vaultUnlocked) void loadAll();
  }, [vaultUnlocked, loadAll]);

  return (
    <>
      <aside
        className={cn(
          'flex shrink-0 flex-col overflow-hidden',
          'border-r border-border bg-bg-sidebar',
          'transition-[width] duration-200 ease-in-out',
          sidebarCollapsed ? 'w-0 border-r-0' : 'w-60',
        )}
        aria-label="Hosts sidebar"
      >
        <div className="flex h-full w-60 flex-col">
          <div className="flex flex-col gap-2 border-b border-border-subtle p-3">
            <SearchInput value={searchQuery} onChange={setSearchQuery} />
            <button
              type="button"
              onClick={() => setActivePanel('p2p-sync')}
              className={cn(
                'flex w-full items-center justify-center gap-1.5 rounded-md py-1.5 text-xs font-semibold shadow-sm transition-all duration-200 active:scale-95',
                activePanel === 'p2p-sync'
                  ? 'bg-[var(--color-accent-hover)] text-white'
                  : 'bg-[var(--color-accent)] text-white hover:bg-[var(--color-accent-hover)]',
              )}
            >
              <Smartphone size={12} />
              <span>Sync Phone</span>
            </button>
            <button
              type="button"
              onClick={handleOpenLocalTerminal}
              className="flex w-full items-center justify-center gap-1.5 rounded-md border border-border-strong bg-bg-elevated py-1.5 text-xs font-semibold text-fg transition-all duration-200 hover:bg-bg-panel active:scale-95"
            >
              <Activity size={12} />
              <span>Open Local Terminal</span>
            </button>
          </div>

          <DiscoveredHostList />
          <HostList />

          <div className="mt-auto border-t border-border-subtle p-1">
            <PanelButton
              icon={<Code2 size={14} />}
              label={strings.sidebar.snippets}
              active={activePanel === 'snippets'}
              onClick={() =>
                setActivePanel(
                  activePanel === 'snippets' ? 'hosts' : 'snippets',
                )
              }
            />
            <PanelButton
              icon={<SettingsIcon size={14} />}
              label={strings.sidebar.settings}
              active={activePanel === 'settings'}
              onClick={() =>
                setActivePanel(
                  activePanel === 'settings' ? 'hosts' : 'settings',
                )
              }
            />
            <PanelButton
              icon={<Activity size={14} />}
              label="Server Stats"
              active={activePanel === 'server-stats'}
              onClick={() =>
                setActivePanel(
                  activePanel === 'server-stats' ? 'hosts' : 'server-stats',
                )
              }
            />
            <PanelButton
              icon={<Box size={14} />}
              label="Docker"
              active={activePanel === 'docker'}
              onClick={() =>
                setActivePanel(activePanel === 'docker' ? 'hosts' : 'docker')
              }
            />

            <button
              type="button"
              onClick={toggleSidebar}
              aria-label="Collapse sidebar"
              title="Collapse sidebar (Ctrl+B)"
              className={cn(
                'flex w-full items-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors',
                'text-fg-muted hover:bg-bg-elevated hover:text-fg',
              )}
            >
              <PanelLeftClose size={14} />
              <span>Collapse</span>
            </button>
          </div>
        </div>
      </aside>

      {/* Floating expand button — only visible when collapsed, on the left edge */}
      {sidebarCollapsed && (
        <button
          type="button"
          onClick={toggleSidebar}
          aria-label="Expand sidebar"
          title="Expand sidebar (Ctrl+B)"
          className={cn(
            'fixed left-0 top-1/2 z-50 -translate-y-1/2',
            'flex h-10 w-6 items-center justify-center rounded-r-md',
            'bg-bg-sidebar text-fg-muted shadow-md',
            'border border-l-0 border-border',
            'hover:bg-bg-elevated hover:text-fg',
            'transition-colors',
          )}
        >
          <PanelLeftOpen size={14} />
        </button>
      )}
    </>
  );
}

function SearchInput({
  value,
  onChange,
}: {
  value: string;
  onChange: (v: string) => void;
}) {
  return (
    <div className="relative">
      <Search
        size={14}
        className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-subtle"
        aria-hidden="true"
      />
      <input
        type="search"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={strings.sidebar.searchPlaceholder}
        aria-label={strings.sidebar.searchPlaceholder}
        className={cn(
          'w-full rounded-md bg-bg-elevated py-1.5 pl-8 pr-2 text-sm',
          'text-fg placeholder:text-fg-subtle',
          'border border-border-subtle',
          'focus:border-accent focus:outline-none focus:ring-1 focus:ring-accent',
        )}
      />
    </div>
  );
}

function PanelButton({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-current={active ? 'page' : undefined}
      className={cn(
        'flex w-full items-center gap-2 rounded-md px-3 py-1.5 text-sm transition-colors',
        active
          ? 'bg-bg-elevated text-fg'
          : 'text-fg-muted hover:bg-bg-elevated hover:text-fg',
      )}
    >
      {icon}
      <span>{label}</span>
    </button>
  );
}
