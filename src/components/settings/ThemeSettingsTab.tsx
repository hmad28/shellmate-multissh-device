import { Check, Trash2 } from 'lucide-react';
import { strings } from '@/i18n/en';
import { Button } from '@/components/ui/Button';
import { ConfirmDialog } from '@/components/ui/ConfirmDialog';
import { cn } from '@/lib/cn';
import { useState } from 'react';
import { useSettingsStore } from '@/stores/settings-store';
import { builtinThemes } from '@/themes/builtin';
import type { ThemeDefinition } from '@/types/theme';

export function ThemeSettingsTab() {
  const settings = useSettingsStore((s) => s.settings);
  const themes = useSettingsStore((s) => s.themes);
  const setTheme = useSettingsStore((s) => s.setTheme);
  const deleteTheme = useSettingsStore((s) => s.deleteTheme);
  const resolveTheme = useSettingsStore((s) => s.resolveTheme);

  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  // Combined list — builtins always present + user-saved themes from DB.
  const builtinIds = new Set(builtinThemes.map((t) => t.id));
  const customThemes = themes.filter((t) => !builtinIds.has(t.id));
  const allThemeIds = [
    ...builtinThemes.map((t) => t.id),
    ...customThemes.map((t) => t.id),
  ];

  const items: Array<{ id: string; def: ThemeDefinition; isBuiltin: boolean }> =
    allThemeIds.map((id) => ({
      id,
      def: resolveTheme(id),
      isBuiltin: builtinIds.has(id),
    }));

  return (
    <section className="flex flex-col gap-4">
      <header>
        <h2 className="text-sm font-semibold text-fg">
          {strings.settings.themeHeading}
        </h2>
        <p className="text-xs text-fg-muted">
          {strings.settings.themeDescription}
        </p>
      </header>

      <div className="grid grid-cols-2 gap-3">
        {items.map((item) => (
          <ThemeCard
            key={item.id}
            theme={item.def}
            isBuiltin={item.isBuiltin}
            isActive={settings.themeId === item.id}
            onApply={() => void setTheme(item.id)}
            onDelete={
              !item.isBuiltin ? () => setConfirmDelete(item.id) : undefined
            }
          />
        ))}
      </div>

      <p className="text-xs text-fg-subtle">
        {strings.settings.themeEditorHint}
      </p>

      <ConfirmDialog
        open={!!confirmDelete}
        title={strings.settings.themeDeleteTitle}
        body={strings.settings.themeDeleteBody}
        confirmLabel={strings.settings.themeDelete}
        variant="danger"
        onConfirm={async () => {
          if (!confirmDelete) return;
          await deleteTheme(confirmDelete);
          setConfirmDelete(null);
        }}
        onCancel={() => setConfirmDelete(null)}
      />
    </section>
  );
}

function ThemeCard({
  theme,
  isActive,
  isBuiltin,
  onApply,
  onDelete,
}: {
  theme: ThemeDefinition;
  isActive: boolean;
  isBuiltin: boolean;
  onApply: () => void;
  onDelete?: (() => void) | undefined;
}) {
  return (
    <div
      className={cn(
        'flex flex-col gap-2 rounded-md border p-3 transition-colors',
        isActive
          ? 'bg-accent/5 border-accent'
          : 'border-border-subtle bg-bg-elevated hover:border-border-strong',
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <span className="truncate text-sm font-medium text-fg">
          {theme.name}
        </span>
        <div className="flex items-center gap-1">
          {isActive && (
            <Check size={14} className="text-accent" aria-label="Active" />
          )}
          {!isBuiltin && onDelete && (
            <button
              type="button"
              onClick={onDelete}
              aria-label="Delete theme"
              className="text-fg-subtle hover:text-status-disconnected"
            >
              <Trash2 size={12} />
            </button>
          )}
        </div>
      </div>

      <div
        className="flex h-12 items-center gap-1 rounded border border-border-subtle p-1"
        style={{ backgroundColor: theme.terminal.background }}
      >
        {theme.terminal.ansi.slice(0, 8).map((c, i) => (
          <span
            key={i}
            className="h-full w-2 rounded-sm"
            style={{ backgroundColor: c }}
            aria-hidden="true"
          />
        ))}
      </div>

      <div className="text-xs text-fg-subtle">
        {theme.base === 'dark' ? 'Dark' : 'Light'}
        {isBuiltin && ' • Built-in'}
      </div>

      {!isActive && (
        <Button variant="secondary" size="sm" onClick={onApply}>
          {strings.settings.themeApply}
        </Button>
      )}
    </div>
  );
}
