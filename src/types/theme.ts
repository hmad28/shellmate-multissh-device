export type ThemeBase = 'dark' | 'light';

/**
 * Token definitions used by Tailwind theme + xterm palette.
 * Built-in themes ship as code; custom themes are stored as JSON in DB.
 */
export interface ThemeDefinition {
  id: string;
  name: string;
  base: ThemeBase;
  ui: {
    bg: string;
    bgSidebar: string;
    bgPanel: string;
    bgElevated: string;
    border: string;
    borderSubtle: string;
    borderStrong: string;
    fg: string;
    fgMuted: string;
    fgSubtle: string;
    accent: string;
    accentHover: string;
    accentSubtle: string;
    statusConnected: string;
    statusConnecting: string;
    statusDisconnected: string;
  };
  terminal: {
    background: string;
    foreground: string;
    cursor: string;
    cursorAccent: string;
    selectionBackground: string;
    /** ANSI 0-15 (8 normal + 8 bright) */
    ansi: [
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
      string,
    ];
  };
  fontFamily: string;
}

/** Storage record from backend. */
export interface Theme {
  id: string;
  name: string;
  base: ThemeBase;
  /** Stringified JSON ThemeDefinition. */
  definition: string;
  isBuiltin: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ThemeInput {
  id: string;
  name: string;
  base: ThemeBase;
  /** Stringified JSON ThemeDefinition. */
  definition: string;
}
