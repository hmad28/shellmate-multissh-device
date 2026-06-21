import { useState, useEffect } from 'react';
import { tauri } from '@/lib/tauri';
import {
  Check,
  Copy,
  Laptop,
  Loader2,
  QrCode,
  RefreshCw,
  Smartphone,
  Trash2,
  Wifi,
  WifiOff,
  Monitor,
  FolderOpen,
  Globe,
} from 'lucide-react';
import { listen } from '@tauri-apps/api/event';
import { RemoteDesktopTab } from './RemoteDesktopTab';
import { FileExplorerTab } from './FileExplorerTab';
import { DevProxyTab } from './DevProxyTab';

type PairedDevice = Awaited<
  ReturnType<typeof tauri.p2p.listPairedDevices>
>[number];

export function P2pSyncPanel() {
  const [activeTab, setActiveTab] = useState<'pairing' | 'desktop' | 'files' | 'proxy'>('pairing');
  const [isRunning, setIsRunning] = useState(false);
  const [pin, setPin] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [syncResult, setSyncResult] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [autoStart, setAutoStart] = useState(false);
  const [pairedDevices, setPairedDevices] = useState<PairedDevice[]>([]);
  const [devicesLoading, setDevicesLoading] = useState(false);
  const [tailscaleMode, setTailscaleMode] = useState<'off' | 'serve' | 'funnel'>('off');
  const [revokingDeviceId, setRevokingDeviceId] = useState<string | null>(null);

  const handleTailscaleModeChange = async (mode: 'off' | 'serve' | 'funnel') => {
    setTailscaleMode(mode);
    await tauri.settings.set('p2p.tailscale_mode', mode);
    if (isRunning) {
      setLoading(true);
      try {
        await tauri.p2p.stopSyncServer();
        const serverPin = await tauri.p2p.startSyncServer();
        setPin(serverPin);
      } catch (e) {
        setError(String(e));
      } finally {
        setLoading(false);
      }
    }
  };

  const refreshStatus = async () => {
    const status = await tauri.p2p.getSyncStatus();
    setIsRunning(status.isRunning);
    setPin(status.pairingCode ?? null);
  };

  const refreshPairedDevices = async () => {
    setDevicesLoading(true);
    try {
      setPairedDevices(await tauri.p2p.listPairedDevices());
    } catch (e) {
      setError(String(e));
    } finally {
      setDevicesLoading(false);
    }
  };

  useEffect(() => {
    void tauri.settings.list().then((settings) => {
      setAutoStart(
        settings.some(
          (setting) =>
            setting.key === 'p2p.auto_start_server' && setting.value === 'true',
        ),
      );
      const mode = settings.find((setting) => setting.key === 'p2p.tailscale_mode')?.value;
      if (mode === 'serve' || mode === 'funnel' || mode === 'off') {
        setTailscaleMode(mode);
      }
    });
    void refreshStatus();
    void refreshPairedDevices();
  }, []);

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
      const message = String(e);
      if (message.toLowerCase().includes('sync server already running')) {
        await refreshStatus();
      } else {
        setError(message);
      }
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

  const toggleAutoStart = async () => {
    const next = !autoStart;
    setAutoStart(next);
    await tauri.settings.set('p2p.auto_start_server', String(next));
  };

  const revokeDevice = async (deviceId: string) => {
    setRevokingDeviceId(deviceId);
    setError(null);
    try {
      await tauri.p2p.revokePairedDevice(deviceId);
      await refreshStatus();
      await refreshPairedDevices();
      setSyncResult('Device revoked. Pairing code rotated.');
    } catch (e) {
      setError(String(e));
    } finally {
      setRevokingDeviceId(null);
    }
  };

  const formatDate = (value: string | null | undefined) => {
    if (!value) return 'Never';
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) return value;
    return date.toLocaleString();
  };

  return (
    <div className="space-y-4">
      {/* Tab headers */}
      <div className="flex border-b border-border text-xs">
        <button
          onClick={() => setActiveTab('pairing')}
          className={`flex items-center gap-1.5 px-4 py-2.5 font-semibold border-b-2 transition-all ${
            activeTab === 'pairing'
              ? 'border-accent text-accent'
              : 'border-transparent text-fg-muted hover:text-fg hover:bg-bg-elevated/20'
          }`}
        >
          <Smartphone className="h-3.5 w-3.5" />
          Pairing & Devices
        </button>
        <button
          onClick={() => setActiveTab('desktop')}
          className={`flex items-center gap-1.5 px-4 py-2.5 font-semibold border-b-2 transition-all ${
            activeTab === 'desktop'
              ? 'border-accent text-accent'
              : 'border-transparent text-fg-muted hover:text-fg hover:bg-bg-elevated/20'
          }`}
        >
          <Monitor className="h-3.5 w-3.5" />
          Remote Desktop
        </button>
        <button
          onClick={() => setActiveTab('files')}
          className={`flex items-center gap-1.5 px-4 py-2.5 font-semibold border-b-2 transition-all ${
            activeTab === 'files'
              ? 'border-accent text-accent'
              : 'border-transparent text-fg-muted hover:text-fg hover:bg-bg-elevated/20'
          }`}
        >
          <FolderOpen className="h-3.5 w-3.5" />
          File Explorer
        </button>
        <button
          onClick={() => setActiveTab('proxy')}
          className={`flex items-center gap-1.5 px-4 py-2.5 font-semibold border-b-2 transition-all ${
            activeTab === 'proxy'
              ? 'border-accent text-accent'
              : 'border-transparent text-fg-muted hover:text-fg hover:bg-bg-elevated/20'
          }`}
        >
          <Globe className="h-3.5 w-3.5" />
          Dev Proxy
        </button>
      </div>

      <div className="p-1">
        {activeTab === 'pairing' && (
          <div className="space-y-5">
            <div className="rounded-lg border border-border bg-bg-panel p-4">
              <div className="mb-3 flex items-center gap-3">
                <span className="flex h-10 w-10 items-center justify-center rounded-md bg-accent text-white">
                  <Smartphone className="h-5 w-5" />
                </span>
                <div>
                  <h2 className="text-sm font-semibold text-fg">Sync Device</h2>
                  <p className="text-xs text-fg-muted">
                    Make this ShellMate instance the main device for phones or other
                    devices.
                  </p>
                </div>
              </div>

              <div className="grid gap-2 text-xs leading-5 text-fg-muted">
                <div className="flex gap-2">
                  <Laptop className="mt-0.5 h-3.5 w-3.5 shrink-0 text-accent" />
                  <span>Keep this main device unlocked while pairing.</span>
                </div>
                <div className="flex gap-2">
                  <Smartphone className="mt-0.5 h-3.5 w-3.5 shrink-0 text-accent" />
                  <span>On the other device, choose Sync device.</span>
                </div>
                <div className="flex gap-2">
                  <QrCode className="mt-0.5 h-3.5 w-3.5 shrink-0 text-accent" />
                  <span>
                    Copy the pairing code into the phone. It includes LAN and VPN
                    endpoints when available.
                  </span>
                </div>
              </div>
            </div>

            {/* Server controls */}
            <div className="flex items-center gap-2">
              {!isRunning ? (
                <button
                  onClick={startServer}
                  disabled={loading}
                  className="flex w-full items-center justify-center gap-2 rounded-md bg-accent px-3 py-2.5 text-xs font-semibold text-white hover:bg-accent-hover disabled:opacity-50"
                >
                  {loading ? (
                    <Loader2 className="h-3 w-3 animate-spin" />
                  ) : (
                    <Wifi className="h-3 w-3" />
                  )}
                  Start Pairing
                </button>
              ) : (
                <button
                  onClick={stopServer}
                  disabled={loading}
                  className="flex w-full items-center justify-center gap-2 rounded-md bg-red-500 px-3 py-2.5 text-xs font-semibold text-white hover:bg-red-600 disabled:opacity-50"
                >
                  {loading ? (
                    <Loader2 className="h-3 w-3 animate-spin" />
                  ) : (
                    <WifiOff className="h-3 w-3" />
                  )}
                  Stop Pairing
                </button>
              )}
            </div>

            <label className="flex items-start gap-2 rounded-md border border-border bg-bg-panel p-3 text-xs text-fg-muted">
              <input
                type="checkbox"
                checked={autoStart}
                onChange={toggleAutoStart}
                className="mt-0.5"
              />
              <span>
                Auto ON when desktop starts. Paired phones can reconnect while this
                laptop is reachable by LAN, Tailscale/WireGuard, Cloudflare Tunnel, or
                another trusted route.
              </span>
            </label>

            {/* Network Connection Mode */}
            <div className="rounded-lg border border-border bg-bg-panel p-4 space-y-3">
              <div>
                <h3 className="text-xs font-semibold text-fg">Network Connection Mode</h3>
                <p className="text-[11px] text-fg-muted">
                  Configure how the pairing code and sync server are exposed to your phone.
                </p>
              </div>
              <div className="grid grid-cols-3 gap-2">
                {(['off', 'serve', 'funnel'] as const).map((mode) => (
                  <button
                    key={mode}
                    onClick={() => handleTailscaleModeChange(mode)}
                    disabled={loading}
                    className={`rounded-md border p-2 text-center text-[11px] font-medium capitalize transition-all disabled:opacity-50 ${
                      tailscaleMode === mode
                        ? 'border-accent bg-accent/10 text-accent font-semibold'
                        : 'border-border bg-bg-sidebar text-fg hover:bg-bg-elevated'
                    }`}
                  >
                    {mode === 'off' ? 'LAN Only' : mode === 'serve' ? 'Tailscale Private' : 'Tailscale Funnel'}
                  </button>
                ))}
              </div>
              <p className="text-[11px] leading-4 text-fg-muted bg-bg-sidebar p-2.5 rounded-md border border-border-subtle">
                {tailscaleMode === 'off' && 'LAN Only: Standard local network sync. Both devices must be on the same local Wi-Fi.'}
                {tailscaleMode === 'serve' && 'Tailscale Private: Securely exposes your desktop terminal to your private Tailscale network (requires Tailscale on phone).'}
                {tailscaleMode === 'funnel' && 'Tailscale Funnel: Exposes your desktop terminal publicly over Tailscale Funnel. No Tailscale app needed on phone.'}
              </p>
            </div>

            {/* PIN display */}
            {isRunning && pin && (
              <div className="space-y-2">
                <p className="text-xs font-medium text-fg">
                  Paste this pairing code on phone
                </p>
                <div className="flex items-center gap-2">
                  <div className="min-w-0 flex-1 break-all rounded-md bg-bg-sidebar p-3 font-mono text-xs leading-5">
                    {pin}
                  </div>
                  <button
                    onClick={copyPin}
                    className="rounded-md p-2 hover:bg-bg-elevated"
                    title="Copy PIN"
                  >
                    {copied ? (
                      <Check className="h-4 w-4 text-green-500" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </button>
                </div>
                <p className="text-xs leading-5 text-fg-muted">
                  Phone path: ShellMate mobile, Sync device, paste this code. For use
                  away from Wi-Fi, keep a VPN/tunnel running on both devices.
                </p>
              </div>
            )}

            {syncResult && (
              <div className="rounded bg-green-500/10 p-2 text-xs text-green-500">
                {syncResult}
              </div>
            )}

            <div className="rounded-lg border border-border bg-bg-panel p-4">
              <div className="mb-3 flex items-center justify-between gap-3">
                <div>
                  <h3 className="text-sm font-semibold text-fg">Paired phones</h3>
                  <p className="text-xs text-fg-muted">
                    Revoke access for phones that should no longer sync.
                  </p>
                </div>
                <button
                  onClick={refreshPairedDevices}
                  disabled={devicesLoading}
                  className="rounded-md p-2 text-fg-muted hover:bg-bg-elevated hover:text-fg disabled:opacity-50"
                  title="Refresh paired phones"
                >
                  <RefreshCw
                    className={`h-4 w-4 ${devicesLoading ? 'animate-spin' : ''}`}
                  />
                </button>
              </div>

              {pairedDevices.length === 0 ? (
                <p className="text-xs text-fg-muted">No phones have paired yet.</p>
              ) : (
                <div className="space-y-2">
                  {pairedDevices.map((device) => {
                    const revoked = Boolean(device.revokedAt);
                    return (
                      <div
                        key={device.id}
                        className="flex items-center gap-3 rounded-md border border-border-subtle bg-bg-sidebar p-3"
                      >
                        <Smartphone className="h-4 w-4 shrink-0 text-accent" />
                        <div className="min-w-0 flex-1">
                          <div className="flex flex-wrap items-center gap-2">
                            <p className="truncate text-xs font-semibold text-fg">
                              {device.deviceName}
                            </p>
                            {revoked && (
                              <span className="rounded bg-red-500/10 px-1.5 py-0.5 text-[10px] font-medium text-red-400">
                                Revoked
                              </span>
                            )}
                          </div>
                          <p className="text-[11px] leading-5 text-fg-muted">
                            {device.boundIp} · Last sync{' '}
                            {formatDate(device.lastSeenAt)}
                          </p>
                          <p className="text-[11px] leading-5 text-fg-muted">
                            Paired {formatDate(device.pairedAt)}
                          </p>
                        </div>
                        {!revoked && (
                          <button
                            onClick={() => revokeDevice(device.id)}
                            disabled={revokingDeviceId === device.id}
                            className="rounded-md p-2 text-fg-muted hover:bg-red-500/10 hover:text-red-400 disabled:opacity-50"
                            title="Revoke phone"
                          >
                            {revokingDeviceId === device.id ? (
                              <Loader2 className="h-4 w-4 animate-spin" />
                            ) : (
                              <Trash2 className="h-4 w-4" />
                            )}
                          </button>
                        )}
                      </div>
                    );
                  })}
                </div>
              )}
            </div>

            {error && (
              <div className="rounded bg-red-500/10 p-2 text-xs text-red-400">
                {error}
              </div>
            )}
          </div>
        )}

        {activeTab === 'desktop' && <RemoteDesktopTab />}

        {activeTab === 'files' && <FileExplorerTab />}

        {activeTab === 'proxy' && <DevProxyTab />}
      </div>
    </div>
  );
}
