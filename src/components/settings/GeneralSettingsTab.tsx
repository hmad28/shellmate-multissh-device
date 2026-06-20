import { useEffect, useState } from 'react';
import { strings } from '@/i18n/en';
import { tauri } from '@/lib/tauri';

export function GeneralSettingsTab() {
  const [version, setVersion] = useState('...');

  useEffect(() => {
    let mounted = true;
    void tauri.app
      .version()
      .then((value) => {
        if (mounted) setVersion(value);
      })
      .catch(() => {
        if (mounted) setVersion('unknown');
      });
    return () => {
      mounted = false;
    };
  }, []);

  return (
    <section className="flex flex-col gap-4">
      <header>
        <h2 className="text-sm font-semibold text-fg">
          {strings.settings.generalHeading}
        </h2>
        <p className="text-xs text-fg-muted">
          {strings.settings.generalDescription}
        </p>
      </header>

      <dl className="flex flex-col gap-2 text-xs">
        <Row label={strings.settings.appName} value="ShellMate" />
        <Row label={strings.settings.version} value={version} />
        <Row label={strings.settings.license} value="MIT" />
      </dl>

      <p className="text-xs text-fg-subtle">
        {strings.settings.moreSettingsHint}
      </p>
    </section>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between rounded-md border border-border-subtle bg-bg-elevated px-3 py-2">
      <dt className="text-fg-muted">{label}</dt>
      <dd className="font-mono text-fg">{value}</dd>
    </div>
  );
}
