import React, { useState, useEffect, useCallback, useRef } from 'react';
import { X, Maximize2, Minimize2, Smartphone } from 'lucide-react';
import { QuickConnect } from '@/components/connect/QuickConnect';
import { SettingsDialog } from '@/components/settings/SettingsDialog';
import { SnippetPanel } from '@/components/snippets/SnippetPanel';
import { SftpBrowser } from '@/components/sftp/SftpBrowser';
import { PortForwardPanel } from '@/components/port-forward/PortForwardPanel';
import { BroadcastModePanel } from '@/components/terminal/BroadcastModePanel';
import { P2pSyncPanel } from '@/components/sync/P2pSyncPanel';
import { CommandHistoryPanel } from '@/components/history/CommandHistoryPanel';
import { ServerStatsPanel } from '@/components/server/ServerStatsPanel';
import { DockerPanel } from '@/components/server/DockerPanel';
import { Terminal } from '@/components/terminal/Terminal';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';
import { tauri } from '@/lib/tauri';
import {
  usePaneStore,
  getAllLeaves,
  findNode,
  type PaneNode,
  type LeafNode,
  type SplitNode,
} from '@/stores/pane-store';
import { useVaultStore } from '@/stores/vault-store';
import { useHostStore } from '@/stores/host-store';
import { useDragStore } from '@/stores/drag-store';
import { cn } from '@/lib/cn';
import type { Tab } from '@/types';

export function ContentArea() {
  const activePanel = useUiStore((s) => s.activePanel);
  const setActivePanel = useUiStore((s) => s.setActivePanel);
  const sftpSessionId = useUiStore((s) => s.sftpSessionId);
  const portForwardSessionId = useUiStore((s) => s.portForwardSessionId);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [snippetOpen, setSnippetOpen] = useState(false);
  const [p2pOpen, setP2pOpen] = useState(false);
  const [historyOpen, setHistoryOpen] = useState(false);

  useEffect(() => {
    if (activePanel === 'settings') setSettingsOpen(true);
    if (activePanel === 'snippets') setSnippetOpen(true);
    if (activePanel === 'p2p-sync') setP2pOpen(true);
    if (activePanel === 'history') setHistoryOpen(true);
  }, [activePanel]);

  const isHosts = activePanel === 'hosts';

  return (
    <>
      <div
        className="flex flex-1 overflow-hidden"
        style={{ display: isHosts ? 'flex' : 'none' }}
      >
        <HostsContent />
      </div>

      {activePanel === 'sftp' && sftpSessionId && (
        <main className="flex flex-1 overflow-hidden bg-bg">
          <SftpBrowser
            sessionId={sftpSessionId}
            onClose={() => setActivePanel('hosts')}
          />
        </main>
      )}
      {activePanel === 'port-forward' && portForwardSessionId && (
        <main className="flex flex-1 overflow-hidden bg-bg p-6">
          <PortForwardPanel sessionId={portForwardSessionId} />
        </main>
      )}
      {activePanel === 'broadcast' && (
        <main className="flex flex-1 overflow-hidden bg-bg p-6">
          <BroadcastModePanel onClose={() => setActivePanel('hosts')} />
        </main>
      )}

      {activePanel === 'server-stats' && (
        <main className="flex flex-1 overflow-hidden bg-bg">
          <ServerStatsPanelWrapper />
        </main>
      )}
      {activePanel === 'docker' && (
        <main className="flex flex-1 overflow-hidden bg-bg">
          <DockerPanelWrapper />
        </main>
      )}

      <SettingsDialog
        open={settingsOpen}
        onClose={() => {
          setSettingsOpen(false);
          if (activePanel === 'settings') setActivePanel('hosts');
        }}
      />
      <PopupDialog
        open={snippetOpen}
        title="Snippets"
        onClose={() => {
          setSnippetOpen(false);
          if (activePanel === 'snippets') setActivePanel('hosts');
        }}
      >
        <SnippetPanel />
      </PopupDialog>
      <PopupDialog
        open={p2pOpen}
        title="Sync Phone"
        className="max-w-4xl max-h-[90vh]"
        onClose={() => {
          setP2pOpen(false);
          if (activePanel === 'p2p-sync') setActivePanel('hosts');
        }}
      >
        <P2pSyncPanel />
      </PopupDialog>
      <PopupDialog
        open={historyOpen}
        title="Command History"
        onClose={() => {
          setHistoryOpen(false);
          if (activePanel === 'history') setActivePanel('hosts');
        }}
      >
        <CommandHistoryPanel onClose={() => setActivePanel('hosts')} />
      </PopupDialog>
    </>
  );
}

function PopupDialog({
  open,
  title,
  onClose,
  children,
  className,
}: {
  open: boolean;
  title: string;
  onClose: () => void;
  children: React.ReactNode;
  className?: string;
}) {
  if (!open) return null;
  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={onClose}
    >
      <div
        className={cn(
          "relative max-h-[80vh] w-full max-w-lg overflow-auto rounded-lg border border-border bg-bg shadow-xl",
          className
        )}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-border px-4 py-3">
          <h2 className="text-sm font-medium text-fg">{title}</h2>
          <button
            onClick={onClose}
            className="text-lg leading-none text-fg-muted hover:text-fg"
          >
            &times;
          </button>
        </div>
        <div className="p-4">{children}</div>
      </div>
    </div>
  );
}

// --- HostsContent: orchestrates pane tree rendering ---

function HostsContent() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const root = usePaneStore((s) => s.root);
  const fullscreenPaneId = usePaneStore((s) => s.fullscreenPaneId);
  const ensureTabInPane = usePaneStore((s) => s.ensureTabInPane);
  const setActiveTabInPane = usePaneStore((s) => s.setActiveTabInPane);
  const vaultUnlocked = useVaultStore((s) => s.unlocked);
  const hoveredZoneType = useDragStore((s) => s.hoveredZoneType);
  const setActivePanel = useUiStore((s) => s.setActivePanel);
  const isEmptyHovered = hoveredZoneType === 'empty';

  const activeTab = tabs.find((t) => t.id === activeTabId) ?? null;

  useEffect(() => {
    if (!vaultUnlocked) {
      const leaf: LeafNode = {
        type: 'leaf',
        id: 'pane-1',
        tabIds: [],
        activeTabId: null,
      };
      usePaneStore.setState({
        root: leaf,
        activePaneId: leaf.id,
        fullscreenPaneId: null,
      });
    }
  }, [vaultUnlocked]);

  useEffect(() => {
    for (const tab of tabs) {
      ensureTabInPane(tab.id);
    }
  }, [tabs, ensureTabInPane]);

  useEffect(() => {
    if (!activeTabId) return;
    const paneId = usePaneStore.getState().getPaneForTab(activeTabId);
    if (paneId) setActiveTabInPane(paneId, activeTabId);
  }, [activeTabId, setActiveTabInPane]);

  if (!activeTab) {
    return (
      <main
        data-drop-zone="empty"
        className={cn(
          'flex flex-1 items-stretch overflow-hidden bg-bg transition-colors',
          isEmptyHovered && 'bg-accent/5 ring-2 ring-inset ring-accent',
        )}
      >
        <div className="m-auto flex w-full max-w-md flex-col gap-3">
          <button
            type="button"
            onClick={() => setActivePanel('p2p-sync')}
            className="border-accent/60 flex w-full items-center justify-center gap-2 rounded-md border bg-accent px-4 py-3 text-sm font-semibold text-white shadow-sm transition-colors hover:bg-accent-hover"
          >
            <Smartphone size={16} />
            <span>Sync Device</span>
          </button>
          <QuickConnect />
        </div>
      </main>
    );
  }

  // Fullscreen mode: show only one pane
  if (fullscreenPaneId) {
    const pane = findNode(root, fullscreenPaneId);
    if (pane && pane.type === 'leaf') {
      return (
        <main className="flex flex-1 overflow-hidden bg-bg">
          <PaneView pane={pane} />
        </main>
      );
    }
  }

  return (
    <main className="flex flex-1 overflow-hidden bg-bg">
      <PaneRenderer node={root} />
    </main>
  );
}

// --- PaneRenderer: recursively renders the pane tree ---

function PaneRenderer({ node }: { node: PaneNode }) {
  if (node.type === 'leaf') return <PaneView pane={node} />;
  return <SplitView node={node} />;
}

function SplitView({ node }: { node: SplitNode }) {
  const updateSizes = usePaneStore((s) => s.updateSizes);
  const isH = node.direction === 'horizontal';
  const containerRef = useRef<HTMLDivElement>(null);
  const [dragIdx, setDragIdx] = useState<number | null>(null);
  const dragRef = useRef<{
    idx: number;
    start: number;
    sizes: number[];
  } | null>(null);

  const onResizeStart = useCallback(
    (idx: number, e: React.MouseEvent) => {
      e.preventDefault();
      setDragIdx(idx);
      dragRef.current = {
        idx,
        start: isH ? e.clientX : e.clientY,
        sizes: [...node.sizes],
      };

      const onMove = (ev: MouseEvent) => {
        const d = dragRef.current;
        if (!d || !containerRef.current) return;
        const rect = containerRef.current.getBoundingClientRect();
        const total = isH ? rect.width : rect.height;
        const delta =
          (((isH ? ev.clientX : ev.clientY) - d.start) / total) * 100;
        const ns = [...d.sizes];
        const l = ns[d.idx] ?? 0;
        const r = ns[d.idx + 1] ?? 0;
        const clamped = Math.max(-l + 10, Math.min(delta, r - 10));
        ns[d.idx] = l + clamped;
        ns[d.idx + 1] = r - clamped;
        updateSizes(node.id, ns);
      };

      const onUp = () => {
        setDragIdx(null);
        dragRef.current = null;
        window.removeEventListener('mousemove', onMove);
        window.removeEventListener('mouseup', onUp);
      };
      window.addEventListener('mousemove', onMove);
      window.addEventListener('mouseup', onUp);
    },
    [isH, node.id, node.sizes, updateSizes],
  );

  return (
    <div
      ref={containerRef}
      className={cn(
        'flex flex-1 overflow-hidden',
        isH ? 'flex-row' : 'flex-col',
      )}
    >
      {node.children.map((child, idx) => (
        <SplitItem
          key={child.id}
          flex={node.sizes[idx] ?? 50}
          isLast={idx === node.children.length - 1}
          isH={isH}
          dragIdx={dragIdx}
          idx={idx}
          onResizeStart={onResizeStart}
        >
          <PaneRenderer node={child} />
        </SplitItem>
      ))}
    </div>
  );
}

function SplitItem({
  children,
  flex,
  isLast,
  isH,
  dragIdx,
  idx,
  onResizeStart,
}: {
  children: React.ReactNode;
  flex: number;
  isLast: boolean;
  isH: boolean;
  dragIdx: number | null;
  idx: number;
  onResizeStart: (idx: number, e: React.MouseEvent) => void;
}) {
  return (
    <>
      <div className="flex overflow-hidden" style={{ flex: `${flex} 1 0%` }}>
        {children}
      </div>
      {!isLast && (
        <div
          onMouseDown={(e) => onResizeStart(idx, e)}
          className={cn(
            'shrink-0 bg-border-subtle transition-colors hover:bg-accent',
            isH ? 'w-[3px] cursor-col-resize' : 'h-[3px] cursor-row-resize',
            dragIdx === idx && 'bg-accent',
          )}
        />
      )}
    </>
  );
}

// --- PaneView: per-pane tab bar + terminal content + drop zones ---

function PaneView({ pane }: { pane: LeafNode }) {
  const tabs = useTabStore((s) => s.tabs);
  const sessionByTab = useSshStore((s) => s.sessionByTab);
  const setActivePane = usePaneStore((s) => s.setActivePane);
  const setActiveTabInPane = usePaneStore((s) => s.setActiveTabInPane);
  const closeTabInPane = usePaneStore((s) => s.closeTabInPane);
  const root = usePaneStore((s) => s.root);
  const leafCount = getAllLeaves(root).length;
  // Subscribe to fullscreenPaneId via a separate child to avoid re-rendering
  // the whole pane tree when only the fullscreen flag changes.
  const fullscreenPaneId = usePaneStore((s) => s.fullscreenPaneId);
  const toggleFullscreen = usePaneStore((s) => s.toggleFullscreen);

  const paneTabs = pane.tabIds
    .map((id) => tabs.find((t) => t.id === id))
    .filter((t): t is Tab => !!t);

  return (
    <PaneViewContainer
      pane={pane}
      paneTabs={paneTabs}
      sessionByTab={sessionByTab}
      leafCount={leafCount}
      fullscreenPaneId={fullscreenPaneId}
      isActive={usePaneStore.getState().activePaneId === pane.id}
      onSelectPane={() => setActivePane(pane.id)}
      onSelectTab={(tabId) => {
        setActiveTabInPane(pane.id, tabId);
        setActivePane(pane.id);
      }}
      onCloseTab={(tabId) => {
        const sessionId = sessionByTab[tabId];
        if (sessionId) {
          void tauri.ssh.disconnect(sessionId).catch(() => {});
          useSshStore.getState().unbind(tabId);
        }
        useTabStore.getState().closeTab(tabId);
        closeTabInPane(pane.id, tabId);
      }}
      onToggleFullscreen={() => toggleFullscreen(pane.id)}
    />
  );
}

// PaneViewContainer: the visual pane. Isolated so we can use
// React.memo to skip re-renders when only unrelated store slices change.
const PaneViewContainer = React.memo(function PaneViewContainer({
  pane,
  paneTabs,
  sessionByTab,
  leafCount,
  fullscreenPaneId,
  isActive,
  onSelectPane,
  onSelectTab,
  onCloseTab,
  onToggleFullscreen,
}: {
  pane: LeafNode;
  paneTabs: Tab[];
  sessionByTab: Record<string, string>;
  leafCount: number;
  fullscreenPaneId: string | null;
  isActive: boolean;
  onSelectPane: () => void;
  onSelectTab: (tabId: string) => void;
  onCloseTab: (tabId: string) => void;
  onToggleFullscreen: () => void;
}) {
  // Drag-store subscriptions live HERE so the PaneView JSX subtree
  // re-renders on drag updates, but the Terminal children (memoized)
  // skip re-render. PaneView's parent already memoized the heavy work.
  const hoveredZoneId = useDragStore((s) => s.hoveredZoneId);
  const hoveredZoneType = useDragStore((s) => s.hoveredZoneType);
  const hoveredPaneSplitRegion = useDragStore((s) => s.hoveredPaneSplitRegion);
  const isHovered = hoveredZoneType === 'pane' && hoveredZoneId === pane.id;
  const isFullscreen = fullscreenPaneId === pane.id;

  return (
    <div
      data-drop-zone="pane"
      data-pane-id={pane.id}
      className={cn(
        'flex flex-1 flex-col overflow-hidden border border-transparent transition-all duration-100',
        isActive && 'border-accent/30',
        isHovered && 'bg-accent/5',
      )}
      onClick={onSelectPane}
    >
      {/* Per-pane tab bar */}
      {leafCount > 1 && (
        <div
          className={cn(
            'flex h-8 shrink-0 items-stretch border-b border-border-subtle bg-bg-sidebar',
          )}
        >
          <div className="flex flex-1 items-stretch overflow-x-auto">
            {paneTabs.map((tab) => (
              <PaneTab
                key={tab.id}
                tab={tab}
                isActive={tab.id === pane.activeTabId}
                onSelect={() => onSelectTab(tab.id)}
                onClose={() => onCloseTab(tab.id)}
              />
            ))}
          </div>
          <div className="flex items-center gap-0.5 px-1">
            <button
              type="button"
              onClick={onToggleFullscreen}
              className="flex h-6 w-6 items-center justify-center rounded text-fg-muted hover:bg-bg-elevated hover:text-fg"
              title={isFullscreen ? 'Restore' : 'Maximize'}
            >
              {isFullscreen ? <Minimize2 size={12} /> : <Maximize2 size={12} />}
            </button>
          </div>
        </div>
      )}

      {/* Terminal content */}
      <div className="relative flex-1 overflow-hidden">
        {pane.tabIds.map((tabId) => {
          const sid = sessionByTab[tabId];
          const isTabActive = tabId === pane.activeTabId;
          return (
            <div
              key={tabId}
              className="absolute inset-0"
              style={{ visibility: isTabActive ? 'visible' : 'hidden' }}
            >
              {sid ? (
                <Terminal tabId={tabId} sessionId={sid} />
              ) : (
                <div className="flex h-full items-center justify-center text-fg-muted">
                  <p className="text-sm">Connecting...</p>
                </div>
              )}
            </div>
          );
        })}

        {/* Drop zone overlay */}
        {isHovered && hoveredPaneSplitRegion && (
          <DropZoneOverlay region={hoveredPaneSplitRegion} />
        )}
      </div>
    </div>
  );
});

// --- PaneTab: individual tab in per-pane tab bar ---

function PaneTab({
  tab,
  isActive,
  onSelect,
  onClose,
}: {
  tab: Tab;
  isActive: boolean;
  onSelect: () => void;
  onClose: () => void;
}) {
  const dragId = useDragStore((s) => s.dragId);
  const dragType = useDragStore((s) => s.dragType);

  const handleMouseDown = (e: React.MouseEvent) => {
    if (e.button !== 0) return;
    const startX = e.clientX;
    const startY = e.clientY;

    const handleMouseMove = (moveEvent: MouseEvent) => {
      const dx = moveEvent.clientX - startX;
      const dy = moveEvent.clientY - startY;
      if (Math.sqrt(dx * dx + dy * dy) > 5) {
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

  const isDragging = dragType === 'tab' && dragId === tab.id;

  return (
    <div
      onMouseDown={handleMouseDown}
      onClick={onSelect}
      className={cn(
        'group flex max-w-44 cursor-pointer items-center gap-1.5 border-r border-border-subtle px-2.5 text-xs transition-all',
        'border-b-2 border-b-transparent',
        isActive
          ? 'border-b-accent bg-bg text-fg'
          : 'text-fg-muted hover:bg-bg-elevated hover:text-fg',
        isDragging && 'opacity-50',
      )}
    >
      <StatusDot status={tab.status} />
      <span className="truncate">{tab.label}</span>
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          onClose();
        }}
        className={cn(
          'ml-auto flex h-4 w-4 items-center justify-center rounded',
          'text-fg-subtle hover:bg-border-strong hover:text-fg',
          'opacity-0 transition-opacity group-hover:opacity-100',
          isActive && 'opacity-100',
        )}
      >
        <X size={10} />
      </button>
    </div>
  );
}

function StatusDot({ status }: { status: string }) {
  const symbol =
    status === 'connected' ? '●' : status === 'connecting' ? '◐' : '○';
  return (
    <span
      className={cn(
        'select-none text-[10px]',
        status === 'connected' && 'text-status-connected',
        status === 'connecting' && 'text-status-connecting',
        status === 'disconnected' && 'text-status-disconnected',
      )}
    >
      {symbol}
    </span>
  );
}

// --- DropZoneOverlay: visual feedback for split direction ---

function DropZoneOverlay({ region }: { region: string }) {
  return (
    <div className="pointer-events-none absolute inset-0 z-40">
      {/* Quadrant indicators */}
      <div
        className={cn(
          'absolute bottom-0 left-0 top-0 w-1/4 rounded-l transition-colors duration-100',
          region === 'left' && 'bg-accent/20 border-r-2 border-accent',
        )}
      />
      <div
        className={cn(
          'absolute bottom-0 right-0 top-0 w-1/4 rounded-r transition-colors duration-100',
          region === 'right' && 'bg-accent/20 border-l-2 border-accent',
        )}
      />
      <div
        className={cn(
          'absolute left-1/4 right-1/4 top-0 h-1/4 transition-colors duration-100',
          region === 'top' && 'bg-accent/20 rounded-t border-b-2 border-accent',
        )}
      />
      <div
        className={cn(
          'absolute bottom-0 left-1/4 right-1/4 h-1/4 transition-colors duration-100',
          region === 'bottom' &&
            'bg-accent/20 rounded-b border-t-2 border-accent',
        )}
      />
      <div
        className={cn(
          'absolute bottom-1/4 left-1/4 right-1/4 top-1/4 rounded transition-colors duration-100',
          region === 'center' &&
            'bg-accent/15 border-2 border-dashed border-accent',
        )}
      />
    </div>
  );
}

function ServerStatsPanelWrapper() {
  const hosts = useHostStore((s) => s.hosts);
  const [selectedHost, setSelectedHost] = useState<string | null>(null);

  if (hosts.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--color-fg-muted)]">
        No hosts configured. Add a host first.
      </div>
    );
  }

  if (!selectedHost) {
    return (
      <div className="h-full overflow-y-auto p-4">
        <h2 className="mb-4 text-lg font-semibold text-[var(--color-fg)]">
          Select a host to view stats
        </h2>
        <div className="space-y-2">
          {hosts.map((h) => (
            <button
              key={h.id}
              onClick={() => setSelectedHost(h.id)}
              className="flex w-full items-center gap-3 rounded-lg border border-[var(--color-border)] p-3 text-left hover:bg-[var(--color-bg-elevated)]"
            >
              <span className="font-medium text-[var(--color-fg)]">
                {h.label}
              </span>
              <span className="text-xs text-[var(--color-fg-muted)]">
                {h.hostname}:{h.port}
              </span>
            </button>
          ))}
        </div>
      </div>
    );
  }

  const host = hosts.find((h) => h.id === selectedHost);
  return (
    <ServerStatsPanel
      hostId={selectedHost}
      hostLabel={host?.label || selectedHost}
    />
  );
}

function DockerPanelWrapper() {
  const hosts = useHostStore((s) => s.hosts);
  const [selectedHost, setSelectedHost] = useState<string | null>(null);

  if (hosts.length === 0) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--color-fg-muted)]">
        No hosts configured. Add a host first.
      </div>
    );
  }

  if (!selectedHost) {
    return (
      <div className="h-full overflow-y-auto p-4">
        <h2 className="mb-4 text-lg font-semibold text-[var(--color-fg)]">
          Select a host for Docker management
        </h2>
        <div className="space-y-2">
          {hosts.map((h) => (
            <button
              key={h.id}
              onClick={() => setSelectedHost(h.id)}
              className="flex w-full items-center gap-3 rounded-lg border border-[var(--color-border)] p-3 text-left hover:bg-[var(--color-bg-elevated)]"
            >
              <span className="font-medium text-[var(--color-fg)]">
                {h.label}
              </span>
              <span className="text-xs text-[var(--color-fg-muted)]">
                {h.hostname}:{h.port}
              </span>
            </button>
          ))}
        </div>
      </div>
    );
  }

  return <DockerPanel hostId={selectedHost} />;
}
