import { useState, useEffect } from 'react';
import { tauri } from '@/lib/tauri';
import {
  Globe,
  RefreshCw,
  Plus,
  Trash2,
  ExternalLink,
  Copy,
  Check,
  QrCode,
  AlertCircle,
  X,
  Server,
  Laptop,
} from 'lucide-react';

interface PortStatus {
  port: number;
  active: boolean;
  isCustom: boolean;
}

interface DecodedPairingCode {
  v: number;
  host: string;
  port: number;
  pin: string;
  endpoints: { label: string; host: string; port: number }[];
}

export function DevProxyTab() {
  const [ports, setPorts] = useState<PortStatus[]>([]);
  const [customPorts, setCustomPorts] = useState<number[]>([]);
  const [newPort, setNewPort] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Client/Remote pairing state
  const [isClient, setIsClient] = useState(false);
  const [desktopToken, setDesktopToken] = useState<string | null>(null);
  const [deviceId, setDeviceId] = useState<string | null>(null);
  const [deviceName, setDeviceName] = useState<string | null>(null);
  const [decodedCode, setDecodedCode] = useState<DecodedPairingCode | null>(null);

  // Host/Server state
  const [isHostServerRunning, setIsHostServerRunning] = useState(false);
  const [decodedHostCode, setDecodedHostCode] = useState<DecodedPairingCode | null>(null);

  // UI state
  const [copiedLink, setCopiedLink] = useState<string | null>(null);
  const [activeQrUrl, setActiveQrUrl] = useState<string | null>(null);
  const [activeQrPort, setActiveQrPort] = useState<number | null>(null);

  // Common port labels
  const getPortLabel = (port: number) => {
    switch (port) {
      case 3000:
        return 'React / Next.js / Express';
      case 3001:
        return 'React Alt / Rails';
      case 3002:
        return 'Dev server Alt';
      case 3333:
        return 'NestJS / AdonisJS';
      case 4000:
        return 'Phoenix / Jekyll';
      case 4200:
        return 'Angular';
      case 5000:
        return 'Flask / ASP.NET HTTP';
      case 5001:
        return 'ASP.NET HTTPS';
      case 5173:
        return 'Vite (React/Vue/Svelte)';
      case 5174:
        return 'Vite Alt';
      case 8000:
        return 'FastAPI / Django / PHP';
      case 8001:
        return 'API Alt';
      case 8080:
        return 'Vue / Webpack / Spring / Go';
      case 8081:
        return 'Vue Alt / Webpack Alt';
      case 8082:
        return 'Dev API Alt';
      case 9000:
        return 'PHP-FPM / Play / Go';
      case 1313:
        return 'Hugo Static site';
      default:
        return 'Local Dev Service';
    }
  };

  const decodePairingCodeStr = (codeStr: string): DecodedPairingCode | null => {
    try {
      let base64 = codeStr.replace(/-/g, '+').replace(/_/g, '/');
      while (base64.length % 4) {
        base64 += '=';
      }
      const decoded = atob(base64);
      return JSON.parse(decoded) as DecodedPairingCode;
    } catch (e) {
      console.error('Failed to decode pairing code', e);
      return null;
    }
  };

  const loadSettingsAndStatus = async () => {
    try {
      const settings = await tauri.settings.list();
      const settingsMap = new Map(settings.map((s) => [s.key, s.value]));

      // 1. Check if we have a paired remote desktop (Client mode)
      const pCode = settingsMap.get('p2p.desktop_pairing_code') ?? null;
      const pToken = settingsMap.get('p2p.desktop_token') ?? null;
      const pDeviceId = settingsMap.get('p2p.device_id') ?? null;
      const pDeviceName = settingsMap.get('p2p.device_name') ?? 'ShellMate Mobile';

      setDesktopToken(pToken);
      setDeviceId(pDeviceId);
      setDeviceName(pDeviceName);

      if (pCode && pToken) {
        setIsClient(true);
        const decoded = decodePairingCodeStr(pCode);
        setDecodedCode(decoded);
      } else {
        setIsClient(false);
        setDecodedCode(null);
      }

      // 2. Check local sync server status (Host mode)
      const hostStatus = await tauri.p2p.getSyncStatus();
      setIsHostServerRunning(hostStatus.isRunning);
      if (hostStatus.pairingCode) {
        setDecodedHostCode(decodePairingCodeStr(hostStatus.pairingCode));
      } else {
        setDecodedHostCode(null);
      }

      // 3. Load custom ports list
      const customPortsStr = settingsMap.get('p2p.custom_dev_ports') ?? '[]';
      const parsedCustom: number[] = JSON.parse(customPortsStr);
      setCustomPorts(parsedCustom);

      return { isClientMode: !!(pCode && pToken), customPorts: parsedCustom };
    } catch (e) {
      console.error('Failed to load settings', e);
      return { isClientMode: false, customPorts: [] };
    }
  };

  const scanPorts = async () => {
    setLoading(true);
    setError(null);
    try {
      const { isClientMode } = await loadSettingsAndStatus();
      
      let res;
      if (isClientMode) {
        res = await tauri.p2p.getRemoteDevPorts();
      } else {
        res = await tauri.p2p.scanDevPorts();
      }
      
      setPorts(res.ports);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void scanPorts();
  }, []);

  const addCustomPort = async (e: React.FormEvent) => {
    e.preventDefault();
    const portNum = parseInt(newPort, 10);
    if (Number.isNaN(portNum) || portNum < 1 || portNum > 65535) {
      setError('Please enter a valid port number (1-65535)');
      return;
    }
    if (customPorts.includes(portNum)) {
      setError('Port is already added');
      return;
    }

    const nextCustom = [...customPorts, portNum].sort((a, b) => a - b);
    setCustomPorts(nextCustom);
    setNewPort('');
    setError(null);

    try {
      await tauri.settings.set('p2p.custom_dev_ports', JSON.stringify(nextCustom));
      await scanPorts();
    } catch (err) {
      setError(String(err));
    }
  };

  const removeCustomPort = async (port: number) => {
    const nextCustom = customPorts.filter((p) => p !== port);
    setCustomPorts(nextCustom);

    try {
      await tauri.settings.set('p2p.custom_dev_ports', JSON.stringify(nextCustom));
      await scanPorts();
    } catch (err) {
      setError(String(err));
    }
  };

  const copyToClipboard = async (text: string) => {
    await navigator.clipboard.writeText(text);
    setCopiedLink(text);
    setTimeout(() => setCopiedLink(null), 2000);
  };

  // Build authenticated URLs for client
  const getProxyUrls = (devPort: number) => {
    const urls: { label: string; url: string }[] = [];
    if (!isClient || !decodedCode || !deviceId || !desktopToken) return urls;

    // Host parameters to append
    const authParams = `?device_id=${encodeURIComponent(deviceId)}&device_name=${encodeURIComponent(deviceName ?? 'ShellMate Mobile')}&token=${encodeURIComponent(desktopToken)}`;

    // Add main pairing host/port
    const mainProto = decodedCode.port === 443 ? 'https' : 'http';
    urls.push({
      label: `Default (${decodedCode.host})`,
      url: `${mainProto}://${decodedCode.host}:${decodedCode.port}/proxy/${devPort}/${authParams}`,
    });

    // Add alternate endpoints
    decodedCode.endpoints.forEach((ep) => {
      const epProto = ep.port === 443 ? 'https' : 'http';
      urls.push({
        label: `${ep.label} (${ep.host})`,
        url: `${epProto}://${ep.host}:${ep.port}/proxy/${devPort}/${authParams}`,
      });
    });

    return urls;
  };

  // Build template URLs for host
  const getHostProxyTemplates = (devPort: number) => {
    const urls: { label: string; url: string }[] = [];
    if (!decodedHostCode) return urls;

    const mainProto = decodedHostCode.port === 443 ? 'https' : 'http';
    urls.push({
      label: `Default (${decodedHostCode.host})`,
      url: `${mainProto}://${decodedHostCode.host}:${decodedHostCode.port}/proxy/${devPort}/`,
    });

    decodedHostCode.endpoints.forEach((ep) => {
      const epProto = ep.port === 443 ? 'https' : 'http';
      urls.push({
        label: `${ep.label} (${ep.host})`,
        url: `${epProto}://${ep.host}:${ep.port}/proxy/${devPort}/`,
      });
    });

    return urls;
  };

  return (
    <div className="space-y-4">
      {/* Overview Card */}
      <div className="rounded-lg border border-border bg-bg-panel p-4 flex flex-col md:flex-row md:items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <span className="flex h-10 w-10 items-center justify-center rounded-md bg-accent/15 text-accent shrink-0">
            <Globe className="h-5 w-5" />
          </span>
          <div>
            <h2 className="text-sm font-semibold text-fg flex items-center gap-2">
              Local Dev Proxy
              <span className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                isClient 
                  ? 'bg-blue-500/10 text-blue-400 border border-blue-500/20' 
                  : 'bg-green-500/10 text-green-400 border border-green-500/20'
              }`}>
                {isClient ? 'Client Mode (Exposing Host)' : 'Host Mode (Local Scan)'}
              </span>
            </h2>
            <p className="text-xs text-fg-muted mt-0.5">
              {isClient
                ? 'Access development servers running on your paired desktop host directly from this device.'
                : 'Scans and exposes active dev servers on this machine to your paired mobile client devices.'}
            </p>
          </div>
        </div>

        <button
          onClick={scanPorts}
          disabled={loading}
          className="flex items-center justify-center gap-1.5 self-start md:self-auto rounded-md border border-border bg-bg-sidebar px-3 py-2 text-xs font-semibold text-fg hover:bg-bg-elevated disabled:opacity-50 transition-all shrink-0"
        >
          <RefreshCw className={`h-3.5 w-3.5 ${loading ? 'animate-spin' : ''}`} />
          {loading ? 'Scanning...' : 'Scan Ports'}
        </button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Active Ports List */}
        <div className="lg:col-span-2 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold text-fg uppercase tracking-wider">Dev Server Sockets</h3>
            <span className="text-[11px] text-fg-muted font-mono">{ports.length} scanned</span>
          </div>

          {ports.length === 0 ? (
            <div className="rounded-lg border border-border-subtle bg-bg-sidebar p-8 text-center space-y-2">
              <Server className="h-8 w-8 text-fg-muted mx-auto opacity-50" />
              <p className="text-xs font-medium text-fg">No active development servers found</p>
              <p className="text-[11px] text-fg-muted max-w-xs mx-auto">
                Start a development server on your machine (e.g. Vite, React, Django) or add a custom port to expose.
              </p>
            </div>
          ) : (
            <div className="space-y-2.5">
              {ports.map((portStatus) => {
                const label = getPortLabel(portStatus.port);
                const clientUrls = getProxyUrls(portStatus.port);
                const hostUrls = getHostProxyTemplates(portStatus.port);

                return (
                  <div
                    key={portStatus.port}
                    className={`rounded-lg border p-3.5 bg-bg-panel transition-all ${
                      portStatus.active
                        ? 'border-green-500/20 shadow-sm shadow-green-500/5'
                        : 'border-border-subtle opacity-75'
                    }`}
                  >
                    <div className="flex items-start justify-between gap-2">
                      <div className="min-w-0">
                        <div className="flex items-center gap-2 flex-wrap">
                          <span className="font-mono text-sm font-bold text-fg">
                            localhost:{portStatus.port}
                          </span>
                          <span className={`text-[10px] px-1.5 py-0.5 rounded font-medium ${
                            portStatus.active
                              ? 'bg-green-500/10 text-green-400'
                              : 'bg-red-500/10 text-red-400'
                          }`}>
                            {portStatus.active ? 'Active / Listening' : 'Offline'}
                          </span>
                          {portStatus.isCustom && (
                            <span className="text-[9px] bg-accent/10 text-accent px-1 py-0.2 rounded border border-accent/20">
                              Custom Port
                            </span>
                          )}
                        </div>
                        <p className="text-xs text-fg-muted mt-1 font-medium">{label}</p>
                      </div>

                      {portStatus.isCustom && !isClient && (
                        <button
                          onClick={() => removeCustomPort(portStatus.port)}
                          className="text-fg-muted hover:text-red-400 p-1.5 rounded-md hover:bg-red-500/10 transition-all"
                          title="Remove custom port"
                        >
                          <Trash2 className="h-3.5 w-3.5" />
                        </button>
                      )}
                    </div>

                    {/* Exponent/Proxy Links */}
                    {portStatus.active && (
                      <div className="mt-3 bg-bg-sidebar/55 rounded-md border border-border-subtle p-2.5 space-y-2">
                        <span className="text-[10px] text-fg-muted font-bold block uppercase tracking-wider">
                          {isClient ? 'Paired Client Access Links' : 'Local Proxy Server Templates'}
                        </span>
                        
                        <div className="space-y-1.5">
                          {(isClient ? clientUrls : hostUrls).map((item, idx) => (
                            <div key={idx} className="flex items-center justify-between gap-3 text-xs">
                              <span className="font-medium text-[11px] text-fg-muted truncate w-24 shrink-0">
                                {item.label}
                              </span>
                              <span className="font-mono text-[11px] text-fg select-all truncate flex-1 text-right bg-black/25 px-1.5 py-0.5 rounded border border-black/10">
                                {item.url}
                              </span>
                              <div className="flex items-center gap-1 shrink-0">
                                <button
                                  onClick={() => copyToClipboard(item.url)}
                                  className="p-1 rounded hover:bg-bg-elevated text-fg-muted hover:text-fg"
                                  title="Copy URL"
                                >
                                  {copiedLink === item.url ? (
                                    <Check className="h-3 w-3 text-green-500" />
                                  ) : (
                                    <Copy className="h-3 w-3" />
                                  )}
                                </button>
                                {isClient && (
                                  <>
                                    <a
                                      href={item.url}
                                      target="_blank"
                                      rel="noreferrer"
                                      className="p-1 rounded hover:bg-bg-elevated text-fg-muted hover:text-fg"
                                      title="Open in Browser"
                                    >
                                      <ExternalLink className="h-3 w-3" />
                                    </a>
                                    <button
                                      onClick={() => {
                                        setActiveQrUrl(item.url);
                                        setActiveQrPort(portStatus.port);
                                      }}
                                      className="p-1 rounded hover:bg-bg-elevated text-fg-muted hover:text-fg"
                                      title="Show QR Code"
                                    >
                                      <QrCode className="h-3 w-3" />
                                    </button>
                                  </>
                                )}
                              </div>
                            </div>
                          ))}
                        </div>
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          )}
        </div>

        {/* Sidebar Controls */}
        <div className="space-y-4">
          {/* Add Custom Port Form */}
          {!isClient && (
            <div className="rounded-lg border border-border bg-bg-panel p-4 space-y-3">
              <div>
                <h4 className="text-xs font-semibold text-fg">Add Custom Port</h4>
                <p className="text-[11px] text-fg-muted">
                  Register alternate dev ports that ShellMate should scan.
                </p>
              </div>

              <form onSubmit={addCustomPort} className="flex gap-2">
                <input
                  type="number"
                  placeholder="e.g. 8085"
                  value={newPort}
                  onChange={(e) => setNewPort(e.target.value)}
                  className="flex-1 bg-bg-sidebar border border-border rounded px-2.5 py-1.5 text-xs focus:border-accent focus:outline-none"
                  min="1"
                  max="65535"
                />
                <button
                  type="submit"
                  className="bg-accent hover:bg-accent-hover text-white rounded px-3 py-1.5 text-xs font-semibold flex items-center gap-1 transition-all"
                >
                  <Plus className="h-3.5 w-3.5" />
                  Add
                </button>
              </form>
            </div>
          )}

          {/* Quick Explainer Panel */}
          <div className="rounded-lg border border-border bg-bg-panel p-4 space-y-3 text-xs leading-5">
            <h4 className="font-semibold text-fg flex items-center gap-1.5">
              <Laptop className="h-3.5 w-3.5 text-accent" />
              How it works
            </h4>
            
            <div className="space-y-2 text-fg-muted text-[11px]">
              <p>
                <strong>Host side:</strong> ShellMate scans common or custom development ports listening on <code>127.0.0.1</code>.
              </p>
              <p>
                <strong>Client side:</strong> Displays active ports of the host, rendering authenticated links containing your paired credentials.
              </p>
              <p>
                <strong>Cookie mapping:</strong> Opening the link sets secure local cookies. Sub-resources (JS, CSS, images) load seamlessly without needing headers!
              </p>
            </div>

            {error && (
              <div className="flex gap-2 items-start bg-red-500/10 p-2.5 rounded border border-red-500/20 text-[11px] text-red-400 mt-2">
                <AlertCircle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                <span>{error}</span>
              </div>
            )}

            {!isClient && !isHostServerRunning && (
              <div className="flex gap-2 items-start bg-red-500/10 p-2.5 rounded border border-red-500/20 text-[11px] text-red-400 mt-2">
                <AlertCircle className="h-3.5 w-3.5 shrink-0 mt-0.5" />
                <span>
                  <strong>Server Offline:</strong> Exposing ports requires the Pairing/Sync server to be running. Start it in the <strong>Pairing & Devices</strong> tab.
                </span>
              </div>
            )}
          </div>
        </div>
      </div>

      {/* QR Code Modal Overlay */}
      {activeQrUrl && (
        <div className="fixed inset-0 bg-black/85 flex items-center justify-center z-50 p-4 animate-fade-in">
          <div className="bg-bg-panel border border-border rounded-lg max-w-sm w-full p-5 space-y-4 shadow-xl">
            <div className="flex items-center justify-between border-b border-border pb-2.5">
              <div className="flex items-center gap-2">
                <QrCode className="h-4 w-4 text-accent" />
                <h4 className="text-xs font-bold text-fg">Scan to Connect to Port {activeQrPort}</h4>
              </div>
              <button
                onClick={() => {
                  setActiveQrUrl(null);
                  setActiveQrPort(null);
                }}
                className="text-fg-muted hover:text-fg p-1 hover:bg-bg-elevated rounded"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="flex flex-col items-center justify-center p-3 bg-white rounded-md border border-border">
              <img
                src={`https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(activeQrUrl)}`}
                alt="QR Code Link"
                className="w-48 h-48 select-none"
              />
            </div>

            <p className="text-[11px] text-fg-muted text-center leading-4">
              Scan this code with your phone camera or QR reader to open the local development site in your mobile browser.
            </p>

            <button
              onClick={() => copyToClipboard(activeQrUrl)}
              className="w-full py-2 bg-bg-sidebar border border-border rounded text-xs font-semibold text-fg hover:bg-bg-elevated flex items-center justify-center gap-1.5"
            >
              {copiedLink === activeQrUrl ? (
                <>
                  <Check className="h-3.5 w-3.5 text-green-500" />
                  Copied authenticated URL!
                </>
              ) : (
                <>
                  <Copy className="h-3.5 w-3.5" />
                  Copy Link
                </>
              )}
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
