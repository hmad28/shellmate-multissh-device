import { useEffect } from 'react';
import { useStatsStore } from '@/stores/stats-store';

interface Props {
  hostId: string;
  hostLabel: string;
}

function UsageBar({ percent, color }: { percent: number; color: string }) {
  return (
    <div className="h-2 w-full rounded-full bg-[var(--color-border)]">
      <div
        className="h-2 rounded-full transition-all duration-300"
        style={{ width: `${Math.min(100, percent)}%`, backgroundColor: color }}
      />
    </div>
  );
}

function formatBytes(mb: number): string {
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${mb} MB`;
}

export function ServerStatsPanel({ hostId, hostLabel }: Props) {
  const { stats, loading, errors, fetchStats, clearStats } = useStatsStore();
  const s = stats[hostId];
  const isLoading = loading[hostId];
  const error = errors[hostId];

  useEffect(() => {
    fetchStats(hostId);
    return () => clearStats(hostId);
  }, [hostId, fetchStats, clearStats]);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--color-fg-muted)]">
        <div className="text-center">
          <div className="mb-2 text-2xl">⟳</div>
          <div>Gathering server stats...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--color-status-disconnected)]">
        <div className="text-center">
          <div className="mb-2 text-2xl">⚠</div>
          <div>{error}</div>
          <button
            onClick={() => fetchStats(hostId)}
            className="mt-2 rounded bg-[var(--color-accent)] px-3 py-1 text-sm text-white"
          >
            Retry
          </button>
        </div>
      </div>
    );
  }

  if (!s) return null;

  const memColor = s.memUsagePercent > 90 ? 'var(--color-status-disconnected)' : s.memUsagePercent > 70 ? 'var(--color-status-connecting)' : 'var(--color-status-connected)';
  const cpuColor = s.cpuUsage > 90 ? 'var(--color-status-disconnected)' : s.cpuUsage > 70 ? 'var(--color-status-connecting)' : 'var(--color-status-connected)';

  return (
    <div className="h-full overflow-y-auto p-4 text-sm">
      {/* Header */}
      <div className="mb-4 flex items-center justify-between">
        <div>
          <h2 className="text-lg font-semibold text-[var(--color-fg)]">{hostLabel}</h2>
          <div className="text-xs text-[var(--color-fg-muted)]">{s.osInfo} · {s.uptime}</div>
        </div>
        <button
          onClick={() => fetchStats(hostId)}
          className="rounded bg-[var(--color-bg-elevated)] px-2 py-1 text-xs text-[var(--color-fg-muted)] hover:text-[var(--color-fg)]"
        >
          ↻ Refresh
        </button>
      </div>

      {/* CPU */}
      <div className="mb-4 rounded-lg border border-[var(--color-border)] p-3">
        <div className="mb-2 flex items-center justify-between">
          <span className="font-medium text-[var(--color-fg)]">CPU</span>
          <span className="text-xs text-[var(--color-fg-muted)]">{s.cpuCores} cores · {s.cpuModel}</span>
        </div>
        <div className="flex items-center gap-3">
          <UsageBar percent={s.cpuUsage} color={cpuColor} />
          <span className="w-12 text-right font-mono text-xs">{s.cpuUsage.toFixed(1)}%</span>
        </div>
        <div className="mt-2 flex gap-4 text-xs text-[var(--color-fg-muted)]">
          <span>Load: {s.load1m} / {s.load5m} / {s.load15m}</span>
        </div>
      </div>

      {/* Memory */}
      <div className="mb-4 rounded-lg border border-[var(--color-border)] p-3">
        <div className="mb-2 flex items-center justify-between">
          <span className="font-medium text-[var(--color-fg)]">Memory</span>
          <span className="text-xs text-[var(--color-fg-muted)]">{formatBytes(s.memUsedMb)} / {formatBytes(s.memTotalMb)}</span>
        </div>
        <div className="flex items-center gap-3">
          <UsageBar percent={s.memUsagePercent} color={memColor} />
          <span className="w-12 text-right font-mono text-xs">{s.memUsagePercent.toFixed(1)}%</span>
        </div>
        <div className="mt-1 text-xs text-[var(--color-fg-muted)]">Available: {formatBytes(s.memAvailableMb)}</div>
      </div>

      {/* Disks */}
      {s.disks.length > 0 && (
        <div className="mb-4 rounded-lg border border-[var(--color-border)] p-3">
          <div className="mb-2 font-medium text-[var(--color-fg)]">Disk</div>
          {s.disks.map((disk, i) => (
            <div key={i} className="mb-2 last:mb-0">
              <div className="mb-1 flex items-center justify-between text-xs">
                <span className="text-[var(--color-fg-muted)]">{disk.mount}</span>
                <span className="text-[var(--color-fg-muted)]">{disk.usedGb.toFixed(1)} / {disk.totalGb.toFixed(1)} GB</span>
              </div>
              <UsageBar percent={disk.usagePercent} color={disk.usagePercent > 90 ? 'var(--color-status-disconnected)' : disk.usagePercent > 80 ? 'var(--color-status-connecting)' : 'var(--color-status-connected)'} />
            </div>
          ))}
        </div>
      )}

      {/* Network */}
      <div className="mb-4 rounded-lg border border-[var(--color-border)] p-3">
        <div className="mb-2 font-medium text-[var(--color-fg)]">Network</div>
        <div className="flex gap-6 text-xs">
          <div>
            <span className="text-[var(--color-fg-muted)]">↓ RX: </span>
            <span className="font-mono">{s.netRxMb.toFixed(1)} MB</span>
          </div>
          <div>
            <span className="text-[var(--color-fg-muted)]">↑ TX: </span>
            <span className="font-mono">{s.netTxMb.toFixed(1)} MB</span>
          </div>
        </div>
      </div>

      {/* Top Processes */}
      {s.processes.length > 0 && (
        <div className="rounded-lg border border-[var(--color-border)] p-3">
          <div className="mb-2 font-medium text-[var(--color-fg)]">Top Processes</div>
          <table className="w-full text-xs">
            <thead>
              <tr className="text-[var(--color-fg-muted)]">
                <th className="pb-1 text-left">PID</th>
                <th className="pb-1 text-left">USER</th>
                <th className="pb-1 text-right">CPU%</th>
                <th className="pb-1 text-right">MEM%</th>
                <th className="pb-1 text-left">COMMAND</th>
              </tr>
            </thead>
            <tbody>
              {s.processes.map((p, i) => (
                <tr key={i} className="border-t border-[var(--color-border-subtle)]">
                  <td className="py-0.5 font-mono">{p.pid}</td>
                  <td className="py-0.5">{p.user}</td>
                  <td className="py-0.5 text-right font-mono">{p.cpuPercent.toFixed(1)}</td>
                  <td className="py-0.5 text-right font-mono">{p.memPercent.toFixed(1)}</td>
                  <td className="max-w-[200px] truncate py-0.5">{p.command}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
