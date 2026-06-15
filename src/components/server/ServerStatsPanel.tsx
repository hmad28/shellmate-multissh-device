import { useEffect, useState, useRef } from 'react';
import { useStatsStore } from '@/stores/stats-store';
import { Cpu, HardDrive, Network, Layers, Activity } from 'lucide-react';

interface Props {
  hostId: string;
  hostLabel: string;
}

function UsageBar({ percent, color }: { percent: number; color: string }) {
  return (
    <div className="h-1.5 w-full rounded-full bg-[var(--color-border)] overflow-hidden">
      <div
        className="h-1.5 rounded-full transition-all duration-300"
        style={{ width: `${Math.min(100, percent)}%`, backgroundColor: color }}
      />
    </div>
  );
}

function formatBytes(mb: number): string {
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} GB`;
  return `${mb} MB`;
}

function formatNetSpeed(speedMb: number): string {
  const speedKb = speedMb * 1024;
  if (speedKb < 1) return '0 B/s';
  if (speedKb < 1024) return `${speedKb.toFixed(1)} KB/s`;
  return `${(speedKb / 1024).toFixed(1)} MB/s`;
}

interface CanvasGraphProps {
  data: number[];
  color: string;
  maxVal?: number;
  minVal?: number;
  label: string;
  currentVal: string;
}

function CanvasGraph({ data, color, maxVal = 100, minVal = 0, label, currentVal }: CanvasGraphProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const rect = canvas.getBoundingClientRect();
    const dpr = window.devicePixelRatio || 1;
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const width = rect.width;
    const height = rect.height;

    ctx.clearRect(0, 0, width, height);

    // Grid lines
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.04)';
    ctx.lineWidth = 1;
    const gridLines = 4;
    for (let i = 0; i <= gridLines; i++) {
      const y = (height / gridLines) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    if (data.length < 2) return;

    const maxPoints = 40;
    const xSpacing = width / (maxPoints - 1);
    const points: { x: number; y: number }[] = [];

    let rangeMax = maxVal;
    let rangeMin = minVal;
    if (maxVal === undefined || minVal === undefined) {
      const vals = [...data];
      const max = Math.max(...vals);
      const min = Math.min(...vals);
      rangeMax = max === min ? max + 1 : max;
      rangeMin = min;
    }
    const delta = rangeMax - rangeMin || 1;

    data.forEach((val, idx) => {
      const offset = maxPoints - data.length;
      const x = (idx + offset) * xSpacing;
      const y = height - ((val - rangeMin) / delta) * height;
      points.push({ x, y });
    });

    // Area fill
    if (points.length === 0) return;
    const firstPoint = points[0]!;
    const lastPoint = points[points.length - 1]!;

    const gradient = ctx.createLinearGradient(0, 0, 0, height);
    gradient.addColorStop(0, hexToRgba(color, 0.15));
    gradient.addColorStop(1, 'rgba(0,0,0,0)');

    ctx.fillStyle = gradient;
    ctx.beginPath();
    ctx.moveTo(firstPoint.x, height);
    points.forEach((p) => ctx.lineTo(p.x, p.y));
    ctx.lineTo(lastPoint.x, height);
    ctx.closePath();
    ctx.fill();

    // Line drawing
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.shadowColor = color;
    ctx.shadowBlur = 3;
    ctx.beginPath();
    points.forEach((p, idx) => {
      if (idx === 0) ctx.moveTo(p.x, p.y);
      else ctx.lineTo(p.x, p.y);
    });
    ctx.stroke();
    ctx.shadowBlur = 0;
  }, [data, color, maxVal, minVal]);

  return (
    <div className="relative w-full bg-[var(--color-bg-elevated)]/20 rounded-md border border-[var(--color-border)] p-2">
      <div className="flex items-center justify-between text-[9px] font-semibold text-[var(--color-fg-muted)] uppercase tracking-wider mb-1">
        <span>{label}</span>
        <span className="font-mono text-[var(--color-fg)] text-[10px]">{currentVal}</span>
      </div>
      <canvas ref={canvasRef} className="w-full h-12 block" />
    </div>
  );
}

function hexToRgba(hex: string, alpha: number): string {
  if (hex.startsWith('var(')) {
    if (hex.includes('accent')) return `rgba(14, 165, 233, ${alpha})`;
    if (hex.includes('disconnected')) return `rgba(239, 68, 68, ${alpha})`;
    if (hex.includes('connected')) return `rgba(34, 197, 94, ${alpha})`;
    return `rgba(255, 255, 255, ${alpha})`;
  }
  const shorthandRegex = /^#?([a-f\d])([a-f\d])([a-f\d])$/i;
  const fullHex = hex.replace(shorthandRegex, (_, r, g, b) => r + r + g + g + b + b);
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(fullHex);
  return result && result[1] && result[2] && result[3]
    ? `rgba(${parseInt(result[1], 16)}, ${parseInt(result[2], 16)}, ${parseInt(result[3], 16)}, ${alpha})`
    : `rgba(255, 255, 255, ${alpha})`;
}

export function ServerStatsPanel({ hostId, hostLabel }: Props) {
  const { stats, loading, errors, fetchStats, clearStats } = useStatsStore();
  const s = stats[hostId];
  const isLoading = loading[hostId];
  const error = errors[hostId];

  const [history, setHistory] = useState<{
    cpu: number[];
    mem: number[];
    rxSpeed: number[];
    txSpeed: number[];
  }>({
    cpu: [],
    mem: [],
    rxSpeed: [],
    txSpeed: [],
  });

  const prevNetRef = useRef<{ rx: number; tx: number; time: number } | null>(null);

  // Poll stats every 2 seconds
  useEffect(() => {
    fetchStats(hostId);
    const interval = setInterval(() => {
      fetchStats(hostId);
    }, 2000);

    return () => {
      clearInterval(interval);
      clearStats(hostId);
    };
  }, [hostId, fetchStats, clearStats]);

  // Reset history on host change
  useEffect(() => {
    setHistory({
      cpu: [],
      mem: [],
      rxSpeed: [],
      txSpeed: [],
    });
    prevNetRef.current = null;
  }, [hostId]);

  // Update history whenever stats arrive
  useEffect(() => {
    if (!s) return;
    const now = Date.now();
    let rxSpeed = 0;
    let txSpeed = 0;

    if (prevNetRef.current) {
      const timeDelta = (now - prevNetRef.current.time) / 1000.0;
      if (timeDelta > 0) {
        rxSpeed = Math.max(0, (s.netRxMb - prevNetRef.current.rx) / timeDelta);
        txSpeed = Math.max(0, (s.netTxMb - prevNetRef.current.tx) / timeDelta);
      }
    }
    prevNetRef.current = { rx: s.netRxMb, tx: s.netTxMb, time: now };

    setHistory((prev) => {
      const nextCpu = [...prev.cpu, s.cpuUsage].slice(-40);
      const nextMem = [...prev.mem, s.memUsagePercent].slice(-40);
      const nextRx = [...prev.rxSpeed, rxSpeed].slice(-40);
      const nextTx = [...prev.txSpeed, txSpeed].slice(-40);
      return {
        cpu: nextCpu,
        mem: nextMem,
        rxSpeed: nextRx,
        txSpeed: nextTx,
      };
    });
  }, [s]);

  if (isLoading && !s) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--color-fg-muted)]">
        <div className="text-center space-y-2">
          <div className="text-2xl animate-spin text-[var(--color-accent)]">⟳</div>
          <div className="text-[10px] font-semibold uppercase tracking-wider">Gathering server stats...</div>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center text-[var(--color-status-disconnected)] p-4">
        <div className="text-center space-y-3 max-w-sm">
          <div className="text-2xl">⚠️</div>
          <div className="text-xs font-semibold">{error}</div>
          <button
            onClick={() => fetchStats(hostId)}
            className="rounded bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] px-4 py-1.5 text-[10px] text-white transition-colors"
          >
            Retry Connection
          </button>
        </div>
      </div>
    );
  }

  if (!s) return null;

  const memColor = s.memUsagePercent > 90 ? 'var(--color-status-disconnected)' : s.memUsagePercent > 70 ? 'var(--color-status-connecting)' : 'var(--color-status-connected)';
  const cpuColor = s.cpuUsage > 90 ? 'var(--color-status-disconnected)' : s.cpuUsage > 70 ? 'var(--color-status-connecting)' : 'var(--color-status-connected)';

  return (
    <div className="h-full overflow-y-auto p-4 space-y-4 text-xs text-[var(--color-fg)]">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-[var(--color-border)] pb-3">
        <div>
          <h2 className="text-sm font-semibold text-[var(--color-fg)] flex items-center gap-1.5">
            <Activity className="h-4 w-4 text-[var(--color-accent)]" />
            {hostLabel}
          </h2>
          <div className="text-[10px] text-[var(--color-fg-muted)] font-medium mt-0.5">{s.osInfo} · {s.uptime}</div>
        </div>
        <div className="flex items-center gap-2">
          <span className="h-2 w-2 rounded-full bg-green-500 animate-pulse" />
          <span className="text-[9px] font-semibold uppercase text-[var(--color-fg-muted)] tracking-wider">Live</span>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* CPU */}
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-sidebar)]/30 p-3 space-y-3">
          <div className="flex items-center justify-between">
            <span className="font-semibold text-xs flex items-center gap-1">
              <Cpu className="h-3.5 w-3.5 text-[var(--color-accent)]" /> CPU
            </span>
            <span className="text-[10px] text-[var(--color-fg-muted)] max-w-[180px] truncate">{s.cpuCores} Cores · {s.cpuModel}</span>
          </div>
          <div className="flex items-center gap-3">
            <UsageBar percent={s.cpuUsage} color={cpuColor} />
            <span className="w-10 text-right font-mono font-bold text-xs">{s.cpuUsage.toFixed(1)}%</span>
          </div>
          <CanvasGraph
            data={history.cpu}
            color="#00f2fe"
            label="CPU Load History"
            currentVal={`${s.cpuUsage.toFixed(1)}%`}
            maxVal={100}
            minVal={0}
          />
          <div className="flex gap-4 text-[10px] text-[var(--color-fg-muted)] font-medium">
            <span>Load Avg: {s.load1m.toFixed(2)} / {s.load5m.toFixed(2)} / {s.load15m.toFixed(2)}</span>
          </div>
        </div>

        {/* Memory */}
        <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-sidebar)]/30 p-3 space-y-3">
          <div className="flex items-center justify-between">
            <span className="font-semibold text-xs flex items-center gap-1">
              <Layers className="h-3.5 w-3.5 text-[var(--color-accent)]" /> Memory
            </span>
            <span className="text-[10px] text-[var(--color-fg-muted)]">{formatBytes(s.memUsedMb)} / {formatBytes(s.memTotalMb)}</span>
          </div>
          <div className="flex items-center gap-3">
            <UsageBar percent={s.memUsagePercent} color={memColor} />
            <span className="w-10 text-right font-mono font-bold text-xs">{s.memUsagePercent.toFixed(1)}%</span>
          </div>
          <CanvasGraph
            data={history.mem}
            color="#a18cd1"
            label="Memory Usage History"
            currentVal={`${s.memUsagePercent.toFixed(1)}%`}
            maxVal={100}
            minVal={0}
          />
          <div className="text-[10px] text-[var(--color-fg-muted)] font-medium">Available: {formatBytes(s.memAvailableMb)}</div>
        </div>
      </div>

      {/* Network */}
      <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-sidebar)]/30 p-3 space-y-3">
        <div className="flex items-center justify-between">
          <span className="font-semibold text-xs flex items-center gap-1">
            <Network className="h-3.5 w-3.5 text-[var(--color-accent)]" /> Network (Realtime Speed)
          </span>
          <div className="flex gap-4 text-[10px] text-[var(--color-fg-muted)] font-semibold uppercase tracking-wider">
            <span>Total Rx: {s.netRxMb.toFixed(1)} MB</span>
            <span>Total Tx: {s.netTxMb.toFixed(1)} MB</span>
          </div>
        </div>
        
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <CanvasGraph
            data={history.rxSpeed}
            color="#00c6ff"
            label="Download (Rx)"
            currentVal={formatNetSpeed(history.rxSpeed[history.rxSpeed.length - 1] || 0)}
            minVal={0}
          />
          <CanvasGraph
            data={history.txSpeed}
            color="#ffaa00"
            label="Upload (Tx)"
            currentVal={formatNetSpeed(history.txSpeed[history.txSpeed.length - 1] || 0)}
            minVal={0}
          />
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {/* Disks */}
        {s.disks.length > 0 && (
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-sidebar)]/30 p-3 space-y-3">
            <span className="font-semibold text-xs flex items-center gap-1">
              <HardDrive className="h-3.5 w-3.5 text-[var(--color-accent)]" /> Disk Storage
            </span>
            <div className="space-y-3">
              {s.disks.map((disk, i) => (
                <div key={i} className="space-y-1">
                  <div className="flex items-center justify-between text-[10px] font-medium text-[var(--color-fg-muted)]">
                    <span>{disk.mount} ({disk.filesystem})</span>
                    <span>{disk.usedGb.toFixed(1)} / {disk.totalGb.toFixed(1)} GB</span>
                  </div>
                  <UsageBar
                    percent={disk.usagePercent}
                    color={disk.usagePercent > 90 ? 'var(--color-status-disconnected)' : disk.usagePercent > 80 ? 'var(--color-status-connecting)' : 'var(--color-status-connected)'}
                  />
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Top Processes */}
        {s.processes.length > 0 && (
          <div className="rounded-lg border border-[var(--color-border)] bg-[var(--color-bg-sidebar)]/30 p-3 space-y-3 overflow-hidden">
            <span className="font-semibold text-xs flex items-center gap-1.5">
              <Layers className="h-3.5 w-3.5 text-[var(--color-accent)]" /> Top Processes
            </span>
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="text-[9px] font-semibold text-[var(--color-fg-muted)] uppercase tracking-wider border-b border-[var(--color-border)]">
                    <th className="pb-1.5 font-semibold">PID</th>
                    <th className="pb-1.5 font-semibold">USER</th>
                    <th className="pb-1.5 text-right font-semibold">CPU%</th>
                    <th className="pb-1.5 text-right font-semibold">MEM%</th>
                    <th className="pb-1.5 pl-2 font-semibold">COMMAND</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-[var(--color-border-subtle)] font-mono text-[10px]">
                  {s.processes.map((p, i) => (
                    <tr key={i} className="hover:bg-[var(--color-bg-elevated)]/40 transition-colors">
                      <td className="py-1 text-[var(--color-fg-muted)]">{p.pid}</td>
                      <td className="py-1 truncate max-w-[60px]">{p.user}</td>
                      <td className="py-1 text-right font-bold text-[var(--color-accent)]">{p.cpuPercent.toFixed(1)}</td>
                      <td className="py-1 text-right">{p.memPercent.toFixed(1)}</td>
                      <td className="py-1 pl-2 truncate max-w-[140px] text-[var(--color-fg-muted)]" title={p.command}>{p.command}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
