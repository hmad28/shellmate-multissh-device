import { useEffect, useState } from 'react';
import { strings } from '@/i18n/en';
import { cn } from '@/lib/cn';
import { useVaultStore } from '@/stores/vault-store';
import { useIsMobile } from '@/hooks/useIsMobile';
import { VaultSetup } from './VaultSetup';
import { VaultUnlock } from './VaultUnlock';
import { MobileOnboard } from './MobileOnboard';

/**
 * Gates the rest of the app behind vault setup/unlock.
 * Renders children only when vault is unlocked.
 */
export function VaultGate({ children }: { children: React.ReactNode }) {
  const { initialized, unlocked, loading, refresh } = useVaultStore();
  const [bootstrapped, setBootstrapped] = useState(false);
  const isMobile = useIsMobile();

  useEffect(() => {
    void refresh().finally(() => setBootstrapped(true));
  }, [refresh]);

  if (!bootstrapped || (loading && !initialized && !unlocked)) {
    return <Splash label="Loading..." />;
  }

  if (unlocked) {
    return <>{children}</>;
  }

  // Mobile: use the satellite-device onboarding flow
  if (isMobile) {
    return <MobileOnboard />;
  }

  // Desktop: traditional vault setup/unlock

  return (
    <div className="flex h-full w-full items-center justify-center bg-bg p-6">
      <div
        className={cn(
          'w-full max-w-md rounded-lg border border-border bg-bg-panel p-6 shadow-lg',
        )}
      >
        <header className="mb-5 flex items-center gap-3">
          <Mark />
          <div>
            <h1 className="text-base font-semibold text-fg">
              {strings.vault.title}
            </h1>
            <p className="text-xs text-fg-muted">
              {initialized
                ? strings.vault.unlockSubtitle
                : strings.vault.setupSubtitle}
            </p>
          </div>
        </header>
        {initialized ? <VaultUnlock /> : <VaultSetup />}
      </div>
    </div>
  );
}

function Splash({ label }: { label: string }) {
  return (
    <div className="flex h-full w-full items-center justify-center bg-bg text-fg-muted">
      <div className="flex items-center gap-3">
        <Mark />
        <span className="text-sm">{label}</span>
      </div>
    </div>
  );
}

function Mark() {
  return (
    <svg
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className="text-accent"
      aria-hidden="true"
    >
      <polyline points="4 17 10 11 4 5" />
      <line x1="12" y1="19" x2="20" y2="19" />
    </svg>
  );
}
