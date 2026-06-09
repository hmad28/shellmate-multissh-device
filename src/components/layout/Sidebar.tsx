import { Search, Plus, Code2, Settings as SettingsIcon } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useUiStore, type ActivePanel } from '@/stores/ui-store';

export function Sidebar() {
  const { sidebarCollapsed, activePanel, setActivePanel } = useUiStore();

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
        <SearchInput />
      </div>

      <nav className="flex-1 overflow-y-auto p-2" aria-label="Host groups">
        <HostGroupPlaceholder
          name={strings.sidebar.groups.production}
          count={0}
        />
        <HostGroupPlaceholder name={strings.sidebar.groups.staging} count={0} />
        <HostGroupPlaceholder
          name={strings.sidebar.groups.development}
          count={0}
        />

        <div className="mt-4 px-2 text-xs text-fg-subtle">
          {strings.sidebar.noHosts}
        </div>
      </nav>

      <div className="border-t border-border-subtle p-2">
        <button
          type="button"
          className={cn(
            'flex w-full items-center gap-2 rounded-md px-3 py-2 text-sm',
            'text-fg transition-colors hover:bg-bg-elevated',
          )}
          aria-label={strings.sidebar.addHost}
        >
          <Plus size={14} />
          <span>{strings.sidebar.addHost}</span>
        </button>
      </div>

      <div className="border-t border-border-subtle p-1">
        <PanelButton
          icon={<Code2 size={14} />}
          label={strings.sidebar.snippets}
          panel="snippets"
          active={activePanel === 'snippets'}
          onClick={() => setActivePanel('snippets')}
        />
        <PanelButton
          icon={<SettingsIcon size={14} />}
          label={strings.sidebar.settings}
          panel="settings"
          active={activePanel === 'settings'}
          onClick={() => setActivePanel('settings')}
        />
      </div>
    </aside>
  );
}

function SearchInput() {
  return (
    <div className="relative">
      <Search
        size={14}
        className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-fg-subtle"
        aria-hidden="true"
      />
      <input
        type="search"
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

function HostGroupPlaceholder({
  name,
  count,
}: {
  name: string;
  count: number;
}) {
  return (
    <div className="mb-2">
      <div
        className={cn(
          'flex items-center justify-between rounded px-2 py-1 text-xs font-medium uppercase tracking-wider',
          'text-fg-muted',
        )}
      >
        <span>{name}</span>
        <span className="text-fg-subtle">{count}</span>
      </div>
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
