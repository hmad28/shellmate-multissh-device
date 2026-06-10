import type { ThemeDefinition } from '@/types/theme';

/**
 * Default dark theme — the original ShellMate palette from Phase 1.
 */
export const darkTheme: ThemeDefinition = {
  id: 'shellmate-dark',
  name: 'ShellMate Dark',
  base: 'dark',
  ui: {
    bg: '#0a0a0f',
    bgSidebar: '#111118',
    bgPanel: '#15151d',
    bgElevated: '#1c1c26',
    border: '#26262f',
    borderSubtle: '#1e1e26',
    borderStrong: '#33333f',
    fg: '#e8e8ea',
    fgMuted: '#9a9aa3',
    fgSubtle: '#6b6b75',
    accent: '#3b82f6',
    accentHover: '#60a5fa',
    accentSubtle: '#1e3a8a',
    statusConnected: '#22c55e',
    statusConnecting: '#f59e0b',
    statusDisconnected: '#ef4444',
  },
  terminal: {
    background: '#0a0a0f',
    foreground: '#e8e8ea',
    cursor: '#3b82f6',
    cursorAccent: '#0a0a0f',
    selectionBackground: '#1e3a8a',
    ansi: [
      '#1c1c26', // black
      '#ef4444', // red
      '#22c55e', // green
      '#f59e0b', // yellow
      '#3b82f6', // blue
      '#a855f7', // magenta
      '#06b6d4', // cyan
      '#e8e8ea', // white
      '#6b6b75', // bright black
      '#fb7185', // bright red
      '#4ade80', // bright green
      '#fbbf24', // bright yellow
      '#60a5fa', // bright blue
      '#c084fc', // bright magenta
      '#22d3ee', // bright cyan
      '#ffffff', // bright white
    ],
  },
  fontFamily: 'JetBrains Mono, Fira Code, Consolas, Monaco, monospace',
};

/**
 * Light theme variant — same accent palette adjusted for light bg.
 */
export const lightTheme: ThemeDefinition = {
  id: 'shellmate-light',
  name: 'ShellMate Light',
  base: 'light',
  ui: {
    bg: '#ffffff',
    bgSidebar: '#f7f7f9',
    bgPanel: '#fafafc',
    bgElevated: '#f0f0f3',
    border: '#d8d8df',
    borderSubtle: '#e4e4ea',
    borderStrong: '#c4c4ce',
    fg: '#1c1c26',
    fgMuted: '#5e5e6b',
    fgSubtle: '#9a9aa3',
    accent: '#2563eb',
    accentHover: '#3b82f6',
    accentSubtle: '#dbeafe',
    statusConnected: '#16a34a',
    statusConnecting: '#d97706',
    statusDisconnected: '#dc2626',
  },
  terminal: {
    background: '#ffffff',
    foreground: '#1c1c26',
    cursor: '#2563eb',
    cursorAccent: '#ffffff',
    selectionBackground: '#dbeafe',
    ansi: [
      '#1c1c26', // black
      '#dc2626', // red
      '#16a34a', // green
      '#d97706', // yellow
      '#2563eb', // blue
      '#9333ea', // magenta
      '#0891b2', // cyan
      '#5e5e6b', // white
      '#9a9aa3', // bright black
      '#ef4444', // bright red
      '#22c55e', // bright green
      '#f59e0b', // bright yellow
      '#3b82f6', // bright blue
      '#a855f7', // bright magenta
      '#06b6d4', // bright cyan
      '#1c1c26', // bright white
    ],
  },
  fontFamily: 'JetBrains Mono, Fira Code, Consolas, Monaco, monospace',
};

/**
 * High-contrast theme — WCAG AAA contrast on all UI text.
 */
export const highContrastTheme: ThemeDefinition = {
  id: 'shellmate-high-contrast',
  name: 'High Contrast',
  base: 'dark',
  ui: {
    bg: '#000000',
    bgSidebar: '#000000',
    bgPanel: '#0a0a0a',
    bgElevated: '#1a1a1a',
    border: '#ffffff',
    borderSubtle: '#666666',
    borderStrong: '#ffffff',
    fg: '#ffffff',
    fgMuted: '#dddddd',
    fgSubtle: '#aaaaaa',
    accent: '#ffff00',
    accentHover: '#ffff66',
    accentSubtle: '#666600',
    statusConnected: '#00ff00',
    statusConnecting: '#ffaa00',
    statusDisconnected: '#ff4444',
  },
  terminal: {
    background: '#000000',
    foreground: '#ffffff',
    cursor: '#ffff00',
    cursorAccent: '#000000',
    selectionBackground: '#666600',
    ansi: [
      '#000000',
      '#ff4444',
      '#00ff00',
      '#ffff00',
      '#4488ff',
      '#ff44ff',
      '#00ffff',
      '#ffffff',
      '#888888',
      '#ff8888',
      '#88ff88',
      '#ffff88',
      '#88aaff',
      '#ff88ff',
      '#88ffff',
      '#ffffff',
    ],
  },
  fontFamily: 'JetBrains Mono, Fira Code, Consolas, Monaco, monospace',
};

export const builtinThemes: ThemeDefinition[] = [
  darkTheme,
  lightTheme,
  highContrastTheme,
];

/** Apply a theme by setting CSS variables on `<html>`. */
export function applyTheme(theme: ThemeDefinition): void {
  const root = document.documentElement;
  const set = (k: string, v: string) => root.style.setProperty(k, v);

  set('--color-bg', theme.ui.bg);
  set('--color-bg-sidebar', theme.ui.bgSidebar);
  set('--color-bg-panel', theme.ui.bgPanel);
  set('--color-bg-elevated', theme.ui.bgElevated);
  set('--color-border', theme.ui.border);
  set('--color-border-subtle', theme.ui.borderSubtle);
  set('--color-border-strong', theme.ui.borderStrong);
  set('--color-fg', theme.ui.fg);
  set('--color-fg-muted', theme.ui.fgMuted);
  set('--color-fg-subtle', theme.ui.fgSubtle);
  set('--color-accent', theme.ui.accent);
  set('--color-accent-hover', theme.ui.accentHover);
  set('--color-accent-subtle', theme.ui.accentSubtle);
  set('--color-status-connected', theme.ui.statusConnected);
  set('--color-status-connecting', theme.ui.statusConnecting);
  set('--color-status-disconnected', theme.ui.statusDisconnected);
  set('--font-mono', theme.fontFamily);

  // Toggle html.dark for Tailwind dark variant compatibility.
  if (theme.base === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
  }
}
