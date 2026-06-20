import React, { useState, useEffect, useRef } from 'react';
import { tauri } from '@/lib/tauri';
import {
  Play,
  Pause,
  Loader2,
  AlertCircle,
  Monitor,
  MousePointer,
  RefreshCw,
} from 'lucide-react';

export function RemoteDesktopTab() {
  const [isPlaying, setIsPlaying] = useState(false);
  const [frame, setFrame] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [fps, setFps] = useState<number>(0);
  const [latency, setLatency] = useState<number>(0);
  const [mode, setMode] = useState<'interactive' | 'view-only'>('interactive');
  
  const frameCount = useRef(0);
  const fpsInterval = useRef<NodeJS.Timeout | null>(null);
  const playInterval = useRef<NodeJS.Timeout | null>(null);
  const imgRef = useRef<HTMLImageElement | null>(null);

  const fetchFrame = async () => {
    const startTime = performance.now();
    try {
      const base64Img = await tauri.p2p.getRemoteDesktopScreenshot();
      setFrame(`data:image/jpeg;base64,${base64Img}`);
      setError(null);
      frameCount.current += 1;
      setLatency(Math.round(performance.now() - startTime));
    } catch (e) {
      setError(String(e));
      setIsPlaying(false);
    }
  };

  useEffect(() => {
    if (isPlaying) {
      setLoading(true);
      fetchFrame().finally(() => setLoading(false));
      
      // Poll at ~5 FPS (every 200ms)
      playInterval.current = setInterval(() => {
        void fetchFrame();
      }, 200);

      // FPS Counter
      fpsInterval.current = setInterval(() => {
        setFps(frameCount.current);
        frameCount.current = 0;
      }, 1000);
    } else {
      if (playInterval.current) clearInterval(playInterval.current);
      if (fpsInterval.current) clearInterval(fpsInterval.current);
      setFps(0);
    }

    return () => {
      if (playInterval.current) clearInterval(playInterval.current);
      if (fpsInterval.current) clearInterval(fpsInterval.current);
    };
  }, [isPlaying]);

  const handlePointerAction = async (
    e: React.MouseEvent<HTMLImageElement>,
    event: 'move' | 'click',
    button?: 'left' | 'right' | 'middle',
  ) => {
    if (mode === 'view-only' || !imgRef.current || !frame) return;

    const img = imgRef.current;
    const rect = img.getBoundingClientRect();
    
    // Scale standard mouse coordinates inside viewport to natural/original screen resolution
    const x = Math.round(((e.clientX - rect.left) / rect.width) * img.naturalWidth);
    const y = Math.round(((e.clientY - rect.top) / rect.height) * img.naturalHeight);

    try {
      await tauri.p2p.sendRemoteDesktopInput(
        event,
        x,
        y,
        button || 'left',
        null,
      );
    } catch (err) {
      console.error('Remote input failed:', err);
    }
  };

  const handleKeyDown = async (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (mode === 'view-only') return;
    
    // Intercept default keys to prevent scrolling or back navigation in UI
    const preventKeys = ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'Backspace', 'Tab', 'Space'];
    if (preventKeys.includes(e.key)) {
      e.preventDefault();
    }

    try {
      await tauri.p2p.sendRemoteDesktopInput(
        'keydown',
        null,
        null,
        null,
        e.key,
      );
    } catch (err) {
      console.error('Key input failed:', err);
    }
  };

  const handleKeyUp = async (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (mode === 'view-only') return;
    try {
      await tauri.p2p.sendRemoteDesktopInput(
        'keyup',
        null,
        null,
        null,
        e.key,
      );
    } catch (err) {
      console.error('Key release failed:', err);
    }
  };

  const refreshOnce = async () => {
    setLoading(true);
    setError(null);
    try {
      await fetchFrame();
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  const isNoDeviceConfigured = error?.toLowerCase().includes('no paired desktop');

  return (
    <div className="flex flex-col h-[500px] bg-bg rounded-lg overflow-hidden border border-border">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-panel text-xs">
        <div className="flex items-center gap-2">
          <button
            onClick={() => setIsPlaying(!isPlaying)}
            disabled={isNoDeviceConfigured}
            className={`flex items-center gap-1.5 px-3 py-1.5 rounded font-semibold transition-all ${
              isPlaying
                ? 'bg-red-500 hover:bg-red-600 text-white'
                : 'bg-accent hover:bg-accent-hover text-white'
            } disabled:opacity-50`}
          >
            {isPlaying ? (
              <>
                <Pause className="h-3.5 w-3.5" />
                Pause
              </>
            ) : (
              <>
                <Play className="h-3.5 w-3.5" />
                Stream Live
              </>
            )}
          </button>

          <button
            onClick={refreshOnce}
            disabled={isPlaying || loading || isNoDeviceConfigured}
            className="flex items-center gap-1 px-2.5 py-1.5 rounded border border-border bg-bg-sidebar hover:bg-bg-elevated text-fg disabled:opacity-50"
            title="Single Screenshot"
          >
            {loading ? (
              <Loader2 className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <RefreshCw className="h-3.5 w-3.5" />
            )}
            Refresh
          </button>
        </div>

        {/* Stats */}
        {isPlaying && (
          <div className="flex items-center gap-3 font-mono text-[10px] text-fg-muted bg-bg-sidebar px-2 py-1 rounded border border-border-subtle">
            <span>FPS: {fps}</span>
            <span>Latency: {latency}ms</span>
          </div>
        )}

        {/* Interaction controls */}
        <div className="flex items-center gap-1 border border-border rounded p-0.5 bg-bg-sidebar">
          <button
            onClick={() => setMode('interactive')}
            disabled={isNoDeviceConfigured}
            className={`flex items-center gap-1 px-2.5 py-1 rounded transition-all ${
              mode === 'interactive'
                ? 'bg-accent text-white font-semibold'
                : 'text-fg-muted hover:text-fg'
            }`}
          >
            <MousePointer className="h-3.5 w-3.5" />
            <span>Interactive</span>
          </button>
          <button
            onClick={() => setMode('view-only')}
            disabled={isNoDeviceConfigured}
            className={`flex items-center gap-1 px-2.5 py-1 rounded transition-all ${
              mode === 'view-only'
                ? 'bg-accent text-white font-semibold'
                : 'text-fg-muted hover:text-fg'
            }`}
          >
            <Monitor className="h-3.5 w-3.5" />
            <span>View Only</span>
          </button>
        </div>
      </div>

      {/* Screen Canvas / Viewer Area */}
      <div 
        className="relative flex-1 flex items-center justify-center bg-black/90 overflow-hidden focus:outline-none"
        tabIndex={0}
        onKeyDown={handleKeyDown}
        onKeyUp={handleKeyUp}
      >
        {frame ? (
          <div className="relative max-h-full max-w-full group">
            <img
              ref={imgRef}
              src={frame}
              alt="Remote Desktop"
              className="max-h-full max-w-full object-contain cursor-crosshair select-none"
              onMouseMove={(e) => handlePointerAction(e, 'move')}
              onMouseDown={(e) => {
                const btnMap = { 0: 'left', 1: 'middle', 2: 'right' } as const;
                const button = btnMap[e.button as keyof typeof btnMap] || 'left';
                void handlePointerAction(e, 'click', button);
              }}
              onContextMenu={(e) => e.preventDefault()}
            />
            {mode === 'interactive' && (
              <div className="absolute top-2 left-2 pointer-events-none bg-black/60 text-white rounded px-2 py-0.5 text-[10px] font-mono opacity-0 group-hover:opacity-100 transition-opacity">
                Keyboard Focused. Press keys to send.
              </div>
            )}
          </div>
        ) : (
          <div className="text-center p-6 space-y-3">
            <Monitor className="h-12 w-12 text-fg-muted mx-auto animate-pulse" />
            <div className="text-sm font-semibold text-fg">Remote Desktop View</div>
            <p className="text-xs text-fg-muted max-w-xs mx-auto">
              Start streaming to view the primary display of your paired laptop or remote device.
            </p>
          </div>
        )}

        {/* Loading Overlay */}
        {loading && !frame && (
          <div className="absolute inset-0 bg-black/75 flex items-center justify-center">
            <div className="text-center space-y-2">
              <Loader2 className="h-8 w-8 text-accent animate-spin mx-auto" />
              <div className="text-xs text-fg-muted">Connecting to remote desktop...</div>
            </div>
          </div>
        )}

        {/* Error Banner */}
        {error && (
          <div className="absolute inset-x-0 bottom-0 bg-red-500/10 border-t border-red-500/30 p-3 flex items-start gap-2.5">
            <AlertCircle className="h-4 w-4 text-red-500 shrink-0 mt-0.5" />
            <div className="text-[11px] leading-4 text-red-400">
              {isNoDeviceConfigured ? (
                <span>
                  <strong>No remote desktop paired.</strong> Please pair another desktop in the <strong>Pairing & Devices</strong> tab first to establish a remote session.
                </span>
              ) : (
                <span>Failed to stream desktop: {error}</span>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
