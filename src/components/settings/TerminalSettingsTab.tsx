import { strings } from '@/i18n/en';
import { Field, Input, Select } from '@/components/ui/Form';
import { type CursorStyle, useSettingsStore } from '@/stores/settings-store';

export function TerminalSettingsTab() {
  const settings = useSettingsStore((s) => s.settings);
  const setFontSize = useSettingsStore((s) => s.setFontSize);
  const setScrollback = useSettingsStore((s) => s.setScrollback);
  const setCursorStyle = useSettingsStore((s) => s.setCursorStyle);
  const setCursorBlink = useSettingsStore((s) => s.setCursorBlink);

  return (
    <section className="flex flex-col gap-4">
      <header>
        <h2 className="text-sm font-semibold text-fg">
          {strings.settings.terminalHeading}
        </h2>
        <p className="text-xs text-fg-muted">
          {strings.settings.terminalDescription}
        </p>
      </header>

      <Field label={strings.settings.fontSize} htmlFor="setting-fontSize">
        <Input
          id="setting-fontSize"
          type="number"
          min={8}
          max={32}
          value={settings.fontSize}
          onChange={(e) => {
            const n = Number(e.target.value);
            if (Number.isFinite(n) && n >= 8 && n <= 32) {
              void setFontSize(n);
            }
          }}
        />
      </Field>

      <Field label={strings.settings.scrollback} htmlFor="setting-scrollback">
        <Input
          id="setting-scrollback"
          type="number"
          min={500}
          max={100_000}
          step={500}
          value={settings.scrollback}
          onChange={(e) => {
            const n = Number(e.target.value);
            if (Number.isFinite(n) && n >= 100 && n <= 1_000_000) {
              void setScrollback(n);
            }
          }}
        />
      </Field>

      <Field label={strings.settings.cursorStyle} htmlFor="setting-cursorStyle">
        <Select
          id="setting-cursorStyle"
          value={settings.cursorStyle}
          onChange={(e) => void setCursorStyle(e.target.value as CursorStyle)}
        >
          <option value="block">Block</option>
          <option value="bar">Bar</option>
          <option value="underline">Underline</option>
        </Select>
      </Field>

      <label className="flex items-center gap-2 text-xs text-fg">
        <input
          type="checkbox"
          checked={settings.cursorBlink}
          onChange={(e) => void setCursorBlink(e.target.checked)}
          className="accent-accent"
        />
        <span>{strings.settings.cursorBlink}</span>
      </label>
    </section>
  );
}
