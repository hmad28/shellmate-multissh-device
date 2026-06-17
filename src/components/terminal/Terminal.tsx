import React, { useEffect, useRef } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
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

export function Terminal({ tabId, sessionId }: TerminalProps) {
  const containerRef = useRef<HTMLDivElement | null>(null);
  const termRef = useRef<XTerm | null>(null);
  const fitRef = useRef<FitAddon | null>(null);
  const updateTabStatus = useTabStore((s) => s.updateTabStatus);
  const isMobile = useIsMobile();

  const handleMobileKey = (data: string) => {
    if (termRef.current) {
      void tauri.ssh.send(sessionId, data);
    }
  };

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

    const fit = new FitAddon();
    const links = new WebLinksAddon();
    term.loadAddon(fit);
    term.loadAddon(links);
    term.open(container);

    termRef.current = term;
    fitRef.current = fit;

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

    let desktopPoll: number | null = null;
    if (isMobile && sessionId.startsWith('desktop:')) {
      desktopPoll = window.setInterval(() => {
        void tauri.localShell
          .read(sessionId)
          .then((data) => {
            if (data) scheduleWrite(data);
          })
          .catch((error) => {
            scheduleWrite(`\r\n\x1b[31m[desktop link] ${error}\x1b[0m\r\n`);
            if (desktopPoll) window.clearInterval(desktopPoll);
            desktopPoll = null;
          });
      }, 120);
    }

    return () => {
      mounted = false;
      if (desktopPoll) window.clearInterval(desktopPoll);
      for (const fn of unlistenRef.current) fn();
      onDataDisposable.dispose();
      onResizeDisposable.dispose();
      resizeObserver?.disconnect();
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, [sessionId, tabId, updateTabStatus, isMobile]);

  return (
    <div className={cn('flex h-full w-full flex-col bg-bg', isMobile ? '' : 'p-2')}>
      <div ref={containerRef} className="min-h-0 flex-1" aria-label="Terminal" />
      {isMobile && <MobileKeyBar onSend={handleMobileKey} />}
    </div>
  );
}

// Memoize so re-renders of parent components (PaneView, HostsContent)
// don't cascade into this component, which would otherwise re-run the
// component body and re-evaluate every JSX node. The xterm instance is
// managed via refs and survives parent re-renders, but unmount/remount
// of <Terminal> is catastrophic (terminal "re-opens" visually).
export default React.memo(Terminal);
