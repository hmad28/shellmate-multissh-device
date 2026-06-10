import { create } from 'zustand';
import { tauri } from '@/lib/tauri';
import { applyTheme, builtinThemes } from '@/themes/builtin';
import type { Theme, ThemeDefinition } from '@/types/theme';

const SETTING_THEME_ID = 'ui.theme.id';
const SETTING_FONT_SIZE = 'terminal.font_size';
const SETTING_SCROLLBACK = 'terminal.scrollback';
const SETTING_AUTOLOCK_SECS = 'vault.autolock_secs';
const SETTING_CURSOR_STYLE = 'terminal.cursor_style';
const SETTING_CURSOR_BLINK = 'terminal.cursor_blink';

export type CursorStyle = 'block' | 'bar' | 'underline';

export interface AppSettings {
  themeId: string;
  fontSize: number;
  scrollback: number;
  autolockSecs: number;
  cursorStyle: CursorStyle;
  cursorBlink: boolean;
}

const DEFAULTS: AppSettings = {
  themeId: 'shellmate-dark',
  fontSize: 14,
  scrollback: 5000,
  autolockSecs: 15 * 60,
  cursorStyle: 'block',
  cursorBlink: true,
};

interface SettingsStore {
  settings: AppSettings;
  themes: Theme[]; // raw DB rows
  loaded: boolean;
  load: () => Promise<void>;
  setTheme: (themeId: string) => Promise<void>;
  setFontSize: (px: number) => Promise<void>;
  setScrollback: (lines: number) => Promise<void>;
  setAutolockSecs: (secs: number) => Promise<void>;
  setCursorStyle: (style: CursorStyle) => Promise<void>;
  setCursorBlink: (blink: boolean) => Promise<void>;
  saveTheme: (def: ThemeDefinition) => Promise<void>;
  deleteTheme: (id: string) => Promise<void>;
  resolveTheme: (id: string) => ThemeDefinition;
}

function parseDefinition(json: string): ThemeDefinition | null {
  try {
    return JSON.parse(json) as ThemeDefinition;
  } catch {
    return null;
  }
}

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: { ...DEFAULTS },
  themes: [],
  loaded: false,

  load: async () => {
    const [rawSettings, themes] = await Promise.all([
      tauri.settings.list().catch(() => []),
      tauri.themes.list().catch(() => []),
    ]);

    const settingsMap = new Map(rawSettings.map((s) => [s.key, s.value]));
    const themeId = settingsMap.get(SETTING_THEME_ID) ?? DEFAULTS.themeId;
    const fontSize =
      Number(settingsMap.get(SETTING_FONT_SIZE)) || DEFAULTS.fontSize;
    const scrollback =
      Number(settingsMap.get(SETTING_SCROLLBACK)) || DEFAULTS.scrollback;
    const autolockSecs = (() => {
      const v = settingsMap.get(SETTING_AUTOLOCK_SECS);
      const n = v === undefined ? NaN : Number(v);
      return Number.isFinite(n) && n >= 0 ? n : DEFAULTS.autolockSecs;
    })();
    const cursorStyle = (settingsMap.get(SETTING_CURSOR_STYLE) ??
      DEFAULTS.cursorStyle) as CursorStyle;
    const cursorBlink = settingsMap.get(SETTING_CURSOR_BLINK) !== 'false';

    const merged: AppSettings = {
      themeId,
      fontSize,
      scrollback,
      autolockSecs,
      cursorStyle,
      cursorBlink,
    };

    set({ settings: merged, themes, loaded: true });
    applyTheme(get().resolveTheme(themeId));
  },

  setTheme: async (themeId) => {
    set({ settings: { ...get().settings, themeId } });
    applyTheme(get().resolveTheme(themeId));
    await tauri.settings.set(SETTING_THEME_ID, themeId);
  },

  setFontSize: async (px) => {
    set({ settings: { ...get().settings, fontSize: px } });
    await tauri.settings.set(SETTING_FONT_SIZE, String(px));
  },

  setScrollback: async (lines) => {
    set({ settings: { ...get().settings, scrollback: lines } });
    await tauri.settings.set(SETTING_SCROLLBACK, String(lines));
  },

  setAutolockSecs: async (secs) => {
    set({ settings: { ...get().settings, autolockSecs: secs } });
    await tauri.settings.set(SETTING_AUTOLOCK_SECS, String(secs));
  },

  setCursorStyle: async (style) => {
    set({ settings: { ...get().settings, cursorStyle: style } });
    await tauri.settings.set(SETTING_CURSOR_STYLE, style);
  },

  setCursorBlink: async (blink) => {
    set({ settings: { ...get().settings, cursorBlink: blink } });
    await tauri.settings.set(SETTING_CURSOR_BLINK, blink ? 'true' : 'false');
  },

  saveTheme: async (def) => {
    const saved = await tauri.themes.save({
      id: def.id,
      name: def.name,
      base: def.base,
      definition: JSON.stringify(def),
    });
    const next = get().themes.filter((t) => t.id !== saved.id);
    next.push(saved);
    set({ themes: next });
  },

  deleteTheme: async (id) => {
    await tauri.themes.delete(id);
    const next = get().themes.filter((t) => t.id !== id);
    set({ themes: next });
    // If currently active theme was deleted, fall back to default.
    if (get().settings.themeId === id) {
      await get().setTheme(DEFAULTS.themeId);
    }
  },

  resolveTheme: (id) => {
    const customRow = get().themes.find((t) => t.id === id);
    if (customRow) {
      const def = parseDefinition(customRow.definition);
      if (def) return def;
    }
    const builtin = builtinThemes.find((t) => t.id === id);
    if (builtin) return builtin;
    // Final fallback to first builtin
    return builtinThemes[0]!;
  },
}));
