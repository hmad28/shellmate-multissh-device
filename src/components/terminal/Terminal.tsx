import React, { useCallback, useEffect, useRef, useState } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  ChevronDown,
  ChevronUp,
  RotateCcw,
  ZoomIn,
  ZoomOut,
} from 'lucide-react';
import '@xterm/xterm/css/xterm.css';
import { tauri } from '@/lib/tauri';
import { useTabStore } from '@/stores/tab-store';
import { useSettingsStore } from '@/stores/settings-store';
import { useIsMobile } from '@/hooks/useIsMobile';
import { MobileKeyBar } from './MobileKeyBar';
import type {
  SshOutputEvent,
  SshStatusEvent,
  SshSessionStatus,
} from '@/types/ssh';
import { cn } from '@/lib/cn';

const MIN_FONT_SIZE = 8;
const MAX_FONT_SIZE = 36;
const DEFAULT_FONT_SIZE = 14;
const FONT_SIZE_STEP = 1;

interface TerminalContextMenu {
  visible: boolean;
  x: number;
  y: number;
  hasSelection: boolean;
}

interface TerminalProps {
  /** Tab id from the frontend tab store. */
  tabId: string;
  /** SSH session id returned from the backend (`ssh:output:{sessionId}` events). */
  sessionId: string;
}

const STATUS_TO_TAB: Record<
  SshSessionStatus,
  'connecting' | 'connected' | 'disconnected'
> = {
  connecting: 'connecting',
  connected: 'connected',
  disconnected: 'disconnected',
  failed: 'disconnected',
};

export const TERMINAL_ZOOM_IN_EVENT = 'shellmate:terminal-zoom-in';
export const TERMINAL_ZOOM_OUT_EVENT = 'shellmate:terminal-zoom-out';
export const TERMINAL_ZOOM_RESET_EVENT = 'shellmate:terminal-zoom-reset';

export function Terminal({ tabId, sessionId }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<XTerm | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const fontSizeRef = useRef<number>(DEFAULT_FONT_SIZE);
  const updateTabStatus = useTabStore((s) => s.updateTabStatus);
  const isMobile = useIsMobile();
  const [contextMenu, setContextMenu] = useState<TerminalContextMenu>({
    visible: false,
    x: 0,
    y: 0,
    hasSelection: false,
  });
  const [showZoomControls, setShowZoomControls] = useState(false);
  const [fontSize, setFontSize] = useState(DEFAULT_FONT_SIZE);

  const scrollTerminalPages = useCallback((pages: number) => {
    termRef.current?.scrollPages(pages);
  }, []);

  const adjustFontSize = useCallback((delta: number) => {
    const term = termRef.current;
    if (!term) return;
    const next = Math.min(
      MAX_FONT_SIZE,
      Math.max(MIN_FONT_SIZE, fontSizeRef.current + delta),
    );
    fontSizeRef.current = next;
    setFontSize(next);
    term.options.fontSize = next;
    fitRef.current?.fit();
    void useSettingsStore
      .getState()
      .setFontSize(next)
      .catch(() => {});
  }, []);

  const resetFontSize = useCallback(() => {
    const term = termRef.current;
    if (!term) return;
    fontSizeRef.current = DEFAULT_FONT_SIZE;
    setFontSize(DEFAULT_FONT_SIZE);
    term.options.fontSize = DEFAULT_FONT_SIZE;
    fitRef.current?.fit();
    void useSettingsStore
      .getState()
      .setFontSize(DEFAULT_FONT_SIZE)
      .catch(() => {});
  }, []);

  const handleCopy = useCallback(() => {
    const term = termRef.current;
    if (!term) return;
    const selection = term.getSelection();
    if (selection) {
      void navigator.clipboard.writeText(selection);
    }
  }, []);

  const handlePaste = useCallback(() => {
    const term = termRef.current;
    if (!term) return;
    void navigator.clipboard.readText().then((text) => {
      if (text && term) {
        void tauri.ssh.send(sessionId, text);
        // Also write to xterm so it renders locally immediately
        term.write(text);
      }
    });
  }, [sessionId]);

  const handleSelectAll = useCallback(() => {
    const term = termRef.current;
    if (!term) return;
    term.selectAll();
  }, []);

  // Close context menu on outside click
  useEffect(() => {
    if (!contextMenu.visible) return;
    const handler = () =>
      setContextMenu((prev) => ({ ...prev, visible: false }));
    document.addEventListener('click', handler);
    return () => document.removeEventListener('click', handler);
  }, [contextMenu.visible]);

  const handleMobileKey = (data: string) => {
    if (termRef.current) {
      void tauri.ssh.send(sessionId, data);
    }
  };

  const toggleZoomControls = () => setShowZoomControls((prev) => !prev);

  const dismissContextMenu = useCallback(
    () => setContextMenu((prev) => ({ ...prev, visible: false })),
    [],
  );

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const { settings } = useSettingsStore.getState();
    const themeDef = useSettingsStore.getState().resolveTheme(settings.themeId);

    const term = new XTerm({
      cursorBlink: settings.cursorBlink,
      cursorStyle: settings.cursorStyle,
      fontFamily: 'JetBrains Mono, Fira Code, Consolas, Monaco, monospace',
      fontSize: settings.fontSize,
      lineHeight: 1.4,
      scrollback: settings.scrollback,
      theme: {
        background: themeDef.terminal.background,
        foreground: themeDef.terminal.foreground,
        cursor: themeDef.terminal.cursor,
        cursorAccent: themeDef.terminal.cursorAccent,
        selectionBackground: themeDef.terminal.selectionBackground,
      },
    });

    fontSizeRef.current = settings.fontSize;
    setFontSize(settings.fontSize);

    const fit = new FitAddon();
    const links = new WebLinksAddon();
    term.loadAddon(fit);
    term.loadAddon(links);
    term.open(container);

    termRef.current = term;
    fitRef.current = fit;

    // --- Keyboard zoom (Ctrl+scroll, Ctrl++/-, Ctrl+0) ---
    const handleWheel = (e: WheelEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      adjustFontSize(e.deltaY < 0 ? FONT_SIZE_STEP : -FONT_SIZE_STEP);
    };
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!e.ctrlKey && !e.metaKey) return;
      if (e.key === '=' || e.key === '+') {
        e.preventDefault();
        adjustFontSize(FONT_SIZE_STEP);
      } else if (e.key === '-') {
        e.preventDefault();
        adjustFontSize(-FONT_SIZE_STEP);
      } else if (e.key === '0') {
        e.preventDefault();
        resetFontSize();
      }
    };
    container.addEventListener('wheel', handleWheel, { passive: false });
    container.addEventListener('keydown', handleKeyDown);

    let touchLastY: number | null = null;
    let touchRemainder = 0;
    const pixelsPerLine = () => {
      const size = Number(term.options.fontSize) || DEFAULT_FONT_SIZE;
      const lineHeight = Number(term.options.lineHeight) || 1.4;
      return Math.max(8, size * lineHeight);
    };
    const handleTouchStart = (e: TouchEvent) => {
      if (e.touches.length !== 1) return;
      touchLastY = e.touches[0]?.clientY ?? null;
      touchRemainder = 0;
    };
    const handleTouchMove = (e: TouchEvent) => {
      if (e.touches.length !== 1 || touchLastY === null) return;
      const y = e.touches[0]?.clientY;
      if (typeof y !== 'number') return;
      const delta = touchLastY - y;
      touchLastY = y;

      touchRemainder += delta;
      const lines = Math.trunc(touchRemainder / pixelsPerLine());
      if (lines !== 0) {
        term.scrollLines(lines);
        touchRemainder -= lines * pixelsPerLine();
      }
      if (e.cancelable) {
        e.preventDefault();
      }
      e.stopPropagation();
    };
    const handleTouchEnd = () => {
      touchLastY = null;
      touchRemainder = 0;
    };
    container.addEventListener('touchstart', handleTouchStart, {
      capture: true,
      passive: true,
    });
    container.addEventListener('touchmove', handleTouchMove, {
      capture: true,
      passive: false,
    });
    container.addEventListener('touchend', handleTouchEnd, { capture: true });
    container.addEventListener('touchcancel', handleTouchEnd, {
      capture: true,
    });

    const handleZoomIn = () => adjustFontSize(FONT_SIZE_STEP);
    const handleZoomOut = () => adjustFontSize(-FONT_SIZE_STEP);
    const handleZoomReset = () => resetFontSize();
    window.addEventListener(TERMINAL_ZOOM_IN_EVENT, handleZoomIn);
    window.addEventListener(TERMINAL_ZOOM_OUT_EVENT, handleZoomOut);
    window.addEventListener(TERMINAL_ZOOM_RESET_EVENT, handleZoomReset);

    // --- Right-click context menu (override webview default) ---
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      const term = termRef.current;
      const hasSelection = term ? term.hasSelection() : false;
      setContextMenu({
        visible: true,
        x: e.clientX,
        y: e.clientY,
        hasSelection,
      });
    };
    container.addEventListener('contextmenu', handleContextMenu);

    // Forward keystrokes to the backend
    const onDataDisposable = term.onData((data) => {
      void tauri.ssh.send(sessionId, data);
    });

    // Notify backend of resize
    const onResizeDisposable = term.onResize(({ cols, rows }) => {
      void tauri.ssh.resize(sessionId, cols, rows);
    });

    // Fit the terminal layout to the container (triggers term.onResize if different from default)
    fit.fit();

    // Explicitly send the current terminal size to the backend immediately to ensure synchronization
    void tauri.ssh.resize(sessionId, term.cols, term.rows);

    let resizeObserver: ResizeObserver | null = null;
    if (typeof ResizeObserver !== 'undefined') {
      resizeObserver = new ResizeObserver(() => {
        try {
          fit.fit();
        } catch {
          // container may be detached during teardown
        }
      });
      resizeObserver.observe(container);
    }

    // Subscribe to backend events for THIS session id only.
    const outputName = `ssh:output:${sessionId}`;
    const statusName = `ssh:status:${sessionId}`;
    const errorName = `ssh:error:${sessionId}`;

    // Batch xterm writes per animation frame. Backend already coalesces SSH
    // chunks into ~12ms / 32KB events, but a noisy session (e.g. `cat big
    // file`) can still produce many IPC events per frame. Accumulating
    // strings and writing once per frame keeps xterm's parser from being
    // invoked on every event and prevents the stutter that looks like
    // the terminal "refreshing".
    let writeBuf = '';
    let writeScheduled = false;
    const flushWrites = () => {
      writeScheduled = false;
      if (writeBuf.length > 0) {
        const data = writeBuf;
        writeBuf = '';
        term.write(data);
      }
    };
    const scheduleWrite = (chunk: string) => {
      writeBuf += chunk;
      if (!writeScheduled) {
        writeScheduled = true;
        requestAnimationFrame(flushWrites);
      }
    };

    // Track listeners via a ref so the cleanup function captures the
    // LATEST array (post-Promise resolution), not the empty initial. Without
    // this, if the effect re-runs (e.g. sessionId change) the previous
    // listeners are NEVER unregistered and accumulate forever, multiplying
    // IPC fan-out.
    const unlistenRef: { current: UnlistenFn[] } = { current: [] };
    let mounted = true;

    void Promise.all([
      listen<SshOutputEvent>(outputName, (e) => {
        scheduleWrite(e.payload.data);
      }),
      listen<SshStatusEvent>(statusName, (e) => {
        updateTabStatus(tabId, STATUS_TO_TAB[e.payload.status]);
        if (e.payload.status === 'failed' && e.payload.message) {
          scheduleWrite(`\r\n\x1b[31m[error] ${e.payload.message}\x1b[0m\r\n`);
        }
        if (e.payload.status === 'disconnected') {
          scheduleWrite('\r\n\x1b[33m[session ended]\x1b[0m\r\n');
        }
      }),
      listen<{ sessionId: string; message: string }>(errorName, (e) => {
        scheduleWrite(`\r\n\x1b[31m[error] ${e.payload.message}\x1b[0m\r\n`);
      }),
    ]).then((fns) => {
      if (!mounted) {
        // Effect already cleaned up — unregister immediately.
        for (const fn of fns) fn();
      } else {
        unlistenRef.current = fns;
      }
    });

    term.focus();

    let desktopPoll: ReturnType<typeof setTimeout> | null = null;
    let pollingActive = true;
    let consecutiveDesktopErrors = 0;
    let desktopErrorShown = false;
    if (isMobile && sessionId.startsWith('desktop:')) {
      const poll = () => {
        if (!pollingActive) return;
        void tauri.localShell
          .read(sessionId)
          .then((data) => {
            if (!pollingActive) return;
            consecutiveDesktopErrors = 0;
            desktopErrorShown = false;
            if (data) {
              scheduleWrite(data);
              // Burst: poll again quickly for more data
              desktopPoll = setTimeout(poll, 5);
            } else {
              // Idle: back to normal interval
              desktopPoll = setTimeout(poll, 40);
            }
          })
          .catch((error) => {
            if (!pollingActive) return;
            consecutiveDesktopErrors += 1;
            if (!desktopErrorShown) {
              scheduleWrite(
                `\r\n\x1b[33m[desktop link] reconnecting to main device...\x1b[0m\r\n`,
              );
              desktopErrorShown = true;
              console.warn('[desktop link] read failed', error);
            }
            const retryDelay = Math.min(3000, 250 * consecutiveDesktopErrors);
            desktopPoll = setTimeout(poll, retryDelay);
          });
      };
      desktopPoll = setTimeout(poll, 40);
    }

    return () => {
      mounted = false;
      pollingActive = false;
      if (desktopPoll) clearTimeout(desktopPoll);
      container.removeEventListener('wheel', handleWheel);
      container.removeEventListener('keydown', handleKeyDown);
      container.removeEventListener('touchstart', handleTouchStart, {
        capture: true,
      });
      container.removeEventListener('touchmove', handleTouchMove, {
        capture: true,
      });
      container.removeEventListener('touchend', handleTouchEnd, {
        capture: true,
      });
      container.removeEventListener('touchcancel', handleTouchEnd, {
        capture: true,
      });
      window.removeEventListener(TERMINAL_ZOOM_IN_EVENT, handleZoomIn);
      window.removeEventListener(TERMINAL_ZOOM_OUT_EVENT, handleZoomOut);
      window.removeEventListener(TERMINAL_ZOOM_RESET_EVENT, handleZoomReset);
      container.removeEventListener('contextmenu', handleContextMenu);
      setContextMenu({ visible: false, x: 0, y: 0, hasSelection: false });
      for (const fn of unlistenRef.current) fn();
      onDataDisposable.dispose();
      onResizeDisposable.dispose();
      resizeObserver?.disconnect();
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, [
    sessionId,
    tabId,
    updateTabStatus,
    isMobile,
    adjustFontSize,
    resetFontSize,
  ]);

  return (
    <div
      className={cn(
        'relative flex h-full w-full flex-col bg-bg',
        isMobile ? '' : 'p-2',
      )}
    >
      {/* Zoom controls (desktop: floating button) */}
      {!isMobile && (
        <div className="absolute right-3 top-3 z-20 flex items-center gap-1">
          <button
            type="button"
            onClick={toggleZoomControls}
            title="Zoom controls"
            className="bg-bg-elevated/80 flex h-8 w-8 items-center justify-center rounded-md text-fg-muted hover:bg-bg-elevated hover:text-fg"
          >
            <ZoomIn size={16} />
          </button>
          {showZoomControls && (
            <div className="bg-bg-elevated/90 flex items-center gap-1 rounded-md px-1 py-1 text-xs text-fg-muted">
              <button
                type="button"
                onClick={() => adjustFontSize(-FONT_SIZE_STEP)}
                className="flex h-7 w-7 items-center justify-center rounded hover:bg-bg-panel"
                title="Zoom out (Ctrl+-)"
              >
                <ZoomOut size={15} />
              </button>
              <span className="w-9 text-center tabular-nums">{fontSize}px</span>
              <button
                type="button"
                onClick={() => adjustFontSize(FONT_SIZE_STEP)}
                className="flex h-7 w-7 items-center justify-center rounded hover:bg-bg-panel"
                title="Zoom in (Ctrl++)"
              >
                <ZoomIn size={15} />
              </button>
              <button
                type="button"
                onClick={resetFontSize}
                className="flex h-7 w-7 items-center justify-center rounded hover:bg-bg-panel"
                title="Reset zoom (Ctrl+0)"
              >
                <RotateCcw size={14} />
              </button>
            </div>
          )}
        </div>
      )}
      <div
        ref={containerRef}
        className="min-h-0 flex-1"
        aria-label="Terminal"
      />
      {isMobile && (
        <div className="absolute right-2 top-2 z-20 flex flex-col gap-1">
          <button
            type="button"
            onClick={() => scrollTerminalPages(-1)}
            aria-label="Scroll terminal up"
            className="bg-bg-elevated/85 flex h-9 w-9 items-center justify-center rounded-md border border-border-subtle text-fg-muted active:bg-bg-panel active:text-fg"
          >
            <ChevronUp size={18} />
          </button>
          <button
            type="button"
            onClick={() => scrollTerminalPages(1)}
            aria-label="Scroll terminal down"
            className="bg-bg-elevated/85 flex h-9 w-9 items-center justify-center rounded-md border border-border-subtle text-fg-muted active:bg-bg-panel active:text-fg"
          >
            <ChevronDown size={18} />
          </button>
        </div>
      )}
      {isMobile && <MobileKeyBar onSend={handleMobileKey} />}

      {/* Custom context menu (overrides webview default right-click) */}
      {contextMenu.visible && (
        <div
          className="fixed z-50 min-w-36 rounded-md border border-border-strong bg-bg-panel py-1 text-sm shadow-xl"
          style={{ left: contextMenu.x, top: contextMenu.y }}
        >
          <button
            type="button"
            onClick={() => {
              handleCopy();
              dismissContextMenu();
            }}
            disabled={!contextMenu.hasSelection}
            className="flex w-full items-center px-3 py-2 text-left text-fg hover:bg-bg-elevated disabled:text-fg-subtle"
          >
            Copy
          </button>
          <button
            type="button"
            onClick={() => {
              handlePaste();
              dismissContextMenu();
            }}
            className="flex w-full items-center px-3 py-2 text-left text-fg hover:bg-bg-elevated"
          >
            Paste
          </button>
          <button
            type="button"
            onClick={() => {
              handleSelectAll();
              dismissContextMenu();
            }}
            className="flex w-full items-center px-3 py-2 text-left text-fg hover:bg-bg-elevated"
          >
            Select All
          </button>
        </div>
      )}
    </div>
  );
}

// Memoize so re-renders of parent components (PaneView, HostsContent)
// don't cascade into this component, which would otherwise re-run the
// component body and re-evaluate every JSX node. The xterm instance is
// managed via refs and survives parent re-renders, but unmount/remount
// of <Terminal> is catastrophic (terminal "re-opens" visually).
export default React.memo(Terminal);
