import { useState } from 'react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { Modal } from '@/components/ui/Modal';
import { GeneralSettingsTab } from './GeneralSettingsTab';
import { TerminalSettingsTab } from './TerminalSettingsTab';
import { VaultSettingsTab } from './VaultSettingsTab';
import { ThemeSettingsTab } from './ThemeSettingsTab';

type Tab = 'general' | 'terminal' | 'vault' | 'theme';

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
  initialTab?: Tab;
}

export function SettingsDialog({
  open,
  onClose,
  initialTab = 'general',
}: SettingsDialogProps) {
  const [tab, setTab] = useState<Tab>(initialTab);

  const tabs: { id: Tab; label: string }[] = [
    { id: 'general', label: strings.settings.tabGeneral },
    { id: 'terminal', label: strings.settings.tabTerminal },
    { id: 'vault', label: strings.settings.tabVault },
    { id: 'theme', label: strings.settings.tabTheme },
  ];

  return (
    <Modal
      open={open}
      onClose={onClose}
      title={strings.settings.title}
      size="lg"
    >
      <div className="flex min-h-[420px] flex-col gap-4">
        <nav
          aria-label="Settings sections"
          className="shrink-0 border-b border-border-subtle pb-3"
        >
          <ul className="flex gap-1 overflow-x-auto rounded-md bg-bg-elevated p-1">
            {tabs.map((t) => (
              <li key={t.id} className="shrink-0">
                <button
                  type="button"
                  onClick={() => setTab(t.id)}
                  aria-current={tab === t.id ? 'page' : undefined}
                  className={cn(
                    'rounded px-3 py-1.5 text-sm transition-colors',
                    tab === t.id
                      ? 'bg-bg text-fg shadow-sm'
                      : 'text-fg-muted hover:bg-bg-panel hover:text-fg',
                  )}
                >
                  {t.label}
                </button>
              </li>
            ))}
          </ul>
        </nav>

        <div className="min-w-0 flex-1">
          {tab === 'general' && <GeneralSettingsTab />}
          {tab === 'terminal' && <TerminalSettingsTab />}
          {tab === 'vault' && <VaultSettingsTab />}
          {tab === 'theme' && <ThemeSettingsTab />}
        </div>
      </div>
    </Modal>
  );
}
