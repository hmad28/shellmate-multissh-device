import { useEffect, useMemo, useState } from 'react';
import {
  ArrowLeft,
  KeyRound,
  Laptop,
  Plus,
  QrCode,
  Server,
  TerminalSquare,
  X,
} from 'lucide-react';
import { QuickConnect } from '@/components/connect/QuickConnect';
import { HostList } from '@/components/hosts/HostList';
import { SnippetPanel } from '@/components/snippets/SnippetPanel';
import { Terminal } from '@/components/terminal/Terminal';
import { ToastContainer } from '@/components/ui/Toast';
import { useHostStore } from '@/stores/host-store';
import { useSshStore } from '@/stores/ssh-store';
import { useTabStore } from '@/stores/tab-store';
import { useUiStore } from '@/stores/ui-store';
import { useVaultStore } from '@/stores/vault-store';
import { tauri } from '@/lib/tauri';
import { cn } from '@/lib/cn';
import { BottomNav } from './BottomNav';
import { ContentArea } from './ContentArea';

type MobileHomeMode = 'home' | 'sync' | 'manual' | 'saved';
type MobileSyncState = 'unknown' | 'syncing' | 'paired' | 'error';

export function MobileLayout() {
  const activePanel = useUiStore((s) => s.activePanel);
  const vaultUnlocked = useVaultStore((s) => s.unlocked);
  const loadAll = useHostStore((s) => s.loadAll);
  const [mode, setMode] = useState<MobileHomeMode>('home');
  const [syncState, setSyncState] = useState<MobileSyncState>('unknown');

  useEffect(() => {
    if (!vaultUnlocked) return;
    void loadAll();
    setSyncState('syncing');
    void tauri.p2p
      .syncWithSavedDesktop()
      .then(() => {
        setSyncState('paired');
        return useHostStore.getState().loadAll();
      })
      .catch(() => setSyncState('unknown'));
  }, [vaultUnlocked, loadAll]);

  useEffect(() => {
    if (activePanel !== 'hosts') return;
    setMode((current) => current);
  }, [activePanel]);

  const title = useMemo(() => {
    if (activePanel === 'terminal') return 'Terminal';
    if (activePanel === 'snippets') return 'Snippets';
    if (activePanel === 'settings') return 'Settings';
    if (mode === 'sync') return 'Sync device';
    if (mode === 'manual') return 'Manual host';
    if (mode === 'saved') return 'Saved hosts';
    return 'ShellMate';
  }, [activePanel, mode]);

  return (
    <div className="flex h-[100dvh] w-full flex-col overflow-hidden bg-bg text-fg">
      <MobileHeader
        title={title}
        syncState={syncState}
        canGoBack={activePanel === 'hosts' && mode !== 'home'}
        onBack={() => setMode('home')}
      />
      <main className="min-h-0 flex-1 overflow-hidden pb-[var(--mobile-content-bottom)]">
        {activePanel === 'hosts' && (
          <>
            {mode === 'home' && (
              <MobileHome onSelect={setMode} syncState={syncState} />
            )}
            {mode === 'sync' && (
              <PairDeviceScreen onPaired={() => setSyncState('paired')} />
            )}
            {mode === 'manual' && (
              <ManualHostScreen
                onConnected={() =>
                  useUiStore.getState().setActivePanel('terminal')
                }
              />
            )}
            {mode === 'saved' && <SavedHostsScreen />}
          </>
        )}
        {activePanel === 'terminal' && <MobileTerminalScreen />}
        {activePanel === 'snippets' && (
          <MobilePanel title="Snippets">
            <SnippetPanel />
          </MobilePanel>
        )}
        {activePanel !== 'hosts' &&
          activePanel !== 'terminal' &&
          activePanel !== 'snippets' && <ContentArea />}
      </main>
      <BottomNav />
      <ToastContainer />
    </div>
  );
}

function MobileHeader({
  title,
  syncState,
  canGoBack,
  onBack,
}: {
  title: string;
  syncState: MobileSyncState;
  canGoBack: boolean;
  onBack: () => void;
}) {
  const tabs = useTabStore((s) => s.tabs);
  const connected = tabs.filter((tab) => tab.status === 'connected').length;

  return (
    <header className="safe-area-inset-top shrink-0 border-b border-border bg-bg-sidebar px-3 pb-3 pt-3">
      <div className="flex items-center gap-2">
        {canGoBack && (
          <button
            type="button"
            onClick={onBack}
            aria-label="Back"
            className="flex h-9 w-9 shrink-0 items-center justify-center rounded-md text-fg-muted active:bg-bg-elevated"
          >
            <ArrowLeft size={18} />
          </button>
        )}
        <div className="min-w-0 flex-1">
          <h1 className="truncate text-base font-semibold leading-tight text-fg">
            {title}
          </h1>
          <p className="truncate text-xs text-fg-muted">
            {connected > 0
              ? `${connected} active session${connected > 1 ? 's' : ''}`
              : syncLabel(syncState)}
          </p>
        </div>
        <span
          className={cn(
            'h-2.5 w-2.5 shrink-0 rounded-full',
            syncState === 'paired' && 'bg-status-connected',
            syncState === 'syncing' && 'bg-status-connecting',
            syncState === 'error' && 'bg-status-disconnected',
            syncState === 'unknown' && 'bg-fg-subtle',
          )}
          title={syncLabel(syncState)}
        />
        <SessionPills />
      </div>
    </header>
  );
}

function syncLabel(syncState: MobileSyncState) {
  if (syncState === 'paired') return 'Laptop paired';
  if (syncState === 'syncing') return 'Syncing laptop';
  if (syncState === 'error') return 'Sync needs attention';
  return 'No laptop link';
}

function SessionPills() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const setActiveTab = useTabStore((s) => s.setActiveTab);
  const setActivePanel = useUiStore((s) => s.setActivePanel);

  if (tabs.length === 0) return null;

  return (
    <div className="flex max-w-[46vw] gap-1 overflow-x-auto">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          type="button"
          onClick={() => {
            setActiveTab(tab.id);
            setActivePanel('terminal');
          }}
          className={cn(
            'shrink-0 rounded-full border px-2.5 py-1 text-[11px]',
            tab.id === activeTabId
              ? 'border-accent bg-accent text-white'
              : 'border-border-strong bg-bg-elevated text-fg-muted',
          )}
        >
          <span className="block max-w-24 truncate">{tab.label}</span>
        </button>
      ))}
    </div>
  );
}

function MobileHome({
  onSelect,
  syncState,
}: {
  onSelect: (mode: MobileHomeMode) => void;
  syncState: MobileSyncState;
}) {
  const hosts = useHostStore((s) => s.hosts);
  const addTab = useTabStore((s) => s.addTab);
  const setActivePanel = useUiStore((s) => s.setActivePanel);
  const [openingDesktop, setOpeningDesktop] = useState(false);

  const openLaptopTerminal = async () => {
    if (openingDesktop) return;
    setOpeningDesktop(true);
    const tabId = addTab({ label: 'Laptop terminal' });
    setActivePanel('terminal');
    try {
      await useSshStore.getState().connectLocal(tabId);
    } finally {
      setOpeningDesktop(false);
    }
  };

  return (
    <div className="h-full overflow-y-auto px-4 py-4">
      <section className="mb-5">
        <p className="text-sm leading-6 text-fg-muted">
          Connect this phone to your existing ShellMate setup, or add a server
          manually for direct SSH access.
        </p>
      </section>

      <div className="grid gap-3">
        <ActionTile
          icon={<Laptop size={22} />}
          title="Sync from laptop"
          body="Pair with ShellMate over LAN or a trusted VPN/tunnel."
          onClick={() => onSelect('sync')}
          primary
        />
        <ActionTile
          icon={<TerminalSquare size={22} />}
          title={openingDesktop ? 'Opening laptop...' : 'Laptop terminal'}
          body={
            syncState === 'paired'
              ? 'Run a terminal on the paired laptop through ShellMate.'
              : 'Pair this phone first to use the laptop-backed terminal.'
          }
          onClick={() => {
            if (syncState === 'paired') void openLaptopTerminal();
            else onSelect('sync');
          }}
        />
        <ActionTile
          icon={<Plus size={22} />}
          title="Add host manually"
          body="Connect directly to a laptop, VPS, or server with SSH details."
          onClick={() => onSelect('manual')}
        />
        <ActionTile
          icon={<Server size={22} />}
          title="Saved hosts"
          body={
            hosts.length > 0
              ? `${hosts.length} host${hosts.length > 1 ? 's' : ''} available`
              : 'No saved hosts on this phone yet.'
          }
          onClick={() => onSelect('saved')}
        />
      </div>
    </div>
  );
}

function ActionTile({
  icon,
  title,
  body,
  onClick,
  primary,
}: {
  icon: React.ReactNode;
  title: string;
  body: string;
  onClick: () => void;
  primary?: boolean;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'flex w-full items-start gap-3 rounded-lg border p-4 text-left active:scale-[0.99]',
        primary ? 'border-accent/60 bg-accent/10' : 'border-border bg-bg-panel',
      )}
    >
      <span
        className={cn(
          'flex h-11 w-11 shrink-0 items-center justify-center rounded-md',
          primary ? 'bg-accent text-white' : 'bg-bg-elevated text-fg-muted',
        )}
      >
        {icon}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block text-sm font-semibold text-fg">{title}</span>
        <span className="mt-1 block text-xs leading-5 text-fg-muted">
          {body}
        </span>
      </span>
    </button>
  );
}

function PairDeviceScreen({ onPaired }: { onPaired: () => void }) {
  const [pairingCode, setPairingCode] = useState('');
  const [deviceName, setDeviceName] = useState('My phone');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const pair = async () => {
    setLoading(true);
    setResult(null);
    setError(null);
    try {
      const message = await tauri.p2p.pairWithDesktop(pairingCode, deviceName);
      await useHostStore.getState().loadAll();
      onPaired();
      setResult(message);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="h-full overflow-y-auto px-4 py-4">
      <div className="mb-4 rounded-lg border border-border bg-bg-panel p-4">
        <div className="mb-3 flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded-md bg-bg-elevated text-accent">
            <QrCode size={20} />
          </span>
          <div>
            <h2 className="text-sm font-semibold text-fg">Pair with laptop</h2>
            <p className="text-xs text-fg-muted">
              Open ShellMate on laptop, then start device sync.
            </p>
          </div>
        </div>

        <ol className="space-y-2 text-xs leading-5 text-fg-muted">
          <li>1. On laptop: open ShellMate desktop.</li>
          <li>2. Go to Sync Device and generate a pairing code or QR.</li>
          <li>
            3. Paste the pairing code below while the laptop is reachable by
            LAN, Tailscale/WireGuard, Cloudflare Tunnel, or ADB reverse.
          </li>
        </ol>
      </div>

      <label
        htmlFor="mobile-device-name"
        className="mb-1 block text-xs text-fg-muted"
      >
        Device name
      </label>
      <input
        id="mobile-device-name"
        value={deviceName}
        onChange={(event) => setDeviceName(event.target.value)}
        className="mb-3 w-full rounded-md border border-border-subtle bg-bg-elevated px-3 py-2 text-sm text-fg outline-none focus:border-accent"
      />

      <label
        htmlFor="mobile-pairing-code"
        className="mb-1 block text-xs text-fg-muted"
      >
        Pairing code
      </label>
      <textarea
        id="mobile-pairing-code"
        value={pairingCode}
        onChange={(event) => setPairingCode(event.target.value)}
        placeholder="Paste code from laptop"
        rows={4}
        className="w-full resize-none rounded-md border border-border-subtle bg-bg-elevated px-3 py-2 font-mono text-sm text-fg outline-none focus:border-accent"
      />

      <button
        type="button"
        onClick={pair}
        disabled={loading || pairingCode.trim().length === 0}
        className="mt-3 flex w-full items-center justify-center gap-2 rounded-md bg-accent px-4 py-3 text-sm font-semibold text-white disabled:bg-border-strong disabled:text-fg-muted"
      >
        <KeyRound size={16} />
        {loading ? 'Pairing...' : 'Pair and sync'}
      </button>

      {result && (
        <div className="mt-3 rounded-md bg-green-500/10 p-3 text-xs leading-5 text-green-400">
          {result}
        </div>
      )}
      {error && (
        <div className="mt-3 rounded-md bg-red-500/10 p-3 text-xs leading-5 text-red-400">
          {error}
        </div>
      )}
    </div>
  );
}

function ManualHostScreen({ onConnected }: { onConnected: () => void }) {
  return (
    <div className="h-full overflow-y-auto px-2 py-3">
      <QuickConnect onConnected={onConnected} />
    </div>
  );
}

function SavedHostsScreen() {
  return (
    <div className="flex h-full min-h-0 flex-col bg-bg-sidebar">
      <HostList />
    </div>
  );
}

function MobileTerminalScreen() {
  const tabs = useTabStore((s) => s.tabs);
  const activeTabId = useTabStore((s) => s.activeTabId);
  const closeTab = useTabStore((s) => s.closeTab);
  const setActivePanel = useUiStore((s) => s.setActivePanel);
  const sessionByTab = useSshStore((s) => s.sessionByTab);
  const activeTab =
    tabs.find((tab) => tab.id === activeTabId) ?? tabs.at(-1) ?? null;
  const sessionId = activeTab ? sessionByTab[activeTab.id] : undefined;

  if (!activeTab) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-3 px-6 text-center">
        <TerminalSquare size={28} className="text-fg-subtle" />
        <p className="text-sm text-fg-muted">No terminal session yet.</p>
        <button
          type="button"
          onClick={() => setActivePanel('hosts')}
          className="rounded-md bg-accent px-4 py-2 text-sm font-semibold text-white"
        >
          Connect a host
        </button>
      </div>
    );
  }

  const handleClose = async () => {
    if (sessionId) {
      await tauri.ssh.disconnect(sessionId).catch(() => {});
      useSshStore.getState().unbind(activeTab.id);
    }
    closeTab(activeTab.id);
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-black">
      <div className="flex h-10 shrink-0 items-center gap-2 border-b border-border bg-bg-sidebar px-3">
        <span
          className={cn(
            'h-2 w-2 shrink-0 rounded-full',
            activeTab.status === 'connected' && 'bg-status-connected',
            activeTab.status === 'connecting' && 'bg-status-connecting',
            activeTab.status === 'disconnected' && 'bg-status-disconnected',
          )}
        />
        <p className="min-w-0 flex-1 truncate text-xs font-medium text-fg">
          {activeTab.label}
        </p>
        <button
          type="button"
          onClick={handleClose}
          aria-label="Close terminal"
          className="flex h-8 w-8 items-center justify-center rounded-md text-fg-muted active:bg-bg-elevated"
        >
          <X size={16} />
        </button>
      </div>

      <div className="min-h-0 flex-1">
        {sessionId ? (
          <Terminal tabId={activeTab.id} sessionId={sessionId} />
        ) : (
          <div className="flex h-full items-center justify-center text-sm text-fg-muted">
            Connecting...
          </div>
        )}
      </div>
    </div>
  );
}

function MobilePanel({
  title,
  children,
}: {
  title: string;
  children: React.ReactNode;
}) {
  return (
    <div className="h-full overflow-y-auto bg-bg p-4">
      <h2 className="mb-3 text-sm font-semibold text-fg">{title}</h2>
      {children}
    </div>
  );
}
