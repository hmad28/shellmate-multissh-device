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
      <div className="flex min-h-[420px] gap-4">
        <nav
          aria-label="Settings sections"
          className="w-36 shrink-0 border-r border-border-subtle pr-2"
        >
          <ul className="flex flex-col gap-0.5">
            {tabs.map((t) => (
              <li key={t.id}>
                <button
                  type="button"
                  onClick={() => setTab(t.id)}
                  aria-current={tab === t.id ? 'page' : undefined}
                  className={cn(
                    'w-full rounded-md px-3 py-1.5 text-left text-sm transition-colors',
                    tab === t.id
                      ? 'bg-bg-elevated text-fg'
                      : 'text-fg-muted hover:bg-bg-elevated hover:text-fg',
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
