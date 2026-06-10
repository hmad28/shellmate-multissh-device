import { useEffect } from 'react';
import { Search, Code2, Settings as SettingsIcon } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { HostList } from '@/components/hosts/HostList';
import { useHostStore } from '@/stores/host-store';
import { useUiStore, type ActivePanel } from '@/stores/ui-store';
import { useVaultStore } from '@/stores/vault-store';

export function Sidebar() {
  const { sidebarCollapsed, activePanel, setActivePanel } = useUiStore();
  const vaultUnlocked = useVaultStore((s) => s.unlocked);
  const loadAll = useHostStore((s) => s.loadAll);
  const searchQuery = useHostStore((s) => s.searchQuery);
  const setSearchQuery = useHostStore((s) => s.setSearchQuery);

  // Load hosts + groups when vault unlocks
  useEffect(() => {
    if (vaultUnlocked) {
      void loadAll();
    }
  }, [vaultUnlocked, loadAll]);

  if (sidebarCollapsed) return null;

  return (
    <aside
      className={cn(
        'flex w-60 shrink-0 flex-col',
        'border-r border-border bg-bg-sidebar',
      )}
      aria-label="Hosts sidebar"
    >
      <div className="border-b border-border-subtle p-3">
        <SearchInput value={searchQuery} onChange={setSearchQuery} />
      </div>

      <HostList />

      <div className="border-t border-border-subtle p-1">
        <PanelButton
          icon={<Code2 size={14} />}
          label={strings.sidebar.snippets}
          panel="snippets"
          active={activePanel === 'snippets'}
          onClick={() =>
            setActivePanel(activePanel === 'snippets' ? 'hosts' : 'snippets')
          }
        />
        <PanelButton
          icon={<SettingsIcon size={14} />}
          label={strings.sidebar.settings}
          panel="settings"
          active={activePanel === 'settings'}
          onClick={() =>
            setActivePanel(activePanel === 'settings' ? 'hosts' : 'settings')
          }
        />
      </div>
    </aside>
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
  panel: _panel,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  panel: ActivePanel;
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
