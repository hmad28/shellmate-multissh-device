import { useState, useEffect } from 'react';
import { strings } from '@/i18n/en';
import { tauri } from '@/lib/tauri';
import { useVaultStore } from '@/stores/vault-store';
import { useHostStore } from '@/stores/host-store';
import { VaultSetup } from './VaultSetup';
import { VaultUnlock } from './VaultUnlock';
import {
  Laptop,
  Server,
  KeyRound,
  QrCode,
  ArrowLeft,
  Loader2,
} from 'lucide-react';

type OnboardMode = 'choose' | 'sync' | 'setup' | 'unlock' | 'reconnect';

export function MobileOnboard() {
  const { initialized, unlocked, refresh } = useVaultStore();
  const [mode, setMode] = useState<OnboardMode>('choose');
  const [syncMessage, setSyncMessage] = useState<string | null>(null);
  const [checkingPairing, setCheckingPairing] = useState(true);

  useEffect(() => {
    let cancelled = false;

    const bootstrap = async () => {
      setCheckingPairing(true);
      try {
        const paired = await hasSavedDesktopPairing();
        if (cancelled) return;

        if (!initialized) {
          setMode(paired ? 'reconnect' : 'choose');
          return;
        }

        if (!paired) {
          setMode('unlock');
          return;
        }

        setMode('reconnect');
        const result = await tauri.p2p.autoUnlock();
        if (cancelled) return;
        setSyncMessage(result);
        await useHostStore.getState().loadAll();
        await refresh();
      } catch (err) {
        if (!cancelled) {
          setSyncMessage(String(err));
          setMode(initialized ? 'reconnect' : 'choose');
        }
      } finally {
        if (!cancelled) setCheckingPairing(false);
      }
    };

    void bootstrap();
    return () => {
      cancelled = true;
    };
  }, [initialized, refresh]);

  if (unlocked) return null;

  if (checkingPairing) {
    return (
      <Shell title="ShellMate" subtitle="Checking device sync...">
        <div className="flex items-center gap-2 rounded-md border border-border bg-bg-sidebar p-3 text-xs text-fg-muted">
          <Loader2 size={14} className="animate-spin text-accent" />
          Preparing mobile access
        </div>
      </Shell>
    );
  }

  if (mode === 'setup') return <VaultSetup />;
  if (mode === 'unlock') return <VaultUnlock />;
  if (mode === 'reconnect') {
    return (
      <Shell
        title={strings.vault.title}
        subtitle="This phone is synced to your main device."
      >
        <ReconnectScreen
          message={syncMessage}
          onRetry={async () => {
            setCheckingPairing(true);
            try {
              const result = await tauri.p2p.autoUnlock();
              setSyncMessage(result);
              await useHostStore.getState().loadAll();
              await refresh();
            } catch (err) {
              setSyncMessage(String(err));
            } finally {
              setCheckingPairing(false);
            }
          }}
          onPairAgain={() => setMode('sync')}
        />
      </Shell>
    );
  }

  return (
    <Shell
      title={strings.vault.title}
      subtitle="Sync an existing ShellMate device or make this phone the first host."
    >
      {mode === 'sync' ? (
        <PairScreen
          onPaired={async (message) => {
            setSyncMessage(message);
            await refresh();
          }}
          onBack={() => setMode('choose')}
          message={syncMessage}
        />
      ) : (
        <ChooseScreen
          onSync={() => setMode('sync')}
          onSetup={() => setMode('setup')}
        />
      )}
    </Shell>
  );
}

async function hasSavedDesktopPairing() {
  const settings = await tauri.settings.list();
  return settings.some(
    (setting) =>
      setting.key === 'p2p.desktop_token' && setting.value.length > 0,
  );
}

function Shell({
  title,
  subtitle,
  children,
}: {
  title: string;
  subtitle: string;
  children: React.ReactNode;
}) {
  return (
    <div className="flex h-full w-full items-center justify-center bg-bg p-6">
      <div className="w-full max-w-md rounded-lg border border-border bg-bg-panel p-6 shadow-lg">
        <header className="mb-5 flex items-center gap-3">
          <Mark />
          <div>
            <h1 className="text-base font-semibold text-fg">{title}</h1>
            <p className="text-xs text-fg-muted">{subtitle}</p>
          </div>
        </header>
        {children}
      </div>
    </div>
  );
}

function ChooseScreen({
  onSync,
  onSetup,
}: {
  onSync: () => void;
  onSetup: () => void;
}) {
  return (
    <div className="space-y-3">
      <button
        type="button"
        onClick={onSync}
        className="border-accent/60 bg-accent/10 flex w-full items-start gap-3 rounded-lg border p-4 text-left active:scale-[0.99]"
      >
        <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-accent text-white">
          <Laptop size={22} />
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-sm font-semibold text-fg">
            Sync Device
          </span>
          <span className="mt-1 block text-xs leading-5 text-fg-muted">
            Pair to your main ShellMate device. Hosts, credentials, snippets,
            and the vault password follow that device.
          </span>
        </span>
      </button>

      <button
        type="button"
        onClick={onSetup}
        className="flex w-full items-start gap-3 rounded-lg border border-border bg-bg-panel p-4 text-left active:scale-[0.99]"
      >
        <span className="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-bg-elevated text-fg-muted">
          <Server size={22} />
        </span>
        <span className="min-w-0 flex-1">
          <span className="block text-sm font-semibold text-fg">
            Setup Host
          </span>
          <span className="mt-1 block text-xs leading-5 text-fg-muted">
            Make this phone the first ShellMate device. Create a master password
            and add SSH hosts manually.
          </span>
        </span>
      </button>
    </div>
  );
}

function PairScreen({
  onPaired,
  onBack,
  message,
}: {
  onPaired: (message: string) => Promise<void>;
  onBack: () => void;
  message: string | null;
}) {
  const [pairingCode, setPairingCode] = useState('');
  const [deviceName, setDeviceName] = useState('My phone');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<string | null>(message);

  const pair = async () => {
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const msg = await tauri.p2p.pairWithDesktop(pairingCode, deviceName);
      await useHostStore.getState().loadAll();
      setResult(msg);
      await onPaired(msg);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <div className="mb-4 rounded-lg border border-border bg-bg-panel p-4">
        <div className="mb-3 flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded-md bg-bg-elevated text-accent">
            <QrCode size={20} />
          </span>
          <div>
            <h2 className="text-sm font-semibold text-fg">
              Pair with main device
            </h2>
            <p className="text-xs text-fg-muted">
              Open ShellMate on your main device, then start device sync.
            </p>
          </div>
        </div>
        <ol className="space-y-2 text-xs leading-5 text-fg-muted">
          <li>1. On main device: open ShellMate and tap Sync Device.</li>
          <li>2. Copy or scan the pairing code.</li>
          <li>
            3. Paste the code below while the main device is reachable by LAN or
            VPN.
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
        onChange={(e) => setDeviceName(e.target.value)}
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
        onChange={(e) => setPairingCode(e.target.value)}
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

      <button
        type="button"
        onClick={onBack}
        className="mt-2 flex w-full items-center justify-center gap-1 rounded-md px-4 py-2 text-sm text-fg-muted hover:text-fg"
      >
        <ArrowLeft size={14} />
        Back
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

function ReconnectScreen({
  message,
  onRetry,
  onPairAgain,
}: {
  message: string | null;
  onRetry: () => void;
  onPairAgain: () => void;
}) {
  return (
    <div className="space-y-3">
      <div className="rounded-lg border border-border bg-bg-sidebar p-4">
        <h2 className="text-sm font-semibold text-fg">Sync device mode</h2>
        <p className="mt-1 text-xs leading-5 text-fg-muted">
          This phone uses the vault from your main ShellMate device. Local data
          remains available; terminal access through the main device only needs
          that device to be reachable.
        </p>
      </div>
      <button
        type="button"
        onClick={onRetry}
        className="flex w-full items-center justify-center gap-2 rounded-md bg-accent px-4 py-3 text-sm font-semibold text-white"
      >
        <KeyRound size={16} />
        Unlock from synced device
      </button>
      <button
        type="button"
        onClick={onPairAgain}
        className="flex w-full items-center justify-center gap-2 rounded-md border border-border bg-bg-panel px-4 py-3 text-sm font-semibold text-fg"
      >
        <QrCode size={16} />
        Pair again
      </button>
      {message && (
        <div className="rounded-md bg-bg-elevated p-3 text-xs leading-5 text-fg-muted">
          {message}
        </div>
      )}
    </div>
  );
}

function Mark() {
  return (
    <svg
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="text-accent"
      aria-hidden="true"
    >
      <polyline points="4 17 10 11 4 5" />
      <line x1="12" y1="19" x2="20" y2="19" />
    </svg>
  );
}
