import { useState, useEffect } from 'react';
import { tauri } from '@/lib/tauri';
import { strings } from '@/i18n/en';
import { Wifi, WifiOff, Loader2, Copy, Check } from 'lucide-react';
import { listen } from '@tauri-apps/api/event';

export function P2pSyncPanel() {
  const [isRunning, setIsRunning] = useState(false);
  const [pin, setPin] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncResult, setSyncResult] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const unlisten = listen<string>('p2p:pairing-ready', (event) => {
      setPin(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const unlisten = listen<string>('p2p:sync-complete', (event) => {
      setSyncResult(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const startServer = async () => {
    setLoading(true);
    setError(null);
    setSyncResult(null);
    try {
      const serverPin = await tauri.p2p.startSyncServer();
      setPin(serverPin);
      setIsRunning(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const stopServer = async () => {
    setLoading(true);
    try {
      await tauri.p2p.stopSyncServer();
      setIsRunning(false);
      setPin(null);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const copyPin = async () => {
    if (pin) {
      await navigator.clipboard.writeText(pin);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="space-y-4 p-4">
      <div className="flex items-center gap-2 text-sm font-medium">
        <Wifi className="h-4 w-4" />
        {strings.p2pSync?.title ?? 'P2P Local Sync'}
      </div>

      <p className="text-muted-foreground text-xs">
        {strings.p2pSync?.description ??
          'Sync your hosts, credentials, and snippets to your mobile device over the local network.'}
      </p>

      {/* Server controls */}
      <div className="flex items-center gap-2">
        {!isRunning ? (
          <button
            onClick={startServer}
            disabled={loading}
            className="bg-primary text-primary-foreground hover:bg-primary/90 flex items-center gap-2 rounded-md px-3 py-2 text-xs font-medium disabled:opacity-50"
          >
            {loading ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <Wifi className="h-3 w-3" />
            )}
            {strings.p2pSync?.startServer ?? 'Start Sync Server'}
          </button>
        ) : (
          <button
            onClick={stopServer}
            disabled={loading}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90 flex items-center gap-2 rounded-md px-3 py-2 text-xs font-medium disabled:opacity-50"
          >
            {loading ? (
              <Loader2 className="h-3 w-3 animate-spin" />
            ) : (
              <WifiOff className="h-3 w-3" />
            )}
            {strings.p2pSync?.stopServer ?? 'Stop Server'}
          </button>
        )}
      </div>

      {/* PIN display */}
      {isRunning && pin && (
        <div className="space-y-2">
          <p className="text-muted-foreground text-xs">
            {strings.p2pSync?.enterPinOnMobile ??
              'Enter this PIN on your mobile device:'}
          </p>
          <div className="flex items-center gap-2">
            <div className="bg-muted flex-1 rounded-md p-3 text-center font-mono text-lg tracking-widest">
              {pin}
            </div>
            <button
              onClick={copyPin}
              className="hover:bg-muted rounded-md p-2"
              title="Copy PIN"
            >
              {copied ? (
                <Check className="h-4 w-4 text-green-500" />
              ) : (
                <Copy className="h-4 w-4" />
              )}
            </button>
          </div>
          <p className="text-muted-foreground text-xs">
            {strings.p2pSync?.scanQrOrEnterPin ??
              'On mobile: tap "Sync Devices" and enter this PIN.'}
          </p>
        </div>
      )}

      {syncResult && (
        <div className="rounded bg-green-500/10 p-2 text-xs text-green-500">
          {syncResult}
        </div>
      )}

      {error && (
        <div className="bg-destructive/10 text-destructive rounded p-2 text-xs">
          {error}
        </div>
      )}
    </div>
  );
}
