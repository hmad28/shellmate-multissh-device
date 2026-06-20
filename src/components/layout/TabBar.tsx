import { X, FolderOpen, Network, Radio, TerminalSquare } from 'lucide-react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { tauri } from '@/lib/tauri';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';
import { useDragStore } from '@/stores/drag-store';
import type { ConnectionStatus, Tab } from '@/types';

export function TabBar() {
  const { tabs, activeTabId, closeTab, setActiveTab } = useTabStore();

  const sessionByTab = useSshStore((s) => s.sessionByTab);
  const unbind = useSshStore((s) => s.unbind);
  const setActivePanel = useUiStore((s) => s.setActivePanel);
  const setSftpSessionId = useUiStore((s) => s.setSftpSessionId);
  const setPortForwardSessionId = useUiStore((s) => s.setPortForwardSessionId);

  const activeSessionId = activeTabId ? sessionByTab[activeTabId] : null;

  const handleClose = (id: string) => {
    const sessionId = sessionByTab[id];
    if (sessionId) {
      void tauri.ssh.disconnect(sessionId).catch(() => {});
      unbind(id);
    }
    closeTab(id);
  };

  const handleOpenSftp = () => {
    if (activeSessionId) {
      setSftpSessionId(activeSessionId);
      setActivePanel('sftp');
    }
  };

  const handleOpenPortForward = () => {
    if (activeSessionId) {
      setPortForwardSessionId(activeSessionId);
      setActivePanel('port-forward');
    }
  };

  const handleOpenBroadcast = () => {
    setActivePanel('broadcast');
  };

  return (
    <div
      role="tablist"
      aria-label="Terminal sessions"
      data-drop-zone="tabbar"
      className={cn(
        'flex h-11 shrink-0 items-stretch',
        'border-b border-border bg-bg-sidebar',
      )}
    >
      <div className="flex flex-1 items-end overflow-x-auto px-2 pt-1">
        {tabs.map((tab) => (
          <TabButton
            key={tab.id}
            tab={tab}
            active={tab.id === activeTabId}
            onSelect={() => setActiveTab(tab.id)}
            onClose={() => handleClose(tab.id)}
          />
        ))}
      </div>

      {activeSessionId && (
        <div className="flex items-center gap-1 border-l border-border-subtle px-2">
          <button
            type="button"
            onClick={handleOpenSftp}
            aria-label="Open SFTP Browser"
            title="SFTP Browser"
            className={cn(
              'flex h-8 w-8 items-center justify-center rounded-md',
              'text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg',
            )}
          >
            <FolderOpen size={14} />
          </button>
          <button
            type="button"
            onClick={handleOpenPortForward}
            aria-label="Port Forwarding"
            title="Port Forwarding"
            className={cn(
              'flex h-8 w-8 items-center justify-center rounded-md',
              'text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg',
            )}
          >
            <Network size={14} />
          </button>
          <button
            type="button"
            onClick={handleOpenBroadcast}
            aria-label="Broadcast Mode"
            title="Broadcast Mode"
            className={cn(
              'flex h-8 w-8 items-center justify-center rounded-md',
              'text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg',
            )}
          >
            <Radio size={14} />
          </button>
        </div>
      )}
    </div>
  );
}

function TabButton({
  tab,
  active,
  onSelect,
  onClose,
}: {
  tab: Tab;
  active: boolean;
  onSelect: () => void;
  onClose: () => void;
}) {
  const dragId = useDragStore((s) => s.dragId);
  const dragType = useDragStore((s) => s.dragType);
  const hoveredZoneId = useDragStore((s) => s.hoveredZoneId);
  const hoveredZoneType = useDragStore((s) => s.hoveredZoneType);

  const isHovered =
    hoveredZoneType === 'tab' &&
    hoveredZoneId === tab.id &&
    !(dragType === 'tab' && dragId === tab.id);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return; // Left click only
    const startX = e.clientX;
    const startY = e.clientY;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaX = moveEvent.clientX - startX;
      const deltaY = moveEvent.clientY - startY;
      if (Math.sqrt(deltaX * deltaX + deltaY * deltaY) > 5) {
        useDragStore
          .getState()
          .startDrag(
            'tab',
            tab.id,
            tab.label,
            moveEvent.clientX,
            moveEvent.clientY,
          );
        window.removeEventListener('mousemove', handleMouseMove);
      }
    };

    const handleMouseUp = () => {
      window.removeEventListener('mousemove', handleMouseMove);
      window.removeEventListener('mouseup', handleMouseUp);
    };

    window.addEventListener('mousemove', handleMouseMove);
    window.addEventListener('mouseup', handleMouseUp);
  };

  return (
    <div
      role="tab"
      aria-selected={active}
      tabIndex={active ? 0 : -1}
      data-drop-zone="tab"
      data-tab-id={tab.id}
      onMouseDown={handleMouseDown}
      onClick={onSelect}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onSelect();
        }
      }}
      className={cn(
        'group relative mb-1 flex h-9 max-w-64 cursor-pointer items-center gap-2 rounded-md px-3',
        'border border-transparent text-sm transition-colors',
        active
          ? 'border-border-subtle bg-bg text-fg shadow-sm'
          : 'text-fg-muted hover:bg-bg-elevated hover:text-fg',
        isHovered && 'border-accent/50 bg-accent/15 font-medium',
      )}
    >
      <TerminalSquare size={14} className="shrink-0 text-fg-subtle" />
      <StatusDot status={tab.status} />
      <span className="min-w-20 flex-1 truncate">{tab.label}</span>
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
        aria-label={`${strings.tabs.closeTab}: ${tab.label}`}
        className={cn(
          'flex h-6 w-6 shrink-0 items-center justify-center rounded',
          'text-fg-subtle hover:bg-border-strong hover:text-fg',
          'opacity-0 transition-opacity group-hover:opacity-100',
          active && 'opacity-100',
        )}
      >
        <X size={12} />
      </button>
    </div>
  );
}

function StatusDot({ status }: { status: ConnectionStatus }) {
  const label =
    status === 'connected'
      ? strings.status.connected
      : status === 'connecting'
        ? strings.status.connecting
        : strings.status.disconnected;

  const symbol =
    status === 'connected' ? '●' : status === 'connecting' ? '◐' : '○';

  return (
    <span
      className={cn(
        'select-none text-xs',
        status === 'connected' && 'text-status-connected',
        status === 'connecting' && 'text-status-connecting',
        status === 'disconnected' && 'text-status-disconnected',
      )}
      aria-label={label}
      title={label}
    >
      {symbol}
    </span>
  );
}
