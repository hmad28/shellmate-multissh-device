import { useEffect, useState } from 'react';
import { Network, Plus } from 'lucide-react';
import {
  useDiscoveryStore,
  type DiscoveredHost,
} from '@/stores/discovery-store';
import { HostForm } from '@/components/hosts/HostForm';
import { cn } from '@/lib/cn';

export function DiscoveredHostList() {
  const { hosts, isScanning, startDiscovery, stopDiscovery } =
    useDiscoveryStore();
  const [selectedHost, setSelectedHost] = useState<DiscoveredHost | null>(null);

  useEffect(() => {
    void startDiscovery();
    return () => {
      void stopDiscovery();
    };
  }, [startDiscovery, stopDiscovery]);

  if (hosts.length === 0) return null;

  return (
    <>
      <div className="border-b border-border-subtle p-2">
        <div className="flex items-center gap-2 px-1 py-1 text-xs font-medium uppercase tracking-wider text-fg-muted">
          <Network
            size={12}
            className={cn(isScanning && 'animate-pulse text-accent')}
          />
          <span>Discovered ({hosts.length})</span>
        </div>
        <ul className="mt-1 flex flex-col gap-0.5">
          {hosts.map((host, idx) => (
            <li key={`${host.hostname}-${idx}`}>
              <div className="group flex items-center justify-between rounded px-2 py-1.5 text-sm text-fg-muted transition-colors hover:bg-bg-elevated hover:text-fg">
                <div className="flex flex-col overflow-hidden">
                  <span className="truncate">{host.hostname}</span>
                  {host.ipAddresses[0] && (
                    <span className="truncate text-xs text-fg-subtle">
                      {host.ipAddresses[0]}
                    </span>
                  )}
                </div>
                <button
                  type="button"
                  onClick={() => setSelectedHost(host)}
                  className="bg-accent/10 hover:bg-accent/20 invisible flex shrink-0 items-center gap-1 rounded px-1.5 py-0.5 text-xs text-accent transition-colors group-hover:visible"
                  title="Add Host"
                >
                  <Plus size={12} />
                </button>
              </div>
            </li>
          ))}
        </ul>
      </div>

      <HostForm
        open={!!selectedHost}
        onClose={() => setSelectedHost(null)}
        initialData={
          selectedHost
            ? {
                label: selectedHost.hostname,
                hostname: selectedHost.ipAddresses[0] || selectedHost.hostname,
                port: String(selectedHost.port),
                username: '',
              }
            : undefined
        }
      />
    </>
  );
}
