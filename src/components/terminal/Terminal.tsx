import { useEffect, useRef } from 'react';
import { Terminal as XTerm } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import '@xterm/xterm/css/xterm.css';
import { tauri } from '@/lib/tauri';
import { useTabStore } from '@/stores/tab-store';
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

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const term = new XTerm({
      cursorBlink: true,
      cursorStyle: 'block',
      fontFamily: 'JetBrains Mono, Fira Code, Consolas, Monaco, monospace',
      fontSize: 14,
      lineHeight: 1.4,
      scrollback: 5000,
      theme: {
        background: '#0a0a0f',
        foreground: '#e8e8ea',
        cursor: '#3b82f6',
        cursorAccent: '#0a0a0f',
        selectionBackground: '#1e3a8a',
      },
    });

    const fit = new FitAddon();
    const links = new WebLinksAddon();
    term.loadAddon(fit);
    term.loadAddon(links);
    term.open(container);
    fit.fit();

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

    let unlisten: UnlistenFn[] = [];
    void Promise.all([
      listen<SshOutputEvent>(outputName, (e) => {
        term.write(e.payload.data);
      }),
      listen<SshStatusEvent>(statusName, (e) => {
        updateTabStatus(tabId, STATUS_TO_TAB[e.payload.status]);
        if (e.payload.status === 'failed' && e.payload.message) {
          term.write(`\r\n\x1b[31m[error] ${e.payload.message}\x1b[0m\r\n`);
        }
        if (e.payload.status === 'disconnected') {
          term.write('\r\n\x1b[33m[session ended]\x1b[0m\r\n');
        }
      }),
      listen<{ sessionId: string; message: string }>(errorName, (e) => {
        term.write(`\r\n\x1b[31m[error] ${e.payload.message}\x1b[0m\r\n`);
      }),
    ]).then((fns) => {
      unlisten = fns;
    });

    term.focus();

    return () => {
      unlisten.forEach((fn) => fn());
      onDataDisposable.dispose();
      onResizeDisposable.dispose();
      resizeObserver?.disconnect();
      term.dispose();
      termRef.current = null;
      fitRef.current = null;
    };
  }, [sessionId, tabId, updateTabStatus]);

  return (
    <div className={cn('h-full w-full bg-bg p-2')}>
      <div ref={containerRef} className="h-full w-full" aria-label="Terminal" />
    </div>
  );
}
