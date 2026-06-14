import { useEffect, useState } from 'react';
import { tauri } from '@/lib/tauri';
import type { DockerContainer } from '@/types/server-stats';

interface Props {
  hostId: string;
}

export function DockerPanel({ hostId }: Props) {
  const [containers, setContainers] = useState<DockerContainer[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const loadContainers = async () => {
    setLoading(true);
    setError('');
    try {
      const output = await tauri.serverStats.execRaw(hostId, 'docker ps -a --format "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.Ports}}" 2>/dev/null');
      const list: DockerContainer[] = output
        .split('\n')
        .filter((l) => l.trim())
        .map((line) => {
          const parts = line.split('|');
          return {
            id: parts[0]?.trim() || '',
            name: parts[1]?.trim() || '',
            image: parts[2]?.trim() || '',
            status: parts[3]?.trim() || '',
            ports: parts[4]?.trim() || '',
          };
        })
        .filter((c) => c.id);
      setContainers(list);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadContainers();
  }, [hostId]);

  const execDocker = async (cmd: string) => {
    try {
      await tauri.serverStats.execRaw(hostId, cmd);
      await loadContainers();
    } catch (err) {
      setError(String(err));
    }
  };

  if (loading) {
    return <div className="flex h-full items-center justify-center text-[var(--color-fg-muted)]">Loading containers...</div>;
  }

  if (error) {
    return (
      <div className="flex h-full flex-col items-center justify-center text-[var(--color-status-disconnected)]">
        <div>{error}</div>
        <button onClick={loadContainers} className="mt-2 rounded bg-[var(--color-accent)] px-3 py-1 text-sm text-white">Retry</button>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto p-4">
      <div className="mb-4 flex items-center justify-between">
        <h2 className="text-lg font-semibold text-[var(--color-fg)]">Docker Containers</h2>
        <button onClick={loadContainers} className="rounded bg-[var(--color-bg-elevated)] px-2 py-1 text-xs text-[var(--color-fg-muted)] hover:text-[var(--color-fg)]">↻ Refresh</button>
      </div>

      {containers.length === 0 ? (
        <div className="text-center text-[var(--color-fg-muted)]">No containers found</div>
      ) : (
        <div className="space-y-2">
          {containers.map((c) => {
            const isRunning = c.status.toLowerCase().includes('up');
            return (
              <div key={c.id} className="rounded-lg border border-[var(--color-border)] p-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <span className={`inline-block h-2 w-2 rounded-full ${isRunning ? 'bg-[var(--color-status-connected)]' : 'bg-[var(--color-status-disconnected)]'}`} />
                    <span className="font-medium text-[var(--color-fg)]">{c.name}</span>
                    <span className="text-xs text-[var(--color-fg-muted)]">{c.id.slice(0, 12)}</span>
                  </div>
                  <div className="flex gap-1">
                    {isRunning ? (
                      <>
                        <button onClick={() => execDocker(`docker stop ${c.id}`)} className="rounded bg-[var(--color-status-connecting)] px-2 py-0.5 text-xs text-white">Stop</button>
                        <button onClick={() => execDocker(`docker restart ${c.id}`)} className="rounded bg-[var(--color-accent)] px-2 py-0.5 text-xs text-white">Restart</button>
                      </>
                    ) : (
                      <>
                        <button onClick={() => execDocker(`docker start ${c.id}`)} className="rounded bg-[var(--color-status-connected)] px-2 py-0.5 text-xs text-white">Start</button>
                        <button onClick={() => execDocker(`docker rm ${c.id}`)} className="rounded bg-[var(--color-status-disconnected)] px-2 py-0.5 text-xs text-white">Remove</button>
                      </>
                    )}
                  </div>
                </div>
                <div className="mt-1 text-xs text-[var(--color-fg-muted)]">
                  <span>{c.image}</span>
                  <span className="mx-2">·</span>
                  <span>{c.status}</span>
                  {c.ports && (
                    <>
                      <span className="mx-2">·</span>
                      <span>{c.ports}</span>
                    </>
                  )}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
